use super::models::StaffMember;
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
    InvalidEmail,
    InvalidPhone,
    DuplicateEmail,
    DuplicateFirebaseUid,
    NotFound,
    StorageUnavailable,
}

pub trait StaffRepository: Send + Sync {
    fn create(&self, staff: StaffMember) -> RepositoryFuture<'_, StaffMember>;
    fn list(&self) -> RepositoryFuture<'_, Vec<StaffMember>>;
    fn update(&self, staff: StaffMember) -> RepositoryFuture<'_, StaffMember>;
}

#[derive(Default)]
pub struct InMemoryStaffRepository {
    staff: Mutex<HashMap<String, StaffMember>>,
}

impl StaffRepository for InMemoryStaffRepository {
    fn create(&self, staff: StaffMember) -> RepositoryFuture<'_, StaffMember> {
        Box::pin(async move {
            let mut staff_members = self
                .staff
                .lock()
                .map_err(|_| RepositoryError::StorageUnavailable)?;
            staff_members.insert(staff.id.clone(), staff.clone());
            Ok(staff)
        })
    }

    fn list(&self) -> RepositoryFuture<'_, Vec<StaffMember>> {
        Box::pin(async move {
            let staff_members = self
                .staff
                .lock()
                .map_err(|_| RepositoryError::StorageUnavailable)?;
            let mut list: Vec<StaffMember> = staff_members.values().cloned().collect();
            list.sort_by(|left, right| left.id.cmp(&right.id));
            Ok(list)
        })
    }

    fn update(&self, staff: StaffMember) -> RepositoryFuture<'_, StaffMember> {
        Box::pin(async move {
            let mut staff_members = self
                .staff
                .lock()
                .map_err(|_| RepositoryError::StorageUnavailable)?;
            if !staff_members.contains_key(&staff.id) {
                return Err(RepositoryError::NotFound);
            }
            staff_members.insert(staff.id.clone(), staff.clone());
            Ok(staff)
        })
    }
}

pub struct SupabaseStaffRepository {
    url: String,
    key: String,
    client: Client,
}

impl SupabaseStaffRepository {
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

    fn staff_url(&self, query: &str) -> String {
        format!("{}/rest/v1/staff?{}", self.url, query)
    }

    async fn decode_rows(response: Response) -> Result<Vec<DatabaseStaff>, RepositoryError> {
        if !response.status().is_success() {
            return Err(Self::response_error(response).await);
        }
        response
            .json::<Vec<DatabaseStaff>>()
            .await
            .map_err(|_| RepositoryError::StorageUnavailable)
    }

    async fn response_error(response: Response) -> RepositoryError {
        let status = response.status();
        let body = response.text().await.unwrap_or_default();
        eprintln!("Supabase staff error {status}: {body}");
        RepositoryError::StorageUnavailable
    }
}

impl StaffRepository for SupabaseStaffRepository {
    fn create(&self, staff: StaffMember) -> RepositoryFuture<'_, StaffMember> {
        Box::pin(async move {
            let response = self
                .request(self.client.post(self.staff_url("select=*")))
                .header("Prefer", "return=representation")
                .json(&database_payload(&staff))
                .send()
                .await
                .map_err(|_| RepositoryError::StorageUnavailable)?;
            let mut rows = Self::decode_rows(response).await?;
            rows.pop()
                .map(StaffMember::from)
                .ok_or(RepositoryError::StorageUnavailable)
        })
    }

    fn list(&self) -> RepositoryFuture<'_, Vec<StaffMember>> {
        Box::pin(async move {
            let response = self
                .request(self.client.get(self.staff_url("select=*&order=id.asc")))
                .send()
                .await
                .map_err(|_| RepositoryError::StorageUnavailable)?;
            Ok(Self::decode_rows(response)
                .await?
                .into_iter()
                .map(StaffMember::from)
                .collect())
        })
    }

    fn update(&self, staff: StaffMember) -> RepositoryFuture<'_, StaffMember> {
        Box::pin(async move {
            let encoded_id = urlencoding::encode(&staff.id);
            let response = self
                .request(
                    self.client
                        .patch(self.staff_url(&format!("id=eq.{encoded_id}&select=*"))),
                )
                .header("Prefer", "return=representation")
                .json(&database_payload(&staff))
                .send()
                .await
                .map_err(|_| RepositoryError::StorageUnavailable)?;
            let mut rows = Self::decode_rows(response).await?;
            rows.pop()
                .map(StaffMember::from)
                .ok_or(RepositoryError::NotFound)
        })
    }
}

#[derive(Deserialize)]
struct DatabaseStaff {
    id: String,
    firebase_uid: String,
    full_name: String,
    email: String,
    phone: Option<String>,
    role: String,
    status: String,
}

impl From<DatabaseStaff> for StaffMember {
    fn from(row: DatabaseStaff) -> Self {
        let (first_name, last_name) = split_full_name(&row.full_name);
        Self {
            id: row.id,
            firebase_uid: row.firebase_uid,
            first_name,
            last_name,
            role: display_role(&row.role),
            phone: row.phone.unwrap_or_default(),
            email: row.email,
            status: display_status(&row.status),
        }
    }
}

fn database_payload(staff: &StaffMember) -> serde_json::Value {
    json!({
        "id": staff.id,
        "firebase_uid": staff.firebase_uid,
        "full_name": staff.full_name(),
        "email": staff.email,
        "phone": if staff.phone.trim().is_empty() { None } else { Some(staff.phone.trim()) },
        "role": database_role(&staff.role),
        "status": database_status(&staff.status),
    })
}

fn split_full_name(full_name: &str) -> (String, String) {
    let trimmed = full_name.trim();
    let Some((first, last)) = trimmed.split_once(' ') else {
        return (trimmed.to_string(), String::new());
    };
    (first.to_string(), last.trim().to_string())
}

fn database_role(role: &str) -> String {
    match role.trim().to_lowercase().as_str() {
        "doctor" => "doctor".to_string(),
        "pharmacist" => "pharmacist".to_string(),
        "admin" => "admin".to_string(),
        _ => "receptionist".to_string(),
    }
}

fn display_role(role: &str) -> String {
    match role.trim().to_lowercase().as_str() {
        "doctor" => "Doctor".to_string(),
        "pharmacist" => "Pharmacist".to_string(),
        "admin" => "Admin".to_string(),
        _ => "Receptionist".to_string(),
    }
}

fn database_status(status: &str) -> String {
    if status.trim().eq_ignore_ascii_case("inactive") {
        "inactive".to_string()
    } else {
        "active".to_string()
    }
}

fn display_status(status: &str) -> String {
    if status.trim().eq_ignore_ascii_case("inactive") {
        "Inactive".to_string()
    } else {
        "Active".to_string()
    }
}
