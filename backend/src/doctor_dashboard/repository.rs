use super::models::DoctorQueueAppointment;
use crate::db::singapore_today;
use chrono::{DateTime, NaiveDate, Timelike, Utc};
use reqwest::{Client, Response};
use serde::Deserialize;
use serde_json::{json, Value};
use std::future::Future;
use std::pin::Pin;

pub type RepositoryFuture<'a, T> =
    Pin<Box<dyn Future<Output = Result<T, RepositoryError>> + Send + 'a>>;

#[derive(Debug)]
pub enum RepositoryError {
    StorageUnavailable,
    Rejected(String),
}

pub trait DoctorDashboardRepository: Send + Sync {
    fn list_appointments(
        &self,
        firebase_uid: &str,
        date: NaiveDate,
    ) -> RepositoryFuture<'_, Vec<DoctorQueueAppointment>>;

    fn start_consultation(
        &self,
        firebase_uid: &str,
        appointment_id: &str,
    ) -> RepositoryFuture<'_, ()>;

    fn complete_consultation(
        &self,
        firebase_uid: &str,
        appointment_id: &str,
    ) -> RepositoryFuture<'_, ()>;
}

#[derive(Default)]
pub struct InMemoryDoctorDashboardRepository;

impl DoctorDashboardRepository for InMemoryDoctorDashboardRepository {
    fn list_appointments(
        &self,
        _firebase_uid: &str,
        _date: NaiveDate,
    ) -> RepositoryFuture<'_, Vec<DoctorQueueAppointment>> {
        Box::pin(async { Ok(Vec::new()) })
    }

    fn start_consultation(
        &self,
        _firebase_uid: &str,
        _appointment_id: &str,
    ) -> RepositoryFuture<'_, ()> {
        Box::pin(async { Ok(()) })
    }

    fn complete_consultation(
        &self,
        _firebase_uid: &str,
        _appointment_id: &str,
    ) -> RepositoryFuture<'_, ()> {
        Box::pin(async { Ok(()) })
    }
}

pub struct SupabaseDoctorDashboardRepository {
    url: String,
    key: String,
    client: Client,
}

impl SupabaseDoctorDashboardRepository {
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

    fn staff_doctor_url(&self, firebase_uid: &str) -> String {
        format!(
            "{}/rest/v1/doctors?select=id,staff!inner(firebase_uid)&staff.firebase_uid=eq.{}&limit=1",
            self.url,
            urlencoding::encode(firebase_uid)
        )
    }

    fn appointments_url(&self, doctor_id: &str, date: NaiveDate) -> String {
        let next_day = date.succ_opt().unwrap_or(date);
        let select = "id,patient_id,scheduled_at,patients(id,first_name,last_name),patient_queue(status,priority)";
        // Today's list also includes older patients whose visit is still unfinished.
        let date_filter = if date == singapore_today() {
            format!(
                "or=(and(scheduled_at.gte.{date}T00:00:00Z,scheduled_at.lt.{next_day}T00:00:00Z),and(scheduled_at.lt.{date}T00:00:00Z,status.in.(Checked%20In,Vitals%20Recorded,In%20Consultation)))"
            )
        } else {
            format!(
                "scheduled_at=gte.{date}T00:00:00Z&scheduled_at=lt.{next_day}T00:00:00Z"
            )
        };

        format!(
            "{}/rest/v1/appointments?select={}&doctor_id=eq.{}&{}&order=scheduled_at.asc",
            self.url,
            select,
            urlencoding::encode(doctor_id),
            date_filter
        )
    }

    async fn decode_doctor_id(response: Response) -> Result<String, RepositoryError> {
        if !response.status().is_success() {
            return Err(Self::response_error(response).await);
        }

        let rows = response
            .json::<Vec<DoctorIdRow>>()
            .await
            .map_err(|_| RepositoryError::StorageUnavailable)?;

        rows.first().map(|row| row.id.clone()).ok_or_else(|| {
            RepositoryError::Rejected("No doctor profile is linked to this account".to_string())
        })
    }

    async fn decode_appointments(
        response: Response,
    ) -> Result<Vec<DatabaseAppointment>, RepositoryError> {
        if !response.status().is_success() {
            return Err(Self::response_error(response).await);
        }

        let body = response
            .text()
            .await
            .map_err(|_| RepositoryError::StorageUnavailable)?;

        serde_json::from_str::<Vec<DatabaseAppointment>>(&body).map_err(|error| {
            eprintln!("Doctor dashboard decode error: {error}");
            eprintln!("Doctor dashboard response body: {body}");
            RepositoryError::StorageUnavailable
        })
    }

    async fn ensure_success(response: Response) -> Result<(), RepositoryError> {
        if response.status().is_success() {
            Ok(())
        } else {
            Err(Self::response_error(response).await)
        }
    }

    async fn reconcile_overdue(&self, today: NaiveDate) -> Result<(), RepositoryError> {
        // This keeps no-show statuses correct even if the doctor opens the system first.
        let response = self
            .request(
                self.client.post(format!(
                    "{}/rest/v1/rpc/reconcile_overdue_appointments",
                    self.url
                )),
            )
            .json(&json!({ "p_today": today.format("%Y-%m-%d").to_string() }))
            .send()
            .await
            .map_err(|_| RepositoryError::StorageUnavailable)?;
        Self::ensure_success(response).await
    }

    async fn response_error(response: Response) -> RepositoryError {
        let status = response.status();
        let body = response.text().await.unwrap_or_default();
        eprintln!("Supabase doctor dashboard error {status}: {body}");

        // Postgres RPC business-rule rejections (raise exception) surface as a
        // 4xx PostgREST error body with a "message" field; a transport-level
        // failure has no parseable body at all, so fall back to StorageUnavailable.
        match serde_json::from_str::<Value>(&body) {
            Ok(json) => match json.get("message").and_then(Value::as_str) {
                Some(message) => RepositoryError::Rejected(message.to_string()),
                None => RepositoryError::StorageUnavailable,
            },
            Err(_) => RepositoryError::StorageUnavailable,
        }
    }
}

impl DoctorDashboardRepository for SupabaseDoctorDashboardRepository {
    fn list_appointments(
        &self,
        firebase_uid: &str,
        date: NaiveDate,
    ) -> RepositoryFuture<'_, Vec<DoctorQueueAppointment>> {
        let firebase_uid = firebase_uid.to_string();

        Box::pin(async move {
            if date == singapore_today() {
                self.reconcile_overdue(date).await?;
            }
            let doctor_response = self
                .request(self.client.get(self.staff_doctor_url(&firebase_uid)))
                .send()
                .await
                .map_err(|_| RepositoryError::StorageUnavailable)?;

            let doctor_id = Self::decode_doctor_id(doctor_response).await?;

            let appointment_response = self
                .request(self.client.get(self.appointments_url(&doctor_id, date)))
                .send()
                .await
                .map_err(|_| RepositoryError::StorageUnavailable)?;

            let rows = Self::decode_appointments(appointment_response).await?;

            Ok(rows.into_iter().map(DoctorQueueAppointment::from).collect())
        })
    }

    fn start_consultation(
        &self,
        firebase_uid: &str,
        appointment_id: &str,
    ) -> RepositoryFuture<'_, ()> {
        let firebase_uid = firebase_uid.to_string();
        let appointment_id = appointment_id.to_string();

        Box::pin(async move {
            let doctor_response = self
                .request(self.client.get(self.staff_doctor_url(&firebase_uid)))
                .send()
                .await
                .map_err(|_| RepositoryError::StorageUnavailable)?;
            let doctor_id = Self::decode_doctor_id(doctor_response).await?;

            let payload = json!({
                "p_appointment_id": appointment_id,
                "p_doctor_id": doctor_id,
            });
            let response = self
                .request(
                    self.client
                        .post(format!("{}/rest/v1/rpc/start_consultation", self.url)),
                )
                .json(&payload)
                .send()
                .await
                .map_err(|_| RepositoryError::StorageUnavailable)?;

            Self::ensure_success(response).await
        })
    }

    fn complete_consultation(
        &self,
        firebase_uid: &str,
        appointment_id: &str,
    ) -> RepositoryFuture<'_, ()> {
        let firebase_uid = firebase_uid.to_string();
        let appointment_id = appointment_id.to_string();

        Box::pin(async move {
            let doctor_response = self
                .request(self.client.get(self.staff_doctor_url(&firebase_uid)))
                .send()
                .await
                .map_err(|_| RepositoryError::StorageUnavailable)?;
            let doctor_id = Self::decode_doctor_id(doctor_response).await?;

            let payload = json!({
                "p_appointment_id": appointment_id,
                "p_doctor_id": doctor_id,
            });
            let response = self
                .request(
                    self.client
                        .post(format!("{}/rest/v1/rpc/complete_consultation", self.url)),
                )
                .json(&payload)
                .send()
                .await
                .map_err(|_| RepositoryError::StorageUnavailable)?;

            Self::ensure_success(response).await
        })
    }
}

#[derive(Deserialize)]
struct DoctorIdRow {
    id: String,
}

#[derive(Deserialize)]
struct DatabaseAppointment {
    id: String,
    patient_id: String,
    scheduled_at: DateTime<Utc>,
    patients: DatabasePatient,
    patient_queue: Option<Value>,
}

#[derive(Deserialize)]
struct DatabasePatient {
    first_name: String,
    last_name: String,
}

impl From<DatabaseAppointment> for DoctorQueueAppointment {
    fn from(row: DatabaseAppointment) -> Self {
        let queue_status = queue_status_from_value(row.patient_queue.as_ref());
        let priority = queue_priority_from_value(row.patient_queue.as_ref());

        DoctorQueueAppointment {
            appointment_id: row.id,
            patient_id: row.patient_id,
            patient_name: format!("{} {}", row.patients.first_name, row.patients.last_name)
                .trim()
                .to_string(),
            appointment_date: row.scheduled_at.format("%Y-%m-%d").to_string(),
            appointment_time: format_time(row.scheduled_at),
            queue_status,
            priority,
        }
    }
}

fn queue_status_from_value(value: Option<&Value>) -> String {
    match value {
        Some(Value::Array(rows)) => rows
            .first()
            .and_then(|row| row.get("status"))
            .and_then(Value::as_str)
            .unwrap_or("Not Checked In")
            .to_string(),

        Some(Value::Object(row)) => row
            .get("status")
            .and_then(Value::as_str)
            .unwrap_or("Not Checked In")
            .to_string(),

        _ => "Not Checked In".to_string(),
    }
}

fn queue_priority_from_value(value: Option<&Value>) -> String {
    match value {
        Some(Value::Array(rows)) => rows
            .first()
            .and_then(|row| row.get("priority"))
            .and_then(Value::as_str)
            .unwrap_or("Normal")
            .to_string(),

        Some(Value::Object(row)) => row
            .get("priority")
            .and_then(Value::as_str)
            .unwrap_or("Normal")
            .to_string(),

        _ => "Normal".to_string(),
    }
}

fn format_time(value: DateTime<Utc>) -> String {
    let hour = value.hour();
    let minute = value.minute();
    let suffix = if hour < 12 { "AM" } else { "PM" };
    let hour_12 = match hour % 12 {
        0 => 12,
        other => other,
    };

    format!("{hour_12}:{minute:02} {suffix}")
}
