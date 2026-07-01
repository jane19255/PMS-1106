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

#[derive(Serialize, Deserialize, Debug, Clone)]
pub struct SupabasePatientRow {
    pub id: String,
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
    pub status: String,
    pub created_at: Option<String>,
}

#[derive(Serialize, Deserialize, Debug, Clone)]
#[serde(rename_all = "camelCase")]
pub struct PatientView {
    pub id: String,
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
    pub status: String,
    pub created_at: Option<String>,
}

impl From<SupabasePatientRow> for PatientView {
    fn from(row: SupabasePatientRow) -> Self {
        Self {
            id: row.id,
            first_name: row.first_name,
            last_name: row.last_name,
            dob: row.dob,
            gender: row.gender,
            nric: row.nric,
            nationality: row.nationality,
            phone: row.phone,
            email: row.email,
            emergency_name: row.emergency_name,
            emergency_phone: row.emergency_phone,
            address: row.address,
            allergies: row.allergies,
            medications: row.medications,
            conditions: row.conditions,
            status: row.status,
            created_at: row.created_at,
        }
    }
}

#[derive(Deserialize, Debug)]
#[serde(rename_all = "camelCase")]
pub struct UpdatePatient {
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
    #[serde(default)]
    pub status: Option<String>,
}

#[derive(Serialize, Deserialize, Debug, Clone)]
#[serde(rename_all = "camelCase")]
pub struct CreateQueueEntryRequest {
    pub appointment_id: String,
    pub priority: Option<String>,
    pub priority_reason: Option<String>,
}

#[derive(Serialize, Deserialize, Debug, Clone)]
#[serde(rename_all = "camelCase")]
pub struct UpdateQueueStatusRequest {
    pub status: String,
}
