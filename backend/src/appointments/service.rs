use chrono::{DateTime, Duration, Utc};
use std::cmp::Reverse;
use std::collections::BinaryHeap;

use crate::appointments::interval_tree::{IntervalTree, TimeInterval};
use crate::appointments::repository::AppointmentRepository;
use crate::models::{CreateAppointmentRequest, SupabaseAppointmentRow};

// =========================================================
// Appointment service
// Contains booking rules, conflict validation, and
// earliest available slot algorithm.
// =========================================================

#[derive(Clone)]
pub struct AppointmentService {
    pub repository: AppointmentRepository,
}

impl AppointmentService {
    pub fn new(repository: AppointmentRepository) -> Self {
        Self { repository }
    }

    /// Basic overlap validation using interval checks.
    pub fn validate_no_overlap(
        existing: &[TimeInterval],
        requested: &TimeInterval,
    ) -> Result<(), String> {
        if existing.iter().any(|item| item.overlaps(requested)) {
            return Err("Requested appointment conflicts with an existing booking".to_string());
        }
        Ok(())
    }

    /// Interval tree validation for more scalable overlap checks.
    pub fn validate_with_interval_tree(
        existing: &[TimeInterval],
        requested: &TimeInterval,
    ) -> Result<(), String> {
        let mut tree = IntervalTree::new();
        for interval in existing {
            tree.insert(interval.clone());
        }

        if tree.has_overlap(requested) {
            return Err("Requested appointment overlaps with another appointment".to_string());
        }

        Ok(())
    }

    /// Earliest available slot algorithm.
    /// Uses a min-heap idea through Reverse + BinaryHeap to process earliest intervals first.
    pub fn find_earliest_available_slot(
        existing: &[TimeInterval],
        requested_start: DateTime<Utc>,
        duration_minutes: i64,
    ) -> TimeInterval {
        let mut heap: BinaryHeap<Reverse<(DateTime<Utc>, DateTime<Utc>)>> = BinaryHeap::new();

        for interval in existing {
            heap.push(Reverse((interval.start, interval.end)));
        }

        let duration = Duration::minutes(duration_minutes);
        let mut candidate_start = requested_start;

        while let Some(Reverse((start, end))) = heap.pop() {
            let candidate_end = candidate_start + duration;

            // If candidate slot ends before the next booking starts, we found a gap.
            if candidate_end <= start {
                return TimeInterval {
                    start: candidate_start,
                    end: candidate_end,
                };
            }

            // If candidate overlaps or starts during occupied time, move forward.
            if candidate_start < end {
                candidate_start = end;
            }
        }

        TimeInterval {
            start: candidate_start,
            end: candidate_start + duration,
        }
    }

    /// Parse Supabase JSON array into appointment rows.
    pub fn parse_appointments(raw: &str) -> Result<Vec<SupabaseAppointmentRow>, String> {
        serde_json::from_str(raw).map_err(|e| format!("Failed to parse appointments: {}", e))
    }

    /// Convert rows into time intervals for scheduling logic.
    pub fn to_intervals(rows: &[SupabaseAppointmentRow]) -> Result<Vec<TimeInterval>, String> {
        rows.iter()
            .map(|row| {
                let start = row
                    .appointment_datetime
                    .parse::<DateTime<Utc>>()
                    .map_err(|e| format!("Invalid appointment datetime: {}", e))?;

                let end = start + Duration::minutes(row.duration_minutes as i64);

                Ok(TimeInterval { start, end })
            })
            .collect()
    }

    /// Main create flow with conflict validation and fallback suggestion.
    pub async fn create_with_validation(
        &self,
        payload: &CreateAppointmentRequest,
    ) -> Result<String, String> {
        let existing_raw = self.repository.list_by_doctor(&payload.doctor_id).await?;
        let rows = Self::parse_appointments(&existing_raw)?;
        let intervals = Self::to_intervals(&rows)?;

        let requested_start = payload
            .appointment_datetime
            .parse::<DateTime<Utc>>()
            .map_err(|e| format!("Invalid requested datetime: {}", e))?;

        let requested_end = requested_start + Duration::minutes(payload.duration_minutes as i64);

        let requested = TimeInterval {
            start: requested_start,
            end: requested_end,
        };

        match Self::validate_with_interval_tree(&intervals, &requested) {
            Ok(_) => self.repository.create(payload).await,
            Err(_) => {
                let suggestion = Self::find_earliest_available_slot(
                    &intervals,
                    requested_start,
                    payload.duration_minutes as i64,
                );

                Err(format!(
                    "Requested slot is unavailable. Suggested slot: {} to {}",
                    suggestion.start, suggestion.end
                ))
            }
        }
    }
}