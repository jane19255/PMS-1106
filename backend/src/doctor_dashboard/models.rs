use serde::{Deserialize, Serialize};

#[derive(Clone, Debug, PartialEq, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct DoctorQueueAppointment {
    pub appointment_id: String,
    pub patient_id: String,
    pub patient_name: String,
    pub appointment_date: String,
    pub appointment_time: String,
    pub queue_status: String,
    pub priority: String,
    // RFC 3339 timestamp; the consultation must end by this time unless extended.
    pub consultation_deadline: Option<String>,
}

#[derive(Debug, Deserialize)]
pub struct AppointmentActionPayload {
    pub appointment_id: String,
}
