use serde::{Deserialize, Serialize};

#[derive(Clone, Debug, PartialEq, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct DashboardAppointment {
    pub appointment_id: String,
    pub priority: String,
    pub doctor: String,
    pub date: String,
    pub time: String,
    pub room: String,
    #[serde(rename = "type")]
    pub appointment_type: String,
    pub referring_provider: String,
    pub special_requirements: Vec<String>,
    pub status: String,
    pub patient: DashboardPatient,
}

#[derive(Clone, Debug, PartialEq, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct DashboardPatient {
    pub id: String,
    pub first_name: String,
    pub last_name: String,
    pub dob: String,
    pub gender: String,
    pub phone: String,
    pub email: String,
}