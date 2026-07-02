use super::models::{Prescription, PrescriptionStatus};
use reqwest::{Client, Response};
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

pub trait PrescriptionRepository: Send + Sync {
    fn create(&self, prescription: Prescription) -> RepositoryFuture<'_, Prescription>;
    fn find_by_id(&self, prescription_id: &str) -> RepositoryFuture<'_, Prescription>;
    fn list(&self) -> RepositoryFuture<'_, Vec<Prescription>>;
    fn list_by_patient(&self, patient_id: &str) -> RepositoryFuture<'_, Vec<Prescription>>;
    fn list_by_doctor(&self, doctor_id: &str) -> RepositoryFuture<'_, Vec<Prescription>>;
    fn update(&self, prescription: Prescription) -> RepositoryFuture<'_, Prescription>;
}

#[derive(Default)]
pub struct InMemoryPrescriptionRepository {
    prescriptions: Mutex<HashMap<String, Prescription>>,
}

impl PrescriptionRepository for InMemoryPrescriptionRepository {
    fn create(&self, prescription: Prescription) -> RepositoryFuture<'_, Prescription> {
        Box::pin(async move {
            let mut prescriptions = self
                .prescriptions
                .lock()
                .map_err(|_| RepositoryError::StorageUnavailable)?;
            prescriptions.insert(prescription.id.clone(), prescription.clone());
            Ok(prescription)
        })
    }

    fn find_by_id(&self, prescription_id: &str) -> RepositoryFuture<'_, Prescription> {
        let prescription_id = prescription_id.to_string();
        Box::pin(async move {
            let prescriptions = self
                .prescriptions
                .lock()
                .map_err(|_| RepositoryError::StorageUnavailable)?;

            prescriptions
                .get(&prescription_id)
                .cloned()
                .ok_or(RepositoryError::NotFound)
        })
    }

    fn list(&self) -> RepositoryFuture<'_, Vec<Prescription>> {
        Box::pin(async move {
            let prescriptions = self
                .prescriptions
                .lock()
                .map_err(|_| RepositoryError::StorageUnavailable)?;

            let mut prescription_list: Vec<Prescription> = prescriptions.values().cloned().collect();
            prescription_list
                .sort_by_key(|prescription| std::cmp::Reverse(prescription.issued_at));
            Ok(prescription_list)
        })
    }

    fn list_by_patient(&self, patient_id: &str) -> RepositoryFuture<'_, Vec<Prescription>> {
        let patient_id = patient_id.to_string();
        Box::pin(async move {
            let prescriptions = self.list().await?;
            Ok(prescriptions
                .into_iter()
                .filter(|prescription| prescription.patient_id == patient_id)
                .collect())
        })
    }

    fn list_by_doctor(&self, doctor_id: &str) -> RepositoryFuture<'_, Vec<Prescription>> {
        let doctor_id = doctor_id.to_string();
        Box::pin(async move {
            let prescriptions = self.list().await?;
            Ok(prescriptions
                .into_iter()
                .filter(|prescription| prescription.doctor_id == doctor_id)
                .collect())
        })
    }

    fn update(&self, prescription: Prescription) -> RepositoryFuture<'_, Prescription> {
        Box::pin(async move {
            let mut prescriptions = self
                .prescriptions
                .lock()
                .map_err(|_| RepositoryError::StorageUnavailable)?;

            if !prescriptions.contains_key(&prescription.id) {
                return Err(RepositoryError::NotFound);
            }

            prescriptions.insert(prescription.id.clone(), prescription.clone());
            Ok(prescription)
        })
    }
}

pub struct SupabasePrescriptionRepository {
    url: String,
    key: String,
    client: Client,
}

impl SupabasePrescriptionRepository {
    pub fn new(url: String, key: String) -> Self {
        Self {
            url: url.trim_end_matches('/').to_string(),
            key,
            client: Client::new(),
        }
    }

    fn request(&self, builder: reqwest::RequestBuilder) -> reqwest::RequestBuilder {
        let builder = builder
            .header("apikey", &self.key)
            .header("Content-Type", "application/json");

        if self.key.starts_with("eyJ") {
            builder.header("Authorization", format!("Bearer {}", self.key))
        } else {
            builder
        }
    }

    fn prescriptions_url(&self, query: &str) -> String {
        format!("{}/rest/v1/prescriptions?{}", self.url, query)
    }

    fn prescription_items_url(&self, query: &str) -> String {
        format!("{}/rest/v1/prescription_items?{}", self.url, query)
    }

    fn medicines_url(&self, query: &str) -> String {
        format!("{}/rest/v1/medicine_inventory?{}", self.url, query)
    }

    async fn decode_rows(response: Response) -> Result<Vec<DatabasePrescription>, RepositoryError> {
        if !response.status().is_success() {
            return Err(Self::response_error(response).await);
        }
        response
            .json::<Vec<DatabasePrescription>>()
            .await
            .map_err(|_| RepositoryError::StorageUnavailable)
    }

    async fn response_error(response: Response) -> RepositoryError {
        let status = response.status();
        let body = response.text().await.unwrap_or_default();
        eprintln!("Supabase prescription error {status}: {body}");
        RepositoryError::StorageUnavailable
    }

    async fn ensure_medicine(&self, prescription: &Prescription) -> Result<String, RepositoryError> {
        let medicine_id = medicine_id_for(&prescription.medication_name, &prescription.dosage);
        let response = self
            .request(self.client.post(self.medicines_url("on_conflict=id")))
            .header("Prefer", "resolution=merge-duplicates,return=minimal")
            .json(&json!({
                "id": medicine_id,
                "name": prescription.medication_name,
                "strength": prescription.dosage,
                "unit_cost": prescription.unit_cost,
                "stock_quantity": 0,
                "reorder_level": 0,
                "status": "Active",
            }))
            .send()
            .await
            .map_err(|_| RepositoryError::StorageUnavailable)?;

        if response.status().is_success() {
            Ok(medicine_id)
        } else {
            Err(Self::response_error(response).await)
        }
    }

    async fn write_item(&self, prescription: &Prescription) -> Result<(), RepositoryError> {
        let medicine_id = self.ensure_medicine(prescription).await?;
        let delete_response = self
            .request(self.client.delete(self.prescription_items_url(&format!(
                "prescription_id=eq.{}",
                urlencoding::encode(&prescription.id)
            ))))
            .send()
            .await
            .map_err(|_| RepositoryError::StorageUnavailable)?;

        if !delete_response.status().is_success() {
            return Err(Self::response_error(delete_response).await);
        }

        let create_response = self
            .request(self.client.post(self.prescription_items_url("select=*")))
            .header("Prefer", "return=minimal")
            .json(&json!({
                "prescription_id": prescription.id,
                "medicine_id": medicine_id,
                "dosage": prescription.dosage,
                "frequency": prescription.frequency,
                "duration": prescription.duration,
                "quantity": prescription.quantity,
                "instructions": prescription.instructions,
            }))
            .send()
            .await
            .map_err(|_| RepositoryError::StorageUnavailable)?;

        if create_response.status().is_success() {
            Ok(())
        } else {
            Err(Self::response_error(create_response).await)
        }
    }
}

impl PrescriptionRepository for SupabasePrescriptionRepository {
    fn create(&self, prescription: Prescription) -> RepositoryFuture<'_, Prescription> {
        Box::pin(async move {
            let response = self
                .request(self.client.post(self.prescriptions_url("select=id")))
                .header("Prefer", "return=minimal")
                .json(&json!({
                    "id": prescription.id,
                    "patient_id": prescription.patient_id,
                    "doctor_id": prescription.doctor_id,
                    "medical_record_id": prescription.medical_record_id,
                    "status": database_status(&prescription.status),
                    "prescribed_at": prescription.issued_at,
                    "dispensed_at": prescription.dispensed_at,
                    "notes": prescription.instructions,
                }))
                .send()
                .await
                .map_err(|_| RepositoryError::StorageUnavailable)?;

            if !response.status().is_success() {
                return Err(Self::response_error(response).await);
            }

            self.write_item(&prescription).await?;
            self.find_by_id(&prescription.id).await
        })
    }

    fn find_by_id(&self, prescription_id: &str) -> RepositoryFuture<'_, Prescription> {
        let prescription_id = prescription_id.to_string();
        Box::pin(async move {
            let encoded_id = urlencoding::encode(&prescription_id);
            let query = format!(
                "select=*,prescription_items(*,medicine:medicine_id(id,name,strength,unit_cost))&id=eq.{}&limit=1",
                encoded_id
            );
            let response = self
                .request(self.client.get(self.prescriptions_url(&query)))
                .send()
                .await
                .map_err(|_| RepositoryError::StorageUnavailable)?;
            let mut rows = Self::decode_rows(response).await?;
            rows.pop()
                .map(Prescription::from)
                .ok_or(RepositoryError::NotFound)
        })
    }

    fn list(&self) -> RepositoryFuture<'_, Vec<Prescription>> {
        Box::pin(async move {
            let response = self
                .request(self.client.get(self.prescriptions_url(
                    "select=*,prescription_items(*,medicine:medicine_id(id,name,strength,unit_cost))&order=prescribed_at.desc",
                )))
                .send()
                .await
                .map_err(|_| RepositoryError::StorageUnavailable)?;
            Ok(Self::decode_rows(response)
                .await?
                .into_iter()
                .map(Prescription::from)
                .collect())
        })
    }

    fn list_by_patient(&self, patient_id: &str) -> RepositoryFuture<'_, Vec<Prescription>> {
        let patient_id = patient_id.to_string();
        Box::pin(async move {
            let encoded_id = urlencoding::encode(&patient_id);
            let query = format!(
                "select=*,prescription_items(*,medicine:medicine_id(id,name,strength,unit_cost))&patient_id=eq.{}&order=prescribed_at.desc",
                encoded_id
            );
            let response = self
                .request(self.client.get(self.prescriptions_url(&query)))
                .send()
                .await
                .map_err(|_| RepositoryError::StorageUnavailable)?;
            Ok(Self::decode_rows(response)
                .await?
                .into_iter()
                .map(Prescription::from)
                .collect())
        })
    }

    fn list_by_doctor(&self, doctor_id: &str) -> RepositoryFuture<'_, Vec<Prescription>> {
        let doctor_id = doctor_id.to_string();
        Box::pin(async move {
            let encoded_id = urlencoding::encode(&doctor_id);
            let query = format!(
                "select=*,prescription_items(*,medicine:medicine_id(id,name,strength,unit_cost))&doctor_id=eq.{}&order=prescribed_at.desc",
                encoded_id
            );
            let response = self
                .request(self.client.get(self.prescriptions_url(&query)))
                .send()
                .await
                .map_err(|_| RepositoryError::StorageUnavailable)?;
            Ok(Self::decode_rows(response)
                .await?
                .into_iter()
                .map(Prescription::from)
                .collect())
        })
    }

    fn update(&self, prescription: Prescription) -> RepositoryFuture<'_, Prescription> {
        Box::pin(async move {
            let encoded_id = urlencoding::encode(&prescription.id);
            let response = self
                .request(self.client.patch(self.prescriptions_url(&format!("id=eq.{encoded_id}"))))
                .header("Prefer", "return=minimal")
                .json(&json!({
                    "patient_id": prescription.patient_id,
                    "doctor_id": prescription.doctor_id,
                    "medical_record_id": prescription.medical_record_id,
                    "status": database_status(&prescription.status),
                    "prescribed_at": prescription.issued_at,
                    "dispensed_at": prescription.dispensed_at,
                    "notes": prescription.instructions,
                }))
                .send()
                .await
                .map_err(|_| RepositoryError::StorageUnavailable)?;

            if !response.status().is_success() {
                return Err(Self::response_error(response).await);
            }

            self.write_item(&prescription).await?;
            self.find_by_id(&prescription.id).await
        })
    }
}

#[derive(Deserialize)]
struct DatabasePrescription {
    id: String,
    patient_id: String,
    doctor_id: Option<String>,
    medical_record_id: Option<String>,
    status: DatabasePrescriptionStatus,
    prescribed_at: chrono::DateTime<chrono::Utc>,
    dispensed_at: Option<chrono::DateTime<chrono::Utc>>,
    notes: Option<String>,
    #[serde(default)]
    prescription_items: Vec<DatabasePrescriptionItem>,
}

#[derive(Deserialize)]
struct DatabasePrescriptionItem {
    dosage: String,
    frequency: String,
    duration: String,
    quantity: u32,
    instructions: Option<String>,
    medicine: Option<DatabaseMedicine>,
}

#[derive(Deserialize)]
struct DatabaseMedicine {
    name: String,
    strength: Option<String>,
    unit_cost: f64,
}

#[derive(Deserialize)]
enum DatabasePrescriptionStatus {
    Prescribed,
    Dispensed,
    Cancelled,
}

impl From<DatabasePrescription> for Prescription {
    fn from(row: DatabasePrescription) -> Self {
        let item = row.prescription_items.into_iter().next();
        let medicine = item.as_ref().and_then(|item| item.medicine.as_ref());
        Self {
            id: row.id,
            patient_id: row.patient_id,
            doctor_id: row.doctor_id.unwrap_or_default(),
            medical_record_id: row.medical_record_id,
            medication_name: medicine
                .map(|medicine| medicine.name.clone())
                .unwrap_or_else(|| "Unknown medicine".to_string()),
            dosage: item
                .as_ref()
                .map(|item| item.dosage.clone())
                .or_else(|| medicine.and_then(|medicine| medicine.strength.clone()))
                .unwrap_or_default(),
            frequency: item
                .as_ref()
                .map(|item| item.frequency.clone())
                .unwrap_or_default(),
            duration: item
                .as_ref()
                .map(|item| item.duration.clone())
                .unwrap_or_default(),
            instructions: item
                .as_ref()
                .and_then(|item| item.instructions.clone())
                .or(row.notes),
            quantity: item.as_ref().map(|item| item.quantity).unwrap_or(0),
            unit_cost: medicine.map(|medicine| medicine.unit_cost).unwrap_or(0.0),
            status: match row.status {
                DatabasePrescriptionStatus::Prescribed => PrescriptionStatus::Active,
                DatabasePrescriptionStatus::Dispensed => PrescriptionStatus::Dispensed,
                DatabasePrescriptionStatus::Cancelled => PrescriptionStatus::Cancelled,
            },
            issued_at: row.prescribed_at,
            dispensed_at: row.dispensed_at,
            cancelled_at: None,
        }
    }
}

fn database_status(status: &PrescriptionStatus) -> &'static str {
    match status {
        PrescriptionStatus::Active => "Prescribed",
        PrescriptionStatus::Dispensed => "Dispensed",
        PrescriptionStatus::Cancelled => "Cancelled",
    }
}

fn medicine_id_for(name: &str, dosage: &str) -> String {
    let key = format!("{} {}", name.trim(), dosage.trim());
    let slug: String = key
        .chars()
        .map(|character| {
            if character.is_ascii_alphanumeric() {
                character.to_ascii_uppercase()
            } else {
                '-'
            }
        })
        .collect::<String>()
        .split('-')
        .filter(|part| !part.is_empty())
        .collect::<Vec<_>>()
        .join("-");

    format!("MED-{}", if slug.is_empty() { "UNKNOWN" } else { &slug })
}
