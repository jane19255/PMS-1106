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

    pub async fn get_document(&self, collection: &str, doc_id: &str) -> Result<String, reqwest::Error> {
        let url = format!("{}/{}/{}", self.base_url(), collection, doc_id);
        let response = self.http_client.get(&url).send().await?;
        Ok(response.text().await?)
    }

    pub async fn create_document(
        &self,
        collection: &str,
        doc_id: &str,
        payload: &Value,
    ) -> Result<String, reqwest::Error> {
        let url = format!("{}/{}?documentId={}", self.base_url(), collection, doc_id);
        let response = self.http_client.post(&url).json(payload).send().await?;
        Ok(response.text().await?)
    }

    pub async fn update_document(
        &self,
        collection: &str,
        doc_id: &str,
        payload: &Value,
    ) -> Result<String, reqwest::Error> {
        let url = format!("{}/{}/{}", self.base_url(), collection, doc_id);
        let response = self.http_client.patch(&url).json(payload).send().await?;
        Ok(response.text().await?)
    }

    pub async fn delete_document(&self, collection: &str, doc_id: &str) -> Result<String, reqwest::Error> {
        let url = format!("{}/{}/{}", self.base_url(), collection, doc_id);
        let response = self.http_client.delete(&url).send().await?;
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
        let url = std::env::var("SUPABASE_URL")
            .expect("SUPABASE_URL must be set in .env");
        let key = std::env::var("SUPABASE_KEY")
            .expect("SUPABASE_KEY must be set in .env");

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

    fn rest_url(&self, path: &str) -> String {
        format!("{}/rest/v1/{}", self.url, path.trim_start_matches('/'))
    }

    fn authed(&self, builder: reqwest::RequestBuilder) -> reqwest::RequestBuilder {
        let builder = builder
            .header("apikey", &self.key)
            .header("Content-Type", "application/json");

        if self.key.starts_with("eyJ") {
            builder.header("Authorization", format!("Bearer {}", &self.key))
        } else {
            builder
        }
    }

    async fn read_response(response: reqwest::Response) -> Result<String, String> {
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

    pub async fn update_patient(&self, patient_id: &str, payload: &Value) -> Result<String, String> {
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
}
