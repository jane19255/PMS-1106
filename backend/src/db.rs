use reqwest::Client;
use serde_json::Value;

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
        Ok(response.text().await?)
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
        let mut query = format!(
            "medical_records?patient_id=eq.{}&select=*&order=recorded_at.desc.nullslast&limit=1",
            encoded_patient_id
        );

        if let Some(appointment_id) = appointment_id.filter(|id| !id.trim().is_empty()) {
            query = format!(
                "medical_records?patient_id=eq.{}&appointment_id=eq.{}&select=*&order=recorded_at.desc.nullslast&limit=1",
                encoded_patient_id,
                urlencoding::encode(appointment_id)
            );
        }

        let response = self
            .authed(self.http_client.get(self.rest_url(&query)))
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

    pub async fn delete_medical_record(&self, record_id: &str) -> Result<String, String> {
        let encoded_id = urlencoding::encode(record_id);
        let url = self.rest_url(&format!("medical_records?id=eq.{}", encoded_id));
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

    pub async fn delete_appointment(&self, appointment_id: &str) -> Result<String, String> {
        let encoded_id = urlencoding::encode(appointment_id);
        let url = self.rest_url(&format!("appointments?id=eq.{}", encoded_id));
        let response = self
            .authed(self.http_client.delete(url))
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

    // Queue Management methods
    pub async fn list_queue_by_doctor(&self, doctor_id: &str) -> Result<String, String> {
        let encoded_id = urlencoding::encode(doctor_id);
        let url = self.rest_url(&format!(
            "patient_queue?select=*,appointments!inner(doctor_id)&appointments.doctor_id=eq.{}&status=neq.Completed&order=checked_in_at.asc",
            encoded_id
        ));

        let response = self
            .authed(self.http_client.get(url))
            .send()
            .await
            .map_err(|e| e.to_string())?;

        Self::read_response(response).await
    }

    pub async fn update_queue_entry(
        &self,
        queue_id: &str,
        payload: &serde_json::Value,
    ) -> Result<String, String> {
        let encoded_id = urlencoding::encode(queue_id);
        let url = self.rest_url(&format!("patient_queue?id=eq.{}", encoded_id));
        let response = self
            .authed(self.http_client.patch(url))
            .header("Prefer", "return=representation")
            .json(payload)
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
    pub async fn insert_row(
        &self,
        table_name: &str,
        payload: &serde_json::Value,
    ) -> Result<String, String> {
        let url = self.rest_url(table_name);
        let req = self
            .authed(self.http_client.post(url))
            .header("Prefer", "return=representation")
            .json(payload);
        let res = req.send().await.map_err(|e| e.to_string())?;
        Self::read_response(res).await
    }
}

/// Extracts the `message` field from a PostgREST/Postgres error body embedded in a
/// `SupabaseRestDb::read_response` error string (e.g. a `raise exception '...'` from
/// an RPC function), so callers can surface the real business-rule reason to the
/// client instead of a generic 500.
pub fn pg_error_message(err: &str) -> Option<String> {
    let start = err.find('{')?;
    let json: Value = serde_json::from_str(&err[start..]).ok()?;
    json.get("message")
        .and_then(Value::as_str)
        .map(str::to_string)
}
