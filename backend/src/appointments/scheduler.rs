//! Appointment scheduler conflict helper.
//!
//! Builds on the interval tree to keep doctor appointment checks focused on time-slot
//! availability instead of route or database concerns.
use chrono::{DateTime, Duration, Utc};

use super::interval::AppointmentInterval;
use super::interval_tree::IntervalTree;

pub struct AppointmentScheduler {
    tree: IntervalTree,
    intervals: Vec<AppointmentInterval>,
}

impl AppointmentScheduler {
    pub fn new(intervals: Vec<AppointmentInterval>) -> Self {
        let mut tree = IntervalTree::new();

        for interval in &intervals {
            tree.insert(interval.clone());
        }

        Self {
            tree,
            intervals,
        }
    }

    pub fn has_conflict(
        &self,
        interval: &AppointmentInterval,
    ) -> bool {
        self.tree.find_overlap(interval).is_some()
    }

    pub fn suggest_slots(
        &self,
        duration_minutes: i64,
    ) -> Vec<(DateTime<Utc>, DateTime<Utc>)> {
        let mut intervals = self.intervals.clone();
        intervals.sort_by_key(|i| i.start);

        let mut suggestions = Vec::new();

        for window in intervals.windows(2) {
            let first = &window[0];
            let second = &window[1];

            let gap =
                second.start.signed_duration_since(first.end);

            if gap >= Duration::minutes(duration_minutes) {
                suggestions.push((
                    first.end,
                    first.end + Duration::minutes(duration_minutes),
                ));
            }
        }

        suggestions
    }
}