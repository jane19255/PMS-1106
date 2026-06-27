use serde::{Deserialize, Serialize};

#[derive(Clone, Debug, PartialEq, Serialize, Deserialize)]
pub struct Doctor {
    pub id: String,
    pub license_number: String,
    pub name: String,
    pub specialization: String,
    pub contact_number: String,
    pub email: String,
    pub status: DoctorStatus,
}

#[derive(Clone, Debug, PartialEq, Serialize, Deserialize)]
pub enum DoctorStatus {
    Available,
    Unavailable,
}

#[derive(Clone, Debug, PartialEq, Serialize, Deserialize)]
pub struct DoctorSchedule {
    pub id: String,
    pub doctor_id: String,
    pub day_of_week: DayOfWeek,
    pub start_time: String,
    pub end_time: String,
}

#[derive(Clone, Debug, PartialEq, Serialize, Deserialize)]
pub enum DayOfWeek {
    Monday,
    Tuesday,
    Wednesday,
    Thursday,
    Friday,
    Saturday,
    Sunday,
}
