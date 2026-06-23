use super::models::{Doctor, DoctorStatus};
use super::repository::{DoctorRepository, RepositoryError};
use serde::Deserialize;
use std::sync::Arc;
use uuid::Uuid;

pub enum DoctorError {
    InvalidInput(String),
    DoctorNotFound,
    StorageUnavailable,
}

#[derive(Deserialize)]
pub struct CreateDoctorForm {
    pub name: String,
    pub specialization: String,
    pub contact_number: String,
    pub email: String,
}

pub struct DoctorService {
    doctor_repository: Arc<dyn DoctorRepository>,
}

impl DoctorService {
    pub fn new(doctor_repository: Arc<dyn DoctorRepository>) -> Self {
        Self { doctor_repository }
    }

    pub fn create_doctor(&self, form: CreateDoctorForm) -> Result<Doctor, DoctorError> {
        self.validate_create_doctor(&form)?;

        let doctor = Doctor {
            id: format!("DOC-{}", Uuid::new_v4()),
            name: form.name.trim().to_string(),
            specialization: form.specialization.trim().to_string(),
            contact_number: form.contact_number.trim().to_string(),
            email: form.email.trim().to_string(),
            status: DoctorStatus::Available,
        };

        self.doctor_repository
            .create(doctor)
            .map_err(Self::map_repository_error)
    }

    fn validate_create_doctor(&self, form: &CreateDoctorForm) -> Result<(), DoctorError> {
        if form.name.trim().is_empty() {
            return Err(DoctorError::InvalidInput("Doctor name is required".to_string()));
        }

        if form.specialization.trim().is_empty() {
            return Err(DoctorError::InvalidInput("Specialization is required".to_string()));
        }

        if form.contact_number.trim().is_empty() {
            return Err(DoctorError::InvalidInput("Contact number is required".to_string()));
        }

        if form.email.trim().is_empty() {
            return Err(DoctorError::InvalidInput("Email is required".to_string()));
        }

        Ok(())
    }

    fn map_repository_error(error: RepositoryError) -> DoctorError {
        match error {
            RepositoryError::NotFound => DoctorError::DoctorNotFound,
            RepositoryError::StorageUnavailable => DoctorError::StorageUnavailable,
        }
    }
}
