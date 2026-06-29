use serde::{Deserialize, Serialize};

#[derive(Clone, Debug, PartialEq, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct DoctorQueueAppointment {
    pub appointment_id: String,
    pub patient_id: String,
    pub patient_name: String,
    pub appointment_time: String,
    pub queue_status: String,
}

#[derive(Debug, Deserialize)]
pub struct AppointmentActionPayload {
    pub appointment_id: String,
}