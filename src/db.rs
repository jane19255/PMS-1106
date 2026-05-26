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

    /// Helper function to build the base Firestore URL
    fn base_url(&self) -> String {
        format!(
            "https://firestore.googleapis.com/v1/projects/{}/databases/(default)/documents",
            self.project_id
        )
    }

    // ── READ ────────────────────────────────────────────────────────────────
    pub async fn get_document(&self, collection: &str, doc_id: &str) -> Result<String, reqwest::Error> {
        let url = format!("{}/{}/{}", self.base_url(), collection, doc_id);
        
        let response = self.http_client.get(&url).send().await?;
        Ok(response.text().await?)
    }

    // ── CREATE ──────────────────────────────────────────────────────────────
    pub async fn create_document(
        &self, 
        collection: &str, 
        doc_id: &str, 
        payload: &Value
    ) -> Result<String, reqwest::Error> {
        // To create a document with a specific ID in Firestore's REST API, 
        // we POST to the collection URL and pass the ID as a query parameter.
        let url = format!("{}/{}?documentId={}", self.base_url(), collection, doc_id);
        
        let response = self.http_client.post(&url).json(payload).send().await?;
        Ok(response.text().await?)
    }

    // ── UPDATE ──────────────────────────────────────────────────────────────
    pub async fn update_document(
        &self, 
        collection: &str, 
        doc_id: &str, 
        payload: &Value
    ) -> Result<String, reqwest::Error> {
        let url = format!("{}/{}/{}", self.base_url(), collection, doc_id);
        
        let response = self.http_client.patch(&url).json(payload).send().await?;
        Ok(response.text().await?)
    }

    // ── DELETE ──────────────────────────────────────────────────────────────
    pub async fn delete_document(&self, collection: &str, doc_id: &str) -> Result<String, reqwest::Error> {
        let url = format!("{}/{}/{}", self.base_url(), collection, doc_id);
        
        let response = self.http_client.delete(&url).send().await?;
        Ok(response.text().await?)
    }
}