//! Shared database and time utilities.
//!
//! This module contains lightweight REST clients for Firebase and Supabase plus
//! Singapore time helpers used by appointment and dashboard workflows.
use chrono::{DateTime, FixedOffset, NaiveDate, TimeZone, Utc};
use reqwest::Client;
use serde_json::Value;

pub fn singapore_offset() -> FixedOffset {
    FixedOffset::east_opt(8 * 60 * 60).expect("Singapore offset is valid")
}

pub fn singapore_now() -> DateTime<FixedOffset> {
    Utc::now().with_timezone(&singapore_offset())
}

pub fn singapore_today() -> NaiveDate {
    // Clinic dates must follow Singapore time instead of the server's UTC date.
    singapore_now().date_naive()
}

pub fn singapore_day_bounds(date: NaiveDate) -> (DateTime<Utc>, DateTime<Utc>) {
    // Supabase stores UTC instants, so Singapore midnight is 16:00 UTC the day before.
    let start = singapore_offset()
        .from_local_datetime(&date.and_hms_opt(0, 0, 0).expect("midnight is valid"))
        .single()
        .expect("Singapore has no daylight-saving ambiguity")
        .with_timezone(&Utc);
    (start, start + chrono::Duration::days(1))
}

#[derive(Clone)]
pub struct FirebaseRestDb {
    pub project_id: String,
    pub http_client: Client,
}

impl FirebaseRestDb {
    pub fn new(project_id: String) -> Self {
        Self {
            project_id,
            http_client: Client::new(),
        }
    }

    fn base_url(&self) -> String {
        format!(
            "https://firestore.googleapis.com/v1/projects/{}/databases/(default)/documents",
            self.project_id
        )
    }

    pub async fn get_document(
        &self,
        collection: &str,
        doc_id: &str,
    ) -> Result<String, reqwest::Error> {
        let url = format!("{}/{}/{}", self.base_url(), collection, doc_id);
        let response = self.http_client.get(&url).send().await?;
        response.text().await
    }
}

#[derive(Clone)]
pub struct SupabaseRestDb {
    pub url: String,
    pub key: String,
    pub http_client: Client,
}

impl SupabaseRestDb {
    pub fn from_env() -> Self {
        let url = std::env::var("SUPABASE_URL").expect("SUPABASE_URL must be set in .env");
        let key = std::env::var("SUPABASE_KEY").expect("SUPABASE_KEY must be set in .env");

        let url = url.trim().trim_end_matches('/').to_string();
        let key = key.trim().to_string();

        if key.starts_with("sb_publishable_") {
            eprintln!(
                "Warning: SUPABASE_KEY is a publishable key. For this backend-only Supabase design, use a Supabase secret key in .env."
            );
        }

        println!("Supabase config loaded successfully.");

        Self {
            url,
            key,
            http_client: Client::new(),
        }
    }

    pub fn rest_url(&self, path: &str) -> String {
        format!("{}/rest/v1/{}", self.url, path.trim_start_matches('/'))
    }

    pub fn authed(&self, builder: reqwest::RequestBuilder) -> reqwest::RequestBuilder {
        builder
            .header("apikey", &self.key)
            .header("Authorization", format!("Bearer {}", &self.key))
            .header("Content-Type", "application/json")
    }

    pub async fn read_response(response: reqwest::Response) -> Result<String, String> {
        let status = response.status();
        let body = response.text().await.unwrap_or_default();
        if status.is_success() {
            Ok(body)
        } else {
            Err(format!("Supabase returned {}: {}", status, body))
        }
    }

    pub async fn list_patients(&self) -> Result<String, String> {
        let url = self.rest_url("patients?select=*&order=created_at.desc.nullslast");
        let response = self
            .authed(self.http_client.get(url))
            .send()
            .await
            .map_err(|e| e.to_string())?;
        Self::read_response(response).await
    }

    pub async fn get_patient(&self, patient_id: &str) -> Result<String, String> {
        let encoded_id = urlencoding::encode(patient_id);
        let url = self.rest_url(&format!("patients?id=eq.{}&select=*&limit=1", encoded_id));
        let response = self
            .authed(self.http_client.get(url))
            .send()
            .await
            .map_err(|e| e.to_string())?;
        Self::read_response(response).await
    }

    pub async fn get_latest_medical_record(
        &self,
        patient_id: &str,
        appointment_id: Option<&str>,
    ) -> Result<String, String> {
        let encoded_patient_id = urlencoding::encode(patient_id);
        let patient_only_query = format!(
            "medical_records?patient_id=eq.{}&select=*&order=recorded_at.desc.nullslast&limit=1",
            encoded_patient_id
        );

        if let Some(appointment_id) = appointment_id.filter(|id| !id.trim().is_empty()) {
            let query = format!(
                "medical_records?patient_id=eq.{}&appointment_id=eq.{}&select=*&order=recorded_at.desc.nullslast&limit=1",
                encoded_patient_id,
                urlencoding::encode(appointment_id)
            );
            let response = self
                .authed(self.http_client.get(self.rest_url(&query)))
                .send()
                .await
                .map_err(|e| e.to_string())?;
            let body = Self::read_response(response).await?;

            // Records aren't always saved with an appointment link, so an
            // exact match here isn't reliable — fall back to the patient's
            // most recent record rather than reporting nothing found.
            let has_match = serde_json::from_str::<Vec<Value>>(&body)
                .map(|rows| !rows.is_empty())
                .unwrap_or(false);
            if has_match {
                return Ok(body);
            }
        }

        let response = self
            .authed(self.http_client.get(self.rest_url(&patient_only_query)))
            .send()
            .await
            .map_err(|e| e.to_string())?;
        Self::read_response(response).await
    }

    pub async fn create_patient(&self, payload: &Value) -> Result<String, String> {
        let url = self.rest_url("patients");
        let response = self
            .authed(self.http_client.post(url))
            .header("Prefer", "return=representation")
            .json(payload)
            .send()
            .await
            .map_err(|e| e.to_string())?;
        Self::read_response(response).await
    }

    pub async fn update_patient(
        &self,
        patient_id: &str,
        payload: &Value,
    ) -> Result<String, String> {
        let encoded_id = urlencoding::encode(patient_id);
        let url = self.rest_url(&format!("patients?id=eq.{}", encoded_id));
        let response = self
            .authed(self.http_client.patch(url))
            .header("Prefer", "return=representation")
            .json(payload)
            .send()
            .await
            .map_err(|e| e.to_string())?;
        Self::read_response(response).await
    }

    pub async fn delete_patient(&self, patient_id: &str) -> Result<String, String> {
        let encoded_id = urlencoding::encode(patient_id);
        let url = self.rest_url(&format!("patients?id=eq.{}", encoded_id));
        let response = self
            .authed(self.http_client.delete(url))
            .send()
            .await
            .map_err(|e| e.to_string())?;
        Self::read_response(response).await
    }

    // Appointment Scheduling methods
    pub async fn list_appointments(&self) -> Result<String, String> {
        let url = self.rest_url("appointments?select=*&order=scheduled_at");
        let response = self
            .authed(self.http_client.get(url))
            .send()
            .await
            .map_err(|e| e.to_string())?;
        Self::read_response(response).await
    }

    /// Reconciles overdue appointments by marking them as "no-show" in the database.
    ///
    /// This invokes a RPC on the Supabase database to bulk-update all appointments with dates set before the provided date accordingly.
    /// 
    /// ### Arguments
    /// * `today` - The calendar date used as the cut-off boundary to determine appointments as "no-show"
    /// 
    /// ### Returns
    /// * `Ok(String)` - A success message or confirmation payload from the Supabase database
    /// * `Err(String)` - An error message regarding the network or API failure
    pub async fn reconcile_overdue_appointments(
        &self,
        today: NaiveDate,
    ) -> Result<String, String> {
        // The database marks old untouched bookings as no-shows in one update.
        let url = self.rest_url("rpc/reconcile_overdue_appointments");
        let response = self
            .authed(self.http_client.post(url))
            .json(&serde_json::json!({
                "p_today": today.format("%Y-%m-%d").to_string()
            }))
            .send()
            .await
            .map_err(|error| error.to_string())?;
        Self::read_response(response).await
    }

    pub async fn get_appointment(&self, appointment_id: &str) -> Result<String, String> {
        let encoded_id = urlencoding::encode(appointment_id);
        let url = self.rest_url(&format!("appointments?id=eq.{}&select=*", encoded_id));
        let response = self
            .authed(self.http_client.get(url))
            .send()
            .await
            .map_err(|e| e.to_string())?;
        Self::read_response(response).await
    }

    pub async fn create_appointment(&self, payload: &Value) -> Result<String, String> {
        let url = self.rest_url("appointments");
        let response = self
            .authed(self.http_client.post(url))
            .header("Prefer", "return=representation")
            .json(payload)
            .send()
            .await
            .map_err(|e| e.to_string())?;
        Self::read_response(response).await
    }

    pub async fn update_appointment(
        &self,
        appointment_id: &str,
        payload: &Value,
    ) -> Result<String, String> {
        let encoded_id = urlencoding::encode(appointment_id);
        let url = self.rest_url(&format!("appointments?id=eq.{}", encoded_id));
        let response = self
            .authed(self.http_client.patch(url))
            .header("Prefer", "return=representation")
            .json(payload)
            .send()
            .await
            .map_err(|e| e.to_string())?;
        Self::read_response(response).await
    }

    pub async fn list_appointments_by_doctor(&self, doctor_id: &str) -> Result<String, String> {
        let encoded_id = urlencoding::encode(doctor_id);
        let url = self.rest_url(&format!(
            "appointments?doctor_id=eq.{}&select=*&order=scheduled_at",
            encoded_id
        ));
        let response = self
            .authed(self.http_client.get(url))
            .send()
            .await
            .map_err(|e| e.to_string())?;
        Self::read_response(response).await
    }

    pub async fn fetch_table(
        &self,
        table_name: &str,
        filter_params: &str,
    ) -> Result<String, String> {
        let full_path = format!("{}?{}", table_name, filter_params);
        let url = self.rest_url(&full_path);
        let req = self.authed(self.http_client.get(url));
        let res = req.send().await.map_err(|e| e.to_string())?;
        Self::read_response(res).await
    }
}
