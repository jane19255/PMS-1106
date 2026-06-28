use chrono::{DateTime, Utc};
use serde::{Deserialize, Serialize};

#[derive(Clone, Debug, PartialEq, Serialize, Deserialize)]
pub struct Prescription {
    pub id: String,
    pub patient_id: String,
    pub doctor_id: String,
    pub medical_record_id: Option<String>,
    pub medication_name: String,
    pub dosage: String,
    pub frequency: String,
    pub duration: String,
    pub instructions: Option<String>,
    pub quantity: u32,
    pub unit_cost: f64,
    pub status: PrescriptionStatus,
    pub issued_at: DateTime<Utc>,
    pub dispensed_at: Option<DateTime<Utc>>,
    pub cancelled_at: Option<DateTime<Utc>>,
}

#[derive(Clone, Debug, PartialEq, Serialize, Deserialize)]
pub enum PrescriptionStatus {
    Active,
    Dispensed,
    Cancelled,
}
