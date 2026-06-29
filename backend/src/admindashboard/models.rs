use chrono::{DateTime, Utc};
use serde::{Serialize, Deserialize};

#[derive(Debug, Serialize, Deserialize)]
pub struct DashboardAppointment {
    pub id: String,
    pub patient_id: String,
    pub patient_name: String,
    pub doctor_name: String,
    pub scheduled_at: DateTime<Utc>,
    pub status: String,
    pub priority: String,
}

#[derive(Debug, Deserialize)]
pub struct MarkArrivedPayload {
    pub appointment_id: String,
}

#[derive(Deserialize)]
pub struct SaveVitalPayload {
    pub appointment_id: String,
    pub bp: String,
    pub temp: f32,
    pub pulse: i32,
    pub height: f32,
    pub weight: f32,
}

#[derive(serde::Deserialize)]
pub struct SendToRoomPayload {
    pub appointment_id: String,
    pub doctor_id: String,
}