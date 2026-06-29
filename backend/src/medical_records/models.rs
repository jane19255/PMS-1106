use chrono::{DateTime, Utc};
use serde::{Deserialize, Serialize};

#[derive(Clone, Debug, PartialEq, Serialize, Deserialize)]
pub struct MedicalRecord {
    pub id: String,
    pub patient_id: String,
    pub appointment_id: Option<String>,
    pub doctor_id: Option<String>,
    pub reason_of_visit: Option<String>,
    pub clinical_findings: Option<String>,
    pub diagnosis: Option<String>,
    pub treatment_plan: Option<String>,
    pub doctor_notes: Option<String>,
    pub blood_pressure: Option<String>,
    pub temperature: Option<String>,
    pub pulse_rate: Option<String>,
    pub height_cm: Option<String>,
    pub weight_kg: Option<String>,
    pub recorded_at: DateTime<Utc>,
    pub created_at: DateTime<Utc>,
    pub updated_at: DateTime<Utc>,
}

#[derive(Clone, Debug, Serialize, Deserialize)]
pub struct PatientTimelineEntry {
    pub record: MedicalRecord,
    pub doctor_name: Option<String>,
}
