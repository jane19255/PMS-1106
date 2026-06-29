use super::models::{MedicalRecord, PatientTimelineEntry};
use super::repository::{MedicalRecordRepository, RepositoryError};
use chrono::Utc;
use serde::Deserialize;
use std::sync::Arc;
use uuid::Uuid;

#[derive(Debug, PartialEq)]
pub enum MedicalRecordError {
    InvalidInput(String),
    RecordNotFound,
    StorageUnavailable,
}

#[derive(Clone, Debug, Deserialize)]
pub struct CreateMedicalRecordForm {
    pub patient_id: String,
    pub appointment_id: Option<String>,
    pub doctor_id: Option<String>,
    pub reason_of_visit: Option<String>,
    pub clinical_findings: Option<String>,
    pub diagnosis: Option<String>,
    pub treatment_plan: Option<String>,
    pub doctor_notes: Option<String>,
    pub blood_pressure: Option<String>,
    pub temperature: Option<String>,
    pub pulse_rate: Option<String>,
    pub height_cm: Option<String>,
    pub weight_kg: Option<String>,
}

#[derive(Clone, Debug, Deserialize)]
pub struct UpdateMedicalRecordForm {
    pub doctor_id: Option<String>,
    pub reason_of_visit: Option<String>,
    pub clinical_findings: Option<String>,
    pub diagnosis: Option<String>,
    pub treatment_plan: Option<String>,
    pub doctor_notes: Option<String>,
    pub blood_pressure: Option<String>,
    pub temperature: Option<String>,
    pub pulse_rate: Option<String>,
    pub height_cm: Option<String>,
    pub weight_kg: Option<String>,
}

pub struct MedicalRecordService {
    repository: Arc<dyn MedicalRecordRepository>,
}

impl MedicalRecordService {
    pub fn new(repository: Arc<dyn MedicalRecordRepository>) -> Self {
        Self { repository }
    }

    pub async fn create_record(
        &self,
        form: CreateMedicalRecordForm,
    ) -> Result<MedicalRecord, MedicalRecordError> {
        let patient_id = form.patient_id.trim().to_string();
        if patient_id.is_empty() {
            return Err(MedicalRecordError::InvalidInput(
                "Patient ID is required".to_string(),
            ));
        }

        let record = MedicalRecord {
            id: format!("MR-{}", Uuid::new_v4()),
            patient_id,
            appointment_id: non_empty_opt(form.appointment_id),
            doctor_id: non_empty_opt(form.doctor_id),
            reason_of_visit: non_empty_opt(form.reason_of_visit),
            clinical_findings: non_empty_opt(form.clinical_findings),
            diagnosis: non_empty_opt(form.diagnosis),
            treatment_plan: non_empty_opt(form.treatment_plan),
            doctor_notes: non_empty_opt(form.doctor_notes),
            blood_pressure: non_empty_opt(form.blood_pressure),
            temperature: non_empty_opt(form.temperature),
            pulse_rate: non_empty_opt(form.pulse_rate),
            height_cm: non_empty_opt(form.height_cm),
            weight_kg: non_empty_opt(form.weight_kg),
            recorded_at: Utc::now(),
            created_at: Utc::now(),
            updated_at: Utc::now(),
        };

        self.repository
            .create(record)
            .await
            .map_err(Self::map_repo_error)
    }

    pub async fn find_record(
        &self,
        record_id: &str,
    ) -> Result<MedicalRecord, MedicalRecordError> {
        if record_id.trim().is_empty() {
            return Err(MedicalRecordError::InvalidInput(
                "Record ID is required".to_string(),
            ));
        }
        self.repository
            .find_by_id(record_id.trim())
            .await
            .map_err(Self::map_repo_error)
    }

    pub async fn list_records(&self) -> Result<Vec<MedicalRecord>, MedicalRecordError> {
        self.repository
            .list()
            .await
            .map_err(Self::map_repo_error)
    }

    pub async fn list_records_for_patient(
        &self,
        patient_id: &str,
    ) -> Result<Vec<MedicalRecord>, MedicalRecordError> {
        if patient_id.trim().is_empty() {
            return Err(MedicalRecordError::InvalidInput(
                "Patient ID is required".to_string(),
            ));
        }
        self.repository
            .list_by_patient(patient_id.trim())
            .await
            .map_err(Self::map_repo_error)
    }

    pub async fn patient_timeline(
        &self,
        patient_id: &str,
    ) -> Result<Vec<PatientTimelineEntry>, MedicalRecordError> {
        let records = self.list_records_for_patient(patient_id).await?;
        let entries = records
            .into_iter()
            .map(|record| PatientTimelineEntry {
                record,
                doctor_name: None,
            })
            .collect();
        Ok(entries)
    }

    pub async fn update_record(
        &self,
        record_id: &str,
        form: UpdateMedicalRecordForm,
    ) -> Result<MedicalRecord, MedicalRecordError> {
        let mut record = self.find_record(record_id).await?;

        record.doctor_id = non_empty_opt(form.doctor_id).or(record.doctor_id);
        record.reason_of_visit = non_empty_opt(form.reason_of_visit);
        record.clinical_findings = non_empty_opt(form.clinical_findings);
        record.diagnosis = non_empty_opt(form.diagnosis);
        record.treatment_plan = non_empty_opt(form.treatment_plan);
        record.doctor_notes = non_empty_opt(form.doctor_notes);
        record.blood_pressure = non_empty_opt(form.blood_pressure);
        record.temperature = non_empty_opt(form.temperature);
        record.pulse_rate = non_empty_opt(form.pulse_rate);
        record.height_cm = non_empty_opt(form.height_cm);
        record.weight_kg = non_empty_opt(form.weight_kg);
        record.updated_at = Utc::now();

        self.repository
            .update(record)
            .await
            .map_err(Self::map_repo_error)
    }

    pub async fn delete_record(&self, record_id: &str) -> Result<(), MedicalRecordError> {
        if record_id.trim().is_empty() {
            return Err(MedicalRecordError::InvalidInput(
                "Record ID is required".to_string(),
            ));
        }
        self.repository
            .delete(record_id.trim())
            .await
            .map_err(Self::map_repo_error)
    }

    fn map_repo_error(error: RepositoryError) -> MedicalRecordError {
        match error {
            RepositoryError::NotFound => MedicalRecordError::RecordNotFound,
            RepositoryError::StorageUnavailable => MedicalRecordError::StorageUnavailable,
        }
    }
}

fn non_empty_opt(value: Option<String>) -> Option<String> {
    value
        .map(|s| s.trim().to_string())
        .filter(|s| !s.is_empty())
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::medical_records::repository::InMemoryMedicalRecordRepository;

    fn service() -> MedicalRecordService {
        MedicalRecordService::new(Arc::new(InMemoryMedicalRecordRepository::default()))
    }

    fn create_form() -> CreateMedicalRecordForm {
        CreateMedicalRecordForm {
            patient_id: "PAT-001".to_string(),
            appointment_id: Some("APT-001".to_string()),
            doctor_id: Some("DOC-001".to_string()),
            reason_of_visit: Some("Headache".to_string()),
            clinical_findings: Some("Mild tension headache".to_string()),
            diagnosis: Some("Tension-type headache".to_string()),
            treatment_plan: Some("Rest and analgesics".to_string()),
            doctor_notes: Some("Follow up in 2 weeks if no improvement".to_string()),
            blood_pressure: Some("120/80".to_string()),
            temperature: Some("36.8".to_string()),
            pulse_rate: Some("72".to_string()),
            height_cm: Some("170".to_string()),
            weight_kg: Some("65".to_string()),
        }
    }

    #[actix_web::test]
    async fn creates_record_with_generated_id() {
        let record = service().create_record(create_form()).await.unwrap();
        assert!(record.id.starts_with("MR-"));
        assert_eq!(record.patient_id, "PAT-001");
        assert_eq!(record.diagnosis.as_deref(), Some("Tension-type headache"));
    }

    #[actix_web::test]
    async fn rejects_missing_patient_id() {
        let mut form = create_form();
        form.patient_id = " ".to_string();
        let error = service().create_record(form).await.unwrap_err();
        assert_eq!(
            error,
            MedicalRecordError::InvalidInput("Patient ID is required".to_string())
        );
    }

    #[actix_web::test]
    async fn lists_records_by_patient() {
        let service = service();
        service.create_record(create_form()).await.unwrap();
        let mut form2 = create_form();
        form2.patient_id = "PAT-002".to_string();
        service.create_record(form2).await.unwrap();

        let records = service
            .list_records_for_patient("PAT-001")
            .await
            .unwrap();
        assert_eq!(records.len(), 1);
        assert_eq!(records[0].patient_id, "PAT-001");
    }

    #[actix_web::test]
    async fn updates_record_fields() {
        let service = service();
        let record = service.create_record(create_form()).await.unwrap();

        let update = UpdateMedicalRecordForm {
            doctor_id: None,
            reason_of_visit: Some("Fever".to_string()),
            clinical_findings: Some("High temperature".to_string()),
            diagnosis: Some("Viral fever".to_string()),
            treatment_plan: Some("Antipyretics".to_string()),
            doctor_notes: None,
            blood_pressure: Some("118/76".to_string()),
            temperature: Some("38.5".to_string()),
            pulse_rate: Some("88".to_string()),
            height_cm: None,
            weight_kg: None,
        };

        let updated = service.update_record(&record.id, update).await.unwrap();
        assert_eq!(updated.diagnosis.as_deref(), Some("Viral fever"));
        assert_eq!(updated.temperature.as_deref(), Some("38.5"));
    }

    #[actix_web::test]
    async fn deletes_record() {
        let service = service();
        let record = service.create_record(create_form()).await.unwrap();
        service.delete_record(&record.id).await.unwrap();

        let error = service.find_record(&record.id).await.unwrap_err();
        assert_eq!(error, MedicalRecordError::RecordNotFound);
    }
}
