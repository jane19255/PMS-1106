use super::models::DashboardAppointment;
use super::repository::{DoctorDashboardRepository, RepositoryError};
use std::sync::Arc;

#[derive(Debug)]
pub enum DoctorDashboardError {
    StorageUnavailable,
}

pub struct DoctorDashboardService {
    repository: Arc<dyn DoctorDashboardRepository>,
}

impl DoctorDashboardService {
    pub fn new(repository: Arc<dyn DoctorDashboardRepository>) -> Self {
        Self { repository }
    }

    pub async fn list_appointments(&self) -> Result<Vec<DashboardAppointment>, DoctorDashboardError> {
        self.repository
            .list_appointments()
            .await
            .map_err(Self::map_repository_error)
    }

    fn map_repository_error(error: RepositoryError) -> DoctorDashboardError {
        match error {
            RepositoryError::StorageUnavailable => DoctorDashboardError::StorageUnavailable,
        }
    }
}