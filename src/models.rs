use serde::{Deserialize, Serialize};

#[derive(Serialize, Deserialize, Debug, Clone)]
pub struct Patient {
    pub first_name: String,
    pub last_name: String,
    pub dob: String,
    pub gender: String,
    pub nric: String,
    pub nationality: String,
    pub phone: String,
    pub email: String,
    pub emergency_name: Option<String>,
    pub emergency_phone: Option<String>,
    pub address: Option<String>,
    pub allergies: Option<String>,
    pub medications: Option<String>,
    pub conditions: Option<String>,
}