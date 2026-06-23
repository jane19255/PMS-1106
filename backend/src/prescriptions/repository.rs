use super::models::Prescription;
use std::collections::HashMap;
use std::sync::Mutex;

#[derive(Debug)]
pub enum RepositoryError {
    NotFound,
    StorageUnavailable,
}

pub trait PrescriptionRepository: Send + Sync {
    fn create(&self, prescription: Prescription) -> Result<Prescription, RepositoryError>;
    fn find_by_id(&self, prescription_id: &str) -> Result<Prescription, RepositoryError>;
    fn list(&self) -> Result<Vec<Prescription>, RepositoryError>;
    fn list_by_patient(&self, patient_id: &str) -> Result<Vec<Prescription>, RepositoryError>;
    fn list_by_doctor(&self, doctor_id: &str) -> Result<Vec<Prescription>, RepositoryError>;
    fn update(&self, prescription: Prescription) -> Result<Prescription, RepositoryError>;
}

#[derive(Default)]
pub struct InMemoryPrescriptionRepository {
    prescriptions: Mutex<HashMap<String, Prescription>>,
}

impl PrescriptionRepository for InMemoryPrescriptionRepository {
    fn create(&self, prescription: Prescription) -> Result<Prescription, RepositoryError> {
        let mut prescriptions = self
            .prescriptions
            .lock()
            .map_err(|_| RepositoryError::StorageUnavailable)?;
        prescriptions.insert(prescription.id.clone(), prescription.clone());
        Ok(prescription)
    }

    fn find_by_id(&self, prescription_id: &str) -> Result<Prescription, RepositoryError> {
        let prescriptions = self
            .prescriptions
            .lock()
            .map_err(|_| RepositoryError::StorageUnavailable)?;

        prescriptions
            .get(prescription_id)
            .cloned()
            .ok_or(RepositoryError::NotFound)
    }

    fn list(&self) -> Result<Vec<Prescription>, RepositoryError> {
        let prescriptions = self
            .prescriptions
            .lock()
            .map_err(|_| RepositoryError::StorageUnavailable)?;

        let mut prescription_list: Vec<Prescription> = prescriptions.values().cloned().collect();
        prescription_list.sort_by(|left, right| right.issued_at.cmp(&left.issued_at));
        Ok(prescription_list)
    }

    fn list_by_patient(&self, patient_id: &str) -> Result<Vec<Prescription>, RepositoryError> {
        let prescriptions = self.list()?;
        Ok(prescriptions
            .into_iter()
            .filter(|prescription| prescription.patient_id == patient_id)
            .collect())
    }

    fn list_by_doctor(&self, doctor_id: &str) -> Result<Vec<Prescription>, RepositoryError> {
        let prescriptions = self.list()?;
        Ok(prescriptions
            .into_iter()
            .filter(|prescription| prescription.doctor_id == doctor_id)
            .collect())
    }

    fn update(&self, prescription: Prescription) -> Result<Prescription, RepositoryError> {
        let mut prescriptions = self
            .prescriptions
            .lock()
            .map_err(|_| RepositoryError::StorageUnavailable)?;

        if !prescriptions.contains_key(&prescription.id) {
            return Err(RepositoryError::NotFound);
        }

        prescriptions.insert(prescription.id.clone(), prescription.clone());
        Ok(prescription)
    }
}
