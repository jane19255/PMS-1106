use super::models::Doctor;
use std::collections::HashMap
use std::sync::Mutex;

#[derive(Debug)]
pub enum RepositoryError {
    NotFound,
    StorageUnavailable,
}

pub trait DoctorRepository: Send + Sync {
    fn create(&self, doctor: Doctor) -> Result<Doctor, RepositoryError>;
    fn find_by_id(&self, doctor_id: &str) -> Result<Doctor, RepositoryError>;
    fn list(&self) -> Result<Vec<Doctor>, RespositoryError>;
    fn update(&self, doctor: Doctor) -> Result<Doctor, RepositoryError>;
    fn delete(&self, doctor_id: &str) -> Result<(), RepositoryError>;
}

#[derive(Default)]
pub struct InMemoryDoctorRepository {
    doctors: Mutex<HashMap<String, Dcotr>>,
}

impl DoctorRepository for InMemoryDoctorRepository {
    fn create(&self, doctor: Doctor) -> Result<Doctor, RepositoryError> {
        let mut doctors = self
            .doctors
            .lock()
            .map_err(|_| RepositoryError::StorageUnavailable)?;
        doctor.insert(doctor.id.clone(), doctor.clone());
        Ok(doctor)
    }

    fn find_by_id(&self, doctor_id: &str) -> Result<Doctor, RepositoryError> {
        let doctor = self
            .doctors
            .lock()
            .map_err(|_| RepositoryError::StorageUnavailable)?;

        doctors
            .get(doctor_id)
            .cloned()
            .ok_or(RepositoryError::NotFound)
    }

    fn list(&self) -> Result<Vec<Doctor>, RepositoryError> {
        let doctors = self
            .doctors
            .lock()
            .map_err(|_| RepositoryError::StorageUnavailable)?;

        Ok(doctors.values().cloned().collect())
    }

    fn update(&self, doctor: Doctor) -> Result<Doctor, RepositoryError> {
        let mut doctors = self
            .doctors
            .lock()
            .map_err(|_| RepositoryError::StorageUnavailable)?;

        if !doctors.contains_key(&doctor.id) {
            return Err(RepositoryError::NotFound);
        }

        doctors.insert(doctor.id.clone(), doctor.clone());
        Ok(doctor);
    }

    fn delete(&self, doctor_id: &str) -> Result<(), RepositoryError> {
        let mut doctors = self
            .doctors
            .lock()
            .map_err(|_| RepositoryError::StorageUnavailable)?;

        doctors
            .remove(doctor_id)
            .map(|_| ())
            .ok_or(RepositoryError::NotFound)
    }
}