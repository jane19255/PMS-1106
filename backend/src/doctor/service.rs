use chrono::{DateTime, Utc};

use crate::doctor::repository::DoctorRepository;
use crate::models::SupabaseDoctorAvailabilityRow;

// =========================================================
// Doctor service
// Handles doctor working windows and availability checks.
// Used by appointment scheduling to validate doctor availability.
// =========================================================

#[derive(Clone)]
pub struct DoctorService {
    pub repository: DoctorRepository,
}

impl DoctorService {
    pub fn new(repository: DoctorRepository) -> Self {
        Self { repository }
    }

    /// Parse doctor availability JSON from DB into typed structs.
    pub fn parse_availability(raw: &str) -> Result<Vec<SupabaseDoctorAvailabilityRow>, String> {
        serde_json::from_str(raw).map_err(|e| format!("Failed to parse doctor availability: {}", e))
    }

    /// Check if doctor is available for the requested time window.
    /// Returns true if any availability interval fully covers the requested slot.
    pub async fn is_doctor_available(
        &self,
        doctor_id: &str,
        requested_start: DateTime<Utc>,
        requested_end: DateTime<Utc>,
    ) -> Result<bool, String> {
        let raw = self.repository.list_availability(doctor_id).await?;
        let rows = Self::parse_availability(&raw)?;

        for row in rows {
            if !row.is_available {
                continue;
            }

            let available_from = row
                .available_from
                .parse::<DateTime<Utc>>()
                .map_err(|e| format!("Invalid available_from datetime: {}", e))?;

            let available_to = row
                .available_to
                .parse::<DateTime<Utc>>()
                .map_err(|e| format!("Invalid available_to datetime: {}", e))?;

            if requested_start >= available_from && requested_end <= available_to {
                return Ok(true);
            }
        }

        Ok(false)
    }
}