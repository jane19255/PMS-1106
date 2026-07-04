//! Admin dashboard request models.
//!
//! Defines payloads submitted by receptionist workflows when patients arrive, vitals
//! are saved, or patients are sent to consultation rooms.
use serde::Deserialize;

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
    pub priority: String,
    pub priority_reason: Option<String>,
}

#[derive(serde::Deserialize)]
pub struct SendToRoomPayload {
    pub appointment_id: String,
    pub doctor_id: String,
}
