//! Staff data models.
//!
//! Defines staff records and related serialized shapes used by staff management APIs.
use serde::{Deserialize, Serialize};

#[derive(Clone, Debug, PartialEq, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct StaffMember {
    pub id: String,
    pub firebase_uid: String,
    pub first_name: String,
    pub last_name: String,
    pub dob: String,
    pub gender: String,
    pub nric: String,
    pub role: String,
    pub phone: String,
    pub email: String,
    pub status: String,
    pub address: String,
    pub emergency: String,
}

impl StaffMember {
    pub fn full_name(&self) -> String {
        format!("{} {}", self.first_name, self.last_name)
            .trim()
            .to_string()
    }
}
