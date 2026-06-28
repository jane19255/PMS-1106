use super::models::{DashboardAppointment, DashboardPatient};
use chrono::{DateTime, Timelike, Utc};
use reqwest::{Client, Response};
use serde::Deserialize;
use std::future::Future;
use std::pin::Pin;

pub type RepositoryFuture<'a, T> =
    Pin<Box<dyn Future<Output = Result<T, RepositoryError>> + Send + 'a>>;

#[derive(Debug)]
pub enum RepositoryError {
    StorageUnavailable,
}

pub trait DoctorDashboardRepository: Send + Sync {
    fn list_appointments(&self) -> RepositoryFuture<'_, Vec<DashboardAppointment>>;
}

#[derive(Default)]
pub struct InMemoryDoctorDashboardRepository;

impl DoctorDashboardRepository for InMemoryDoctorDashboardRepository {
    fn list_appointments(&self) -> RepositoryFuture<'_, Vec<DashboardAppointment>> {
        Box::pin(async { Ok(Vec::new()) })
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

    fn appointments_url(&self) -> String {
        let select = "id,scheduled_at,reason,status,patient:patient_id(id,first_name,last_name,dob,gender,phone,email),doctor:doctor_id(id,room,staff:staff_id(full_name))";
        format!(
            "{}/rest/v1/appointments?select={}&order=scheduled_at.asc",
            self.url, select
        )
    }

    async fn decode_rows(response: Response) -> Result<Vec<DatabaseAppointment>, RepositoryError> {
        if !response.status().is_success() {
            return Err(Self::response_error(response).await);
        }
        response
            .json::<Vec<DatabaseAppointment>>()
            .await
            .map_err(|_| RepositoryError::StorageUnavailable)
    }

    async fn response_error(response: Response) -> RepositoryError {
        let status = response.status();
        let body = response.text().await.unwrap_or_default();
        eprintln!("Supabase doctor dashboard error {status}: {body}");
        RepositoryError::StorageUnavailable
    }
}

impl DoctorDashboardRepository for SupabaseDoctorDashboardRepository {
    fn list_appointments(&self) -> RepositoryFuture<'_, Vec<DashboardAppointment>> {
        Box::pin(async move {
            let response = self
                .request(self.client.get(self.appointments_url()))
                .send()
                .await
                .map_err(|_| RepositoryError::StorageUnavailable)?;
            let rows = Self::decode_rows(response).await?;
            Ok(rows.into_iter().map(DashboardAppointment::from).collect())
        })
    }
}

#[derive(Deserialize)]
struct DatabaseAppointment {
    id: String,
    scheduled_at: DateTime<Utc>,
    reason: String,
    status: String,
    patient: DatabasePatient,
    doctor: DatabaseDoctor,
}

#[derive(Deserialize)]
struct DatabasePatient {
    id: String,
    first_name: String,
    last_name: String,
    dob: String,
    gender: String,
    phone: String,
    email: String,
}

#[derive(Deserialize)]
struct DatabaseDoctor {
    room: Option<String>,
    staff: Option<DatabaseDoctorStaff>,
}

#[derive(Deserialize)]
struct DatabaseDoctorStaff {
    full_name: String,
}

impl From<DatabaseAppointment> for DashboardAppointment {
    fn from(row: DatabaseAppointment) -> Self {
        let doctor_name = row
            .doctor
            .staff
            .map(|staff| staff.full_name)
            .unwrap_or_else(|| "Unassigned doctor".to_string());
        let reason = row.reason.trim().to_string();

        Self {
            appointment_id: row.id,
            priority: priority_from_reason(&reason),
            doctor: doctor_name,
            date: row.scheduled_at.format("%Y-%m-%d").to_string(),
            time: format_time(row.scheduled_at),
            room: row.doctor.room.unwrap_or_default(),
            appointment_type: if reason.is_empty() { "Consultation".to_string() } else { reason.clone() },
            referring_provider: String::new(),
            special_requirements: Vec::new(),
            status: dashboard_status(&row.status),
            patient: DashboardPatient {
                id: row.patient.id,
                first_name: row.patient.first_name,
                last_name: row.patient.last_name,
                dob: row.patient.dob,
                gender: row.patient.gender,
                phone: row.patient.phone,
                email: row.patient.email,
            },
        }
    }
}

fn priority_from_reason(reason: &str) -> String {
    let normalized = reason.to_lowercase();
    if normalized.contains("emergency") {
        "Emergency".to_string()
    } else if normalized.contains("urgent") {
        "Urgent".to_string()
    } else if normalized.contains("follow") {
        "Follow-up".to_string()
    } else {
        "Normal".to_string()
    }
}

fn dashboard_status(status: &str) -> String {
    match status.trim() {
        "Checked In" => "Checked-In".to_string(),
        "In Consultation" => "In-Room".to_string(),
        "No Show" => "No-Show".to_string(),
        other => other.to_string(),
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

#[cfg(test)]
mod tests {
    use super::{dashboard_status, priority_from_reason};

    #[test]
    fn maps_supabase_status_for_dashboard_badges() {
        assert_eq!(dashboard_status("Checked In"), "Checked-In");
        assert_eq!(dashboard_status("In Consultation"), "In-Room");
        assert_eq!(dashboard_status("No Show"), "No-Show");
        assert_eq!(dashboard_status("Completed"), "Completed");
    }

    #[test]
    fn derives_priority_from_reason() {
        assert_eq!(priority_from_reason("Emergency review"), "Emergency");
        assert_eq!(priority_from_reason("urgent referral"), "Urgent");
        assert_eq!(priority_from_reason("Follow up"), "Follow-up");
        assert_eq!(priority_from_reason("Routine checkup"), "Normal");
    }
}