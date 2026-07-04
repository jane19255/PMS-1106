//! Medical record repository implementations.
//!
//! Provides medical record persistence and patient timeline loading through in-memory
//! and Supabase-backed repositories.
use super::models::MedicalRecord;
use reqwest::Client;
use serde::Deserialize;
use serde_json::json;
use std::collections::HashMap;
use std::future::Future;
use std::pin::Pin;
use std::sync::Mutex;

pub type RepositoryFuture<'a, T> =
    Pin<Box<dyn Future<Output = Result<T, RepositoryError>> + Send + 'a>>;

#[derive(Debug)]
pub enum RepositoryError {
    NotFound,
    StorageUnavailable,
}

pub trait MedicalRecordRepository: Send + Sync {
    fn create(&self, record: MedicalRecord) -> RepositoryFuture<'_, MedicalRecord>;
    fn find_by_id(&self, record_id: &str) -> RepositoryFuture<'_, MedicalRecord>;
    fn list(&self) -> RepositoryFuture<'_, Vec<MedicalRecord>>;
    fn list_by_patient(&self, patient_id: &str) -> RepositoryFuture<'_, Vec<MedicalRecord>>;
    fn update(&self, record: MedicalRecord) -> RepositoryFuture<'_, MedicalRecord>;
    fn delete(&self, record_id: &str) -> RepositoryFuture<'_, ()>;
}

// ── In-memory implementation (default fallback) ─────────────────────────────

#[derive(Default)]
pub struct InMemoryMedicalRecordRepository {
    records: Mutex<HashMap<String, MedicalRecord>>,
}

impl MedicalRecordRepository for InMemoryMedicalRecordRepository {
    fn create(&self, record: MedicalRecord) -> RepositoryFuture<'_, MedicalRecord> {
        Box::pin(async move {
            let mut records = self
                .records
                .lock()
                .map_err(|_| RepositoryError::StorageUnavailable)?;
            records.insert(record.id.clone(), record.clone());
            Ok(record)
        })
    }

    fn find_by_id(&self, record_id: &str) -> RepositoryFuture<'_, MedicalRecord> {
        let record_id = record_id.to_string();
        Box::pin(async move {
            let records = self
                .records
                .lock()
                .map_err(|_| RepositoryError::StorageUnavailable)?;
            records
                .get(&record_id)
                .cloned()
                .ok_or(RepositoryError::NotFound)
        })
    }

    fn list(&self) -> RepositoryFuture<'_, Vec<MedicalRecord>> {
        Box::pin(async move {
            let records = self
                .records
                .lock()
                .map_err(|_| RepositoryError::StorageUnavailable)?;
            let mut list: Vec<MedicalRecord> = records.values().cloned().collect();
            list.sort_by_key(|record| std::cmp::Reverse(record.recorded_at));
            Ok(list)
        })
    }

    fn list_by_patient(&self, patient_id: &str) -> RepositoryFuture<'_, Vec<MedicalRecord>> {
        let patient_id = patient_id.to_string();
        Box::pin(async move {
            let all = self.list().await?;
            Ok(all
                .into_iter()
                .filter(|r| r.patient_id == patient_id)
                .collect())
        })
    }

    fn update(&self, record: MedicalRecord) -> RepositoryFuture<'_, MedicalRecord> {
        Box::pin(async move {
            let mut records = self
                .records
                .lock()
                .map_err(|_| RepositoryError::StorageUnavailable)?;
            if !records.contains_key(&record.id) {
                return Err(RepositoryError::NotFound);
            }
            records.insert(record.id.clone(), record.clone());
            Ok(record)
        })
    }

    fn delete(&self, record_id: &str) -> RepositoryFuture<'_, ()> {
        let record_id = record_id.to_string();
        Box::pin(async move {
            let mut records = self
                .records
                .lock()
                .map_err(|_| RepositoryError::StorageUnavailable)?;
            records
                .remove(&record_id)
                .map(|_| ())
                .ok_or(RepositoryError::NotFound)
        })
    }
}

// ── Supabase implementation ──────────────────────────────────────────────────

pub struct SupabaseMedicalRecordRepository {
    url: String,
    key: String,
    client: Client,
}

impl SupabaseMedicalRecordRepository {
    pub fn new(url: String, key: String) -> Self {
        Self {
            url: url.trim_end_matches('/').to_string(),
            key,
            client: Client::new(),
        }
    }

    fn request(&self, builder: reqwest::RequestBuilder) -> reqwest::RequestBuilder {
        builder
            .header("apikey", &self.key)
            .header("Authorization", format!("Bearer {}", self.key))
            .header("Content-Type", "application/json")
    }

    fn records_url(&self, query: &str) -> String {
        format!("{}/rest/v1/medical_records?{}", self.url, query)
    }

    async fn decode_rows(
        response: reqwest::Response,
    ) -> Result<Vec<DatabaseMedicalRecord>, RepositoryError> {
        if !response.status().is_success() {
            let status = response.status();
            let body = response.text().await.unwrap_or_default();
            eprintln!("Supabase medical records error {status}: {body}");
            return Err(RepositoryError::StorageUnavailable);
        }
        response
            .json::<Vec<DatabaseMedicalRecord>>()
            .await
            .map_err(|e| {
                eprintln!("Medical records JSON parse error: {e}");
                RepositoryError::StorageUnavailable
            })
    }
}

impl MedicalRecordRepository for SupabaseMedicalRecordRepository {
    fn create(&self, record: MedicalRecord) -> RepositoryFuture<'_, MedicalRecord> {
        Box::pin(async move {
            let response = self
                .request(self.client.post(self.records_url("select=id")))
                .header("Prefer", "return=minimal")
                .json(&json!({
                    "id": record.id,
                    "patient_id": record.patient_id,
                    "appointment_id": record.appointment_id,
                    "doctor_id": record.doctor_id,
                    "reason_of_visit": record.reason_of_visit,
                    "clinical_findings": record.clinical_findings,
                    "diagnosis": record.diagnosis,
                    "treatment_plan": record.treatment_plan,
                    "doctor_notes": record.doctor_notes,
                    "blood_pressure": record.blood_pressure,
                    "temperature": record.temperature,
                    "pulse_rate": record.pulse_rate,
                    "height_cm": record.height_cm,
                    "weight_kg": record.weight_kg,
                    "recorded_at": record.recorded_at,
                }))
                .send()
                .await
                .map_err(|_| RepositoryError::StorageUnavailable)?;

            if !response.status().is_success() {
                let status = response.status();
                let body = response.text().await.unwrap_or_default();
                eprintln!("Supabase create medical record error {status}: {body}");
                return Err(RepositoryError::StorageUnavailable);
            }

            self.find_by_id(&record.id).await
        })
    }

    fn find_by_id(&self, record_id: &str) -> RepositoryFuture<'_, MedicalRecord> {
        let record_id = record_id.to_string();
        Box::pin(async move {
            let encoded_id = urlencoding::encode(&record_id);
            let query = format!("id=eq.{}&select=*&limit=1", encoded_id);
            let response = self
                .request(self.client.get(self.records_url(&query)))
                .send()
                .await
                .map_err(|_| RepositoryError::StorageUnavailable)?;
            let mut rows = Self::decode_rows(response).await?;
            rows.pop()
                .map(MedicalRecord::from)
                .ok_or(RepositoryError::NotFound)
        })
    }

    fn list(&self) -> RepositoryFuture<'_, Vec<MedicalRecord>> {
        Box::pin(async move {
            let response = self
                .request(
                    self.client
                        .get(self.records_url("select=*&order=recorded_at.desc")),
                )
                .send()
                .await
                .map_err(|_| RepositoryError::StorageUnavailable)?;
            Ok(Self::decode_rows(response)
                .await?
                .into_iter()
                .map(MedicalRecord::from)
                .collect())
        })
    }

    fn list_by_patient(&self, patient_id: &str) -> RepositoryFuture<'_, Vec<MedicalRecord>> {
        let patient_id = patient_id.to_string();
        Box::pin(async move {
            let encoded_id = urlencoding::encode(&patient_id);
            let query = format!(
                "patient_id=eq.{}&select=*&order=recorded_at.desc",
                encoded_id
            );
            let response = self
                .request(self.client.get(self.records_url(&query)))
                .send()
                .await
                .map_err(|_| RepositoryError::StorageUnavailable)?;
            Ok(Self::decode_rows(response)
                .await?
                .into_iter()
                .map(MedicalRecord::from)
                .collect())
        })
    }

    fn update(&self, record: MedicalRecord) -> RepositoryFuture<'_, MedicalRecord> {
        Box::pin(async move {
            let encoded_id = urlencoding::encode(&record.id);
            let response = self
                .request(
                    self.client
                        .patch(self.records_url(&format!("id=eq.{encoded_id}"))),
                )
                .header("Prefer", "return=minimal")
                .json(&json!({
                    "reason_of_visit": record.reason_of_visit,
                    "clinical_findings": record.clinical_findings,
                    "diagnosis": record.diagnosis,
                    "treatment_plan": record.treatment_plan,
                    "doctor_notes": record.doctor_notes,
                    "blood_pressure": record.blood_pressure,
                    "temperature": record.temperature,
                    "pulse_rate": record.pulse_rate,
                    "height_cm": record.height_cm,
                    "weight_kg": record.weight_kg,
                    "doctor_id": record.doctor_id,
                }))
                .send()
                .await
                .map_err(|_| RepositoryError::StorageUnavailable)?;

            if !response.status().is_success() {
                let status = response.status();
                let body = response.text().await.unwrap_or_default();
                eprintln!("Supabase update medical record error {status}: {body}");
                return Err(RepositoryError::StorageUnavailable);
            }

            self.find_by_id(&record.id).await
        })
    }

    fn delete(&self, record_id: &str) -> RepositoryFuture<'_, ()> {
        let record_id = record_id.to_string();
        Box::pin(async move {
            let encoded_id = urlencoding::encode(&record_id);
            let response = self
                .request(
                    self.client
                        .delete(self.records_url(&format!("id=eq.{encoded_id}"))),
                )
                .send()
                .await
                .map_err(|_| RepositoryError::StorageUnavailable)?;

            if response.status().is_success() {
                Ok(())
            } else {
                let status = response.status();
                let body = response.text().await.unwrap_or_default();
                eprintln!("Supabase delete medical record error {status}: {body}");
                Err(RepositoryError::StorageUnavailable)
            }
        })
    }
}

// ── Database row types ───────────────────────────────────────────────────────

#[derive(Deserialize)]
struct DatabaseMedicalRecord {
    id: String,
    patient_id: String,
    appointment_id: Option<String>,
    doctor_id: Option<String>,
    reason_of_visit: Option<String>,
    clinical_findings: Option<String>,
    diagnosis: Option<String>,
    treatment_plan: Option<String>,
    doctor_notes: Option<String>,
    blood_pressure: Option<String>,
    temperature: Option<String>,
    pulse_rate: Option<String>,
    height_cm: Option<String>,
    weight_kg: Option<String>,
    recorded_at: chrono::DateTime<chrono::Utc>,
    created_at: chrono::DateTime<chrono::Utc>,
    updated_at: chrono::DateTime<chrono::Utc>,
}

impl From<DatabaseMedicalRecord> for MedicalRecord {
    fn from(row: DatabaseMedicalRecord) -> Self {
        Self {
            id: row.id,
            patient_id: row.patient_id,
            appointment_id: row.appointment_id,
            doctor_id: row.doctor_id,
            reason_of_visit: row.reason_of_visit,
            clinical_findings: row.clinical_findings,
            diagnosis: row.diagnosis,
            treatment_plan: row.treatment_plan,
            doctor_notes: row.doctor_notes,
            blood_pressure: row.blood_pressure,
            temperature: row.temperature,
            pulse_rate: row.pulse_rate,
            height_cm: row.height_cm,
            weight_kg: row.weight_kg,
            recorded_at: row.recorded_at,
            created_at: row.created_at,
            updated_at: row.updated_at,
        }
    }
}
