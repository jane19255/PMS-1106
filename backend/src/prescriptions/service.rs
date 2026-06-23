use super::models::{Prescription, PrescriptionStatus};
use super::repository::{PrescriptionRepository, RepositoryError};
use chrono::Utc;
use serde::Deserialize;
use std::sync::Arc;
use uuid::Uuid;

#[derive(Debug, PartialEq)]
pub enum PrescriptionError {
    InvalidInput(String),
    PrescriptionNotFound,
    StorageUnavailable,
}

#[derive(Clone, Debug, Deserialize)]
pub struct CreatePrescriptionForm {
    pub patient_id: String,
    pub doctor_id: String,
    pub medical_record_id: Option<String>,
    pub medication_name: String,
    pub dosage: String,
    pub frequency: String,
    pub duration: String,
    pub instructions: Option<String>,
    pub quantity: String,
    pub unit_cost: String,
}

#[derive(Clone, Debug, Deserialize)]
pub struct UpdatePrescriptionForm {
    pub medication_name: String,
    pub dosage: String,
    pub frequency: String,
    pub duration: String,
    pub instructions: Option<String>,
    pub quantity: String,
    pub unit_cost: String,
}

pub struct PrescriptionService {
    prescription_repository: Arc<dyn PrescriptionRepository>,
}

impl PrescriptionService {
    pub fn new(prescription_repository: Arc<dyn PrescriptionRepository>) -> Self {
        Self {
            prescription_repository,
        }
    }

    pub fn create_prescription(
        &self,
        form: CreatePrescriptionForm,
    ) -> Result<Prescription, PrescriptionError> {
        self.validate_prescription_fields(
            &form.patient_id,
            &form.doctor_id,
            &form.medication_name,
            &form.dosage,
            &form.frequency,
            &form.duration,
        )?;
        let quantity = self.parse_quantity(&form.quantity)?;
        let unit_cost = self.parse_unit_cost(&form.unit_cost)?;

        let prescription = Prescription {
            id: format!("RX-{}", Uuid::new_v4()),
            patient_id: form.patient_id.trim().to_string(),
            doctor_id: form.doctor_id.trim().to_string(),
            medical_record_id: form
                .medical_record_id
                .map(|id| id.trim().to_string())
                .filter(|id| !id.is_empty()),
            medication_name: form.medication_name.trim().to_string(),
            dosage: form.dosage.trim().to_string(),
            frequency: form.frequency.trim().to_string(),
            duration: form.duration.trim().to_string(),
            instructions: form
                .instructions
                .map(|instructions| instructions.trim().to_string())
                .filter(|instructions| !instructions.is_empty()),
            quantity,
            unit_cost,
            status: PrescriptionStatus::Active,
            issued_at: Utc::now(),
            dispensed_at: None,
            cancelled_at: None,
        };

        self.prescription_repository
            .create(prescription)
            .map_err(Self::map_repository_error)
    }

    pub fn list_prescriptions(&self) -> Result<Vec<Prescription>, PrescriptionError> {
        self.prescription_repository
            .list()
            .map_err(Self::map_repository_error)
    }

    pub fn list_prescriptions_for_patient(
        &self,
        patient_id: &str,
    ) -> Result<Vec<Prescription>, PrescriptionError> {
        if patient_id.trim().is_empty() {
            return Err(PrescriptionError::InvalidInput(
                "Patient ID is required".to_string(),
            ));
        }

        self.prescription_repository
            .list_by_patient(patient_id.trim())
            .map_err(Self::map_repository_error)
    }

    pub fn list_prescriptions_for_doctor(
        &self,
        doctor_id: &str,
    ) -> Result<Vec<Prescription>, PrescriptionError> {
        if doctor_id.trim().is_empty() {
            return Err(PrescriptionError::InvalidInput(
                "Doctor ID is required".to_string(),
            ));
        }

        self.prescription_repository
            .list_by_doctor(doctor_id.trim())
            .map_err(Self::map_repository_error)
    }

    pub fn find_prescription(
        &self,
        prescription_id: &str,
    ) -> Result<Prescription, PrescriptionError> {
        if prescription_id.trim().is_empty() {
            return Err(PrescriptionError::InvalidInput(
                "Prescription ID is required".to_string(),
            ));
        }

        self.prescription_repository
            .find_by_id(prescription_id.trim())
            .map_err(Self::map_repository_error)
    }

    pub fn update_prescription(
        &self,
        prescription_id: &str,
        form: UpdatePrescriptionForm,
    ) -> Result<Prescription, PrescriptionError> {
        let mut prescription = self.find_prescription(prescription_id)?;

        if prescription.status != PrescriptionStatus::Active {
            return Err(PrescriptionError::InvalidInput(
                "Only active prescriptions can be updated".to_string(),
            ));
        }

        self.validate_prescription_fields(
            &prescription.patient_id,
            &prescription.doctor_id,
            &form.medication_name,
            &form.dosage,
            &form.frequency,
            &form.duration,
        )?;
        prescription.medication_name = form.medication_name.trim().to_string();
        prescription.dosage = form.dosage.trim().to_string();
        prescription.frequency = form.frequency.trim().to_string();
        prescription.duration = form.duration.trim().to_string();
        prescription.instructions = form
            .instructions
            .map(|instructions| instructions.trim().to_string())
            .filter(|instructions| !instructions.is_empty());
        prescription.quantity = self.parse_quantity(&form.quantity)?;
        prescription.unit_cost = self.parse_unit_cost(&form.unit_cost)?;

        self.prescription_repository
            .update(prescription)
            .map_err(Self::map_repository_error)
    }

    pub fn dispense_prescription(
        &self,
        prescription_id: &str,
    ) -> Result<Prescription, PrescriptionError> {
        let mut prescription = self.find_prescription(prescription_id)?;

        if prescription.status != PrescriptionStatus::Active {
            return Err(PrescriptionError::InvalidInput(
                "Only active prescriptions can be dispensed".to_string(),
            ));
        }

        prescription.status = PrescriptionStatus::Dispensed;
        prescription.dispensed_at = Some(Utc::now());

        self.prescription_repository
            .update(prescription)
            .map_err(Self::map_repository_error)
    }

    pub fn cancel_prescription(
        &self,
        prescription_id: &str,
    ) -> Result<Prescription, PrescriptionError> {
        let mut prescription = self.find_prescription(prescription_id)?;

        if prescription.status == PrescriptionStatus::Dispensed {
            return Err(PrescriptionError::InvalidInput(
                "Dispensed prescriptions cannot be cancelled".to_string(),
            ));
        }
        if prescription.status == PrescriptionStatus::Cancelled {
            return Err(PrescriptionError::InvalidInput(
                "Prescription is already cancelled".to_string(),
            ));
        }

        prescription.status = PrescriptionStatus::Cancelled;
        prescription.cancelled_at = Some(Utc::now());

        self.prescription_repository
            .update(prescription)
            .map_err(Self::map_repository_error)
    }

    fn validate_prescription_fields(
        &self,
        patient_id: &str,
        doctor_id: &str,
        medication_name: &str,
        dosage: &str,
        frequency: &str,
        duration: &str,
    ) -> Result<(), PrescriptionError> {
        for (field_name, value) in [
            ("Patient ID", patient_id),
            ("Doctor ID", doctor_id),
            ("Medication name", medication_name),
            ("Dosage", dosage),
            ("Frequency", frequency),
            ("Duration", duration),
        ] {
            if value.trim().is_empty() {
                return Err(PrescriptionError::InvalidInput(format!(
                    "{field_name} is required"
                )));
            }
        }

        Ok(())
    }

    fn parse_quantity(&self, value: &str) -> Result<u32, PrescriptionError> {
        let quantity = value.trim().parse::<u32>().map_err(|_| {
            PrescriptionError::InvalidInput("Quantity must be a whole number".to_string())
        })?;

        if quantity == 0 {
            return Err(PrescriptionError::InvalidInput(
                "Quantity must be greater than zero".to_string(),
            ));
        }

        Ok(quantity)
    }

    fn parse_unit_cost(&self, value: &str) -> Result<f64, PrescriptionError> {
        if value.trim().is_empty() {
            return Ok(0.0);
        }

        let unit_cost = value.trim().parse::<f64>().map_err(|_| {
            PrescriptionError::InvalidInput("Unit cost must be a valid number".to_string())
        })?;

        if !unit_cost.is_finite() || unit_cost < 0.0 {
            return Err(PrescriptionError::InvalidInput(
                "Unit cost cannot be negative".to_string(),
            ));
        }

        Ok((unit_cost * 100.0).round() / 100.0)
    }

    fn map_repository_error(error: RepositoryError) -> PrescriptionError {
        match error {
            RepositoryError::NotFound => PrescriptionError::PrescriptionNotFound,
            RepositoryError::StorageUnavailable => PrescriptionError::StorageUnavailable,
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::prescriptions::repository::InMemoryPrescriptionRepository;

    fn service() -> PrescriptionService {
        PrescriptionService::new(Arc::new(InMemoryPrescriptionRepository::default()))
    }

    fn create_form() -> CreatePrescriptionForm {
        CreatePrescriptionForm {
            patient_id: "PAT-001".to_string(),
            doctor_id: "DOC-001".to_string(),
            medical_record_id: Some("MR-001".to_string()),
            medication_name: "Amoxicillin".to_string(),
            dosage: "500mg".to_string(),
            frequency: "Three times daily".to_string(),
            duration: "7 days".to_string(),
            instructions: Some("Take after meals".to_string()),
            quantity: "21".to_string(),
            unit_cost: "1.25".to_string(),
        }
    }

    #[test]
    fn creates_active_prescription() {
        let prescription = service().create_prescription(create_form()).unwrap();

        assert!(prescription.id.starts_with("RX-"));
        assert_eq!(prescription.status, PrescriptionStatus::Active);
        assert_eq!(prescription.quantity, 21);
        assert_eq!(prescription.unit_cost, 1.25);
    }

    #[test]
    fn rejects_missing_patient_id() {
        let mut form = create_form();
        form.patient_id = " ".to_string();

        let error = service().create_prescription(form).unwrap_err();

        assert_eq!(
            error,
            PrescriptionError::InvalidInput("Patient ID is required".to_string())
        );
    }

    #[test]
    fn dispenses_active_prescription() {
        let service = service();
        let prescription = service.create_prescription(create_form()).unwrap();

        let dispensed = service.dispense_prescription(&prescription.id).unwrap();

        assert_eq!(dispensed.status, PrescriptionStatus::Dispensed);
        assert!(dispensed.dispensed_at.is_some());
    }

    #[test]
    fn rejects_cancel_after_dispense() {
        let service = service();
        let prescription = service.create_prescription(create_form()).unwrap();
        service.dispense_prescription(&prescription.id).unwrap();

        let error = service.cancel_prescription(&prescription.id).unwrap_err();

        assert_eq!(
            error,
            PrescriptionError::InvalidInput(
                "Dispensed prescriptions cannot be cancelled".to_string()
            )
        );
    }
}
