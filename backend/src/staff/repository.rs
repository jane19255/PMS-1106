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
    ReferencedByDoctor,
    NotFound,
    StorageUnavailable,
}

pub trait StaffRepository: Send + Sync {
    fn create(&self, staff: StaffMember) -> RepositoryFuture<'_, StaffMember>;
    fn list(&self) -> RepositoryFuture<'_, Vec<StaffMember>>;
    fn update(&self, staff: StaffMember) -> RepositoryFuture<'_, StaffMember>;
    fn delete(&self, staff_id: &str) -> RepositoryFuture<'_, ()>;
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

    fn delete(&self, staff_id: &str) -> RepositoryFuture<'_, ()> {
        let staff_id = staff_id.to_string();
        Box::pin(async move {
            let removed = self
                .staff
                .lock()
                .map_err(|_| RepositoryError::StorageUnavailable)?
                .remove(&staff_id);
            removed.map(|_| ()).ok_or(RepositoryError::NotFound)
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

        if body.contains("staff_phone_check") {
            RepositoryError::InvalidPhone
        } else if body.contains("staff_email_check") {
            RepositoryError::InvalidEmail
        } else if body.contains("staff_email_key") || body.contains("staff_email_lower_uidx") {
            RepositoryError::DuplicateEmail
        } else if body.contains("staff_firebase_uid_key") {
            RepositoryError::DuplicateFirebaseUid
        } else if body.contains("doctors_staff_id_fkey") {
            RepositoryError::ReferencedByDoctor
        } else {
            RepositoryError::StorageUnavailable
        }
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

    fn delete(&self, staff_id: &str) -> RepositoryFuture<'_, ()> {
        let encoded_id = urlencoding::encode(staff_id).into_owned();
        Box::pin(async move {
            let response = self
                .request(
                    self.client
                        .delete(self.staff_url(&format!("id=eq.{encoded_id}&select=id"))),
                )
                .header("Prefer", "return=representation")
                .send()
                .await
                .map_err(|_| RepositoryError::StorageUnavailable)?;
            if !response.status().is_success() {
                return Err(Self::response_error(response).await);
            }
            let rows = response
                .json::<Vec<serde_json::Value>>()
                .await
                .map_err(|_| RepositoryError::StorageUnavailable)?;
            if rows.is_empty() {
                Err(RepositoryError::NotFound)
            } else {
                Ok(())
            }
        })
    }
}

#[derive(Deserialize)]
struct DatabaseStaff {
    id: String,
    firebase_uid: String,
    full_name: String,
    dob: Option<String>,
    gender: Option<String>,
    nric: Option<String>,
    email: String,
    phone: Option<String>,
    role: String,
    status: String,
    address: Option<String>,
    emergency_contact: Option<String>,
}

impl From<DatabaseStaff> for StaffMember {
    fn from(row: DatabaseStaff) -> Self {
        let (first_name, last_name) = split_full_name(&row.full_name);
        Self {
            id: row.id,
            firebase_uid: row.firebase_uid,
            first_name,
            last_name,
            dob: row.dob.unwrap_or_default(),
            gender: row.gender.unwrap_or_default(),
            nric: row.nric.unwrap_or_default(),
            role: display_role(&row.role),
            phone: row.phone.unwrap_or_default(),
            email: row.email,
            status: display_status(&row.status),
            address: row.address.unwrap_or_default(),
            emergency: row.emergency_contact.unwrap_or_default(),
        }
    }
}

fn database_payload(staff: &StaffMember) -> serde_json::Value {
    json!({
        "id": staff.id,
        "firebase_uid": staff.firebase_uid,
        "full_name": staff.full_name(),
        "dob": optional_text(&staff.dob),
        "gender": optional_text(&staff.gender),
        "nric": optional_text(&staff.nric),
        "email": staff.email,
        "phone": if staff.phone.trim().is_empty() { None } else { Some(staff.phone.trim()) },
        "role": database_role(&staff.role),
        "status": database_status(&staff.status),
        "address": optional_text(&staff.address),
        "emergency_contact": optional_text(&staff.emergency),
    })
}

fn optional_text(value: &str) -> Option<&str> {
    let value = value.trim();
    if value.is_empty() {
        None
    } else {
        Some(value)
    }
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
