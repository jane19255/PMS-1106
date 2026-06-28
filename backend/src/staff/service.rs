use super::models::StaffMember;
use super::repository::{RepositoryError, StaffRepository};
use serde::Deserialize;
use std::sync::Arc;
use uuid::Uuid;

#[derive(Debug, PartialEq)]
pub enum StaffError {
    InvalidInput(String),
    StaffNotFound,
    StorageUnavailable,
}

#[derive(Clone, Debug, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct StaffForm {
    pub firebase_uid: String,
    pub first_name: String,
    pub last_name: String,
    pub role: String,
    pub phone: String,
    pub email: String,
    pub status: Option<String>,
}

pub struct StaffService {
    staff_repository: Arc<dyn StaffRepository>,
}

impl StaffService {
    pub fn new(staff_repository: Arc<dyn StaffRepository>) -> Self {
        Self { staff_repository }
    }

    pub async fn create_staff(&self, form: StaffForm) -> Result<StaffMember, StaffError> {
        self.validate_staff_form(&form)?;
        let staff = StaffMember {
            id: format!("STF-{}", Uuid::new_v4()),
            firebase_uid: form.firebase_uid.trim().to_string(),
            first_name: form.first_name.trim().to_string(),
            last_name: form.last_name.trim().to_string(),
            role: normalize_role(&form.role),
            phone: form.phone.trim().to_string(),
            email: form.email.trim().to_string(),
            status: form
                .status
                .as_deref()
                .map(normalize_status)
                .unwrap_or_else(|| "Active".to_string()),
        };

        self.staff_repository
            .create(staff)
            .await
            .map_err(Self::map_repository_error)
    }

    pub async fn list_staff(&self) -> Result<Vec<StaffMember>, StaffError> {
        self.staff_repository
            .list()
            .await
            .map_err(Self::map_repository_error)
    }

    pub async fn update_staff(
        &self,
        staff_id: &str,
        form: StaffForm,
    ) -> Result<StaffMember, StaffError> {
        if staff_id.trim().is_empty() {
            return Err(StaffError::InvalidInput("Staff ID is required".to_string()));
        }
        self.validate_staff_form(&form)?;
        let staff = StaffMember {
            id: staff_id.trim().to_string(),
            firebase_uid: form.firebase_uid.trim().to_string(),
            first_name: form.first_name.trim().to_string(),
            last_name: form.last_name.trim().to_string(),
            role: normalize_role(&form.role),
            phone: form.phone.trim().to_string(),
            email: form.email.trim().to_string(),
            status: form
                .status
                .as_deref()
                .map(normalize_status)
                .unwrap_or_else(|| "Active".to_string()),
        };

        self.staff_repository
            .update(staff)
            .await
            .map_err(Self::map_repository_error)
    }

    fn validate_staff_form(&self, form: &StaffForm) -> Result<(), StaffError> {
        for (field_name, value) in [
            ("Firebase UID", form.firebase_uid.as_str()),
            ("First name", form.first_name.as_str()),
            ("Last name", form.last_name.as_str()),
            ("Role", form.role.as_str()),
            ("Email", form.email.as_str()),
        ] {
            if value.trim().is_empty() {
                return Err(StaffError::InvalidInput(format!(
                    "{field_name} is required"
                )));
            }
        }

        if !matches!(
            form.role.trim().to_lowercase().as_str(),
            "admin" | "doctor" | "receptionist" | "pharmacist"
        ) {
            return Err(StaffError::InvalidInput(
                "Role must be admin, doctor, receptionist, or pharmacist".to_string(),
            ));
        }
        if let Some(status) = form.status.as_deref() {
            if !matches!(status.trim().to_lowercase().as_str(), "active" | "inactive") {
                return Err(StaffError::InvalidInput(
                    "Status must be Active or Inactive".to_string(),
                ));
            }
        }

        let phone = form.phone.trim();
        if !phone.is_empty() && !is_singapore_phone(phone) {
            return Err(StaffError::InvalidInput(
                "Phone number must be 8 digits and start with 6, 8, or 9".to_string(),
            ));
        }

        if !is_valid_email(form.email.trim()) {
            return Err(StaffError::InvalidInput(
                "Email format is invalid".to_string(),
            ));
        }

        Ok(())
    }

    fn map_repository_error(error: RepositoryError) -> StaffError {
        match error {
            RepositoryError::InvalidEmail => {
                StaffError::InvalidInput("Email format is invalid".to_string())
            }
            RepositoryError::InvalidPhone => StaffError::InvalidInput(
                "Phone number must be 8 digits and start with 6, 8, or 9".to_string(),
            ),
            RepositoryError::DuplicateEmail => StaffError::InvalidInput(
                "Email is already used by another staff member".to_string(),
            ),
            RepositoryError::DuplicateFirebaseUid => StaffError::InvalidInput(
                "Firebase UID is already used by another staff member".to_string(),
            ),
            RepositoryError::NotFound => StaffError::StaffNotFound,
            RepositoryError::StorageUnavailable => StaffError::StorageUnavailable,
        }
    }
}

fn normalize_role(role: &str) -> String {
    match role.trim().to_lowercase().as_str() {
        "admin" => "Admin".to_string(),
        "doctor" => "Doctor".to_string(),
        "pharmacist" => "Pharmacist".to_string(),
        _ => "Receptionist".to_string(),
    }
}

fn normalize_status(status: &str) -> String {
    if status.trim().eq_ignore_ascii_case("inactive") {
        "Inactive".to_string()
    } else {
        "Active".to_string()
    }
}

fn is_singapore_phone(phone: &str) -> bool {
    phone.len() == 8
        && phone.chars().all(|character| character.is_ascii_digit())
        && matches!(phone.as_bytes().first(), Some(b'6' | b'8' | b'9'))
}

fn is_valid_email(email: &str) -> bool {
    let Some((local, domain)) = email.split_once('@') else {
        return false;
    };

    !local.trim().is_empty()
        && domain.contains('.')
        && !domain.starts_with('.')
        && !domain.ends_with('.')
}
#[cfg(test)]
mod tests {
    use super::*;
    use crate::staff::repository::InMemoryStaffRepository;

    fn service() -> StaffService {
        StaffService::new(Arc::new(InMemoryStaffRepository::default()))
    }

    fn staff_form() -> StaffForm {
        StaffForm {
            firebase_uid: "firebase-user-1".to_string(),
            first_name: "Ada".to_string(),
            last_name: "Lovelace".to_string(),
            role: "Doctor".to_string(),
            phone: "91234567".to_string(),
            email: "ada@example.com".to_string(),
            status: Some("Active".to_string()),
        }
    }

    #[actix_web::test]
    async fn creates_staff_member() {
        let staff = service().create_staff(staff_form()).await.unwrap();

        assert!(staff.id.starts_with("STF-"));
        assert_eq!(staff.role, "Doctor");
        assert_eq!(staff.status, "Active");
    }

    #[actix_web::test]
    async fn rejects_missing_firebase_uid() {
        let mut form = staff_form();
        form.firebase_uid = " ".to_string();

        let error = service().create_staff(form).await.unwrap_err();

        assert_eq!(
            error,
            StaffError::InvalidInput("Firebase UID is required".to_string())
        );
    }

    #[actix_web::test]
    async fn updates_existing_staff_member() {
        let service = service();
        let staff = service.create_staff(staff_form()).await.unwrap();
        let mut form = staff_form();
        form.role = "Pharmacist".to_string();
        form.status = Some("Inactive".to_string());

        let updated = service.update_staff(&staff.id, form).await.unwrap();

        assert_eq!(updated.role, "Pharmacist");
        assert_eq!(updated.status, "Inactive");
    }
}
