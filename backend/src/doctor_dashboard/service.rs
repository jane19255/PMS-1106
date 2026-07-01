use super::models::DoctorQueueAppointment;
use super::repository::{DoctorDashboardRepository, RepositoryError};
use chrono::NaiveDate;
use std::sync::Arc;

#[derive(Debug)]
pub enum DoctorDashboardError {
    StorageUnavailable,
    Rejected(String),
}

pub struct DoctorDashboardService {
    repository: Arc<dyn DoctorDashboardRepository>,
}

impl DoctorDashboardService {
    pub fn new(repository: Arc<dyn DoctorDashboardRepository>) -> Self {
        Self { repository }
    }

    pub async fn list_appointments(
        &self,
        firebase_uid: &str,
        date: NaiveDate,
    ) -> Result<Vec<DoctorQueueAppointment>, DoctorDashboardError> {
        self.repository
            .list_appointments(firebase_uid, date)
            .await
            .map_err(Self::map_repository_error)
    }

    pub async fn start_consultation(
        &self,
        firebase_uid: &str,
        appointment_id: &str,
    ) -> Result<(), DoctorDashboardError> {
        self.repository
            .start_consultation(firebase_uid, appointment_id)
            .await
            .map_err(Self::map_repository_error)
    }

    pub async fn complete_consultation(
        &self,
        firebase_uid: &str,
        appointment_id: &str,
    ) -> Result<(), DoctorDashboardError> {
        self.repository
            .complete_consultation(firebase_uid, appointment_id)
            .await
            .map_err(Self::map_repository_error)
    }

    fn map_repository_error(error: RepositoryError) -> DoctorDashboardError {
        match error {
            RepositoryError::StorageUnavailable => DoctorDashboardError::StorageUnavailable,
            RepositoryError::Rejected(message) => DoctorDashboardError::Rejected(message),
        }
    }
}
