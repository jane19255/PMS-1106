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
}

// ====================
// Appointment Models 
// ====================
#[derive(Serialize, Deserialize, Debug, Clone)]
pub struct Appointment {
    pub patient_id: String,
    pub doctor_id: String,
    pub appointment_datetime: String,
    pub status: String,
    pub queue_number: Option<i32>,
    pub medical_record_id: Option<String>,
    pub notes: Option<String>,
}

#[derive(Serialize, Deserialize, Debug, Clone)]
pub struct SupabaseAppointmentRow {
    pub id: String,
    pub patient_id: String,
    pub doctor_id: String,
    pub appointment_datetime: String,
    pub status: String,
    pub queue_number: Option<i32>,
    pub medical_record_id: Option<String>,
    pub notes: Option<String>,
    pub created_at: Option<String>,
}

#[derive(Serialize, Deserialize, Debug, Clone)]
#[serde(rename_all = "camelCase")]
pub struct AppointmentView {
    pub id: String,
    pub patient_id: String,
    pub doctor_id: String,
    pub appointment_datetime: String,
    pub status: String,
    pub queue_number: Option<i32>,
    pub medical_record_id: Option<String>,
    pub notes: Option<String>,
    pub created_at: Option<String>,
}

impl From<SupabaseAppointmentRow> for AppointmentView {
    fn from(row: SupabaseAppointmentRow) -> Self {
        Self {
            id: row.id,
            patient_id: row.patient_id,
            doctor_id: row.doctor_id,
            appointment_datetime: row.appointment_datetime,
            status: row.status,
            queue_number: row.queue_number,
            medical_record_id: row.medical_record_id,
            notes: row.notes,
            created_at: row.created_at,
        }
    }
}

// ========================
// Queue Management Models 
// ========================
#[derive(Serialize, Deserialize, Debug, Clone)]
pub struct QueueEntry {
    pub appointment_id: String,
    pub patient_id: String,
    pub doctor_id: String,
    pub priority: i32, // 1=Emergency, 2=Urgent, 3=Normal, 4=Follow-up
    pub status: String, // Waiting, InProgress, Completed, Cancelled, etc.
    pub queue_position: Option<i32>,
    pub checked_in_at: Option<String>,
    pub called_at: Option<String>,
    pub completed_at: Option<String>,
}

#[derive(Serialize, Deserialize, Debug, Clone)]
pub struct SupabaseQueueRow {
    pub id: String,
    pub appointment_id: String,
    pub patient_id: String,
    pub doctor_id: String,
    pub priority: i32,
    pub status: String,
    pub queue_position: Option<i32>,
    pub checked_in_at: Option<String>,
    pub called_at: Option<String>,
    pub completed_at: Option<String>,
    pub created_at: Option<String>,
    pub updated_at: Option<String>,
}

#[derive(Serialize, Deserialize, Debug, Clone)]
#[serde(rename_all = "camelCase")]
pub struct QueueView {
    pub id: String,
    pub appointment_id: String,
    pub patient_id: String,
    pub doctor_id: String,
    pub priority: i32,
    pub status: String,
    pub queue_position: Option<i32>,
    pub checked_in_at: Option<String>,
    pub called_at: Option<String>,
    pub completed_at: Option<String>,
    pub created_at: Option<String>,
    pub updated_at: Option<String>,
}

impl From<SupabaseQueueRow> for QueueView {
    fn from(row: SupabaseQueueRow) -> Self {
        Self {
            id: row.id,
            appointment_id: row.appointment_id,
            patient_id: row.patient_id,
            doctor_id: row.doctor_id,
            priority: row.priority,
            status: row.status,
            queue_position: row.queue_position,
            checked_in_at: row.checked_in_at,
            called_at: row.called_at,
            completed_at: row.completed_at,
            created_at: row.created_at,
            updated_at: row.updated_at,
        }
    }
}

#[derive(Serialize, Deserialize, Debug, Clone)]
#[serde(rename_all = "camelCase")]
pub struct CreateQueueEntryRequest {
    pub appointment_id: String,
    pub patient_id: String,
    pub doctor_id: String,
    pub priority: i32,
}

#[derive(Serialize, Deserialize, Debug, Clone)]
#[serde(rename_all = "camelCase")]
pub struct UpdateQueueStatusRequest {
    pub status: String,
}

// ===========================
// Doctor Availability Models 
// ===========================
#[derive(Serialize, Deserialize, Debug, Clone)]
pub struct DoctorAvailability {
    pub doctor_id: String,
    pub available_from: String,
    pub available_to: String,
    pub is_available: bool,
}

#[derive(Serialize, Deserialize, Debug, Clone)]
pub struct SupabaseDoctorAvailabilityRow {
    pub id: String,
    pub doctor_id: String,
    pub available_from: String,
    pub available_to: String,
    pub is_available: bool,
    pub created_at: Option<String>,
    pub updated_at: Option<String>,
}

#[derive(Serialize, Deserialize, Debug, Clone)]
#[serde(rename_all = "camelCase")]
pub struct DoctorAvailabilityView {
    pub id: String,
    pub doctor_id: String,
    pub available_from: String,
    pub available_to: String,
    pub is_available: bool,
    pub created_at: Option<String>,
    pub updated_at: Option<String>,
}

impl From<SupabaseDoctorAvailabilityRow> for DoctorAvailabilityView {
    fn from(row: SupabaseDoctorAvailabilityRow) -> Self {
        Self {
            id: row.id,
            doctor_id: row.doctor_id,
            available_from: row.available_from,
            available_to: row.available_to,
            is_available: row.is_available,
            created_at: row.created_at,
            updated_at: row.updated_at,
        }
    }
}