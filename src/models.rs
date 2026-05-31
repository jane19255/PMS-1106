use serde::{Deserialize, Serialize};
use chrono::NaiveDate;

#[derive(Serialize, Deserialize, Debug, Clone)]
pub enum Gender {
    Male,
    Female,
    Other,
    Unknown,
}

#[derive(Serialize, Deserialize, Debug, Clone)]
pub struct Patient {
    pub first_name: String,
    pub last_name: String,
    pub dob: NaiveDate, // naivedate is a date without timezone, suitable for birthdates
    pub gender: Gender, 
    pub nric: String, 
    pub nationality: String, 
    pub phone: String, // standard practice is to store phone numbers as strings to preserve formatting and leading zeros
    pub email: String, 
    pub emergency_name: Option<String>,
    pub emergency_phone: Option<String>, 
    pub address: Option<String>,
    pub allergies: Option<String>,
    pub medications: Option<String>,
    pub conditions: Option<String>,
}

use chrono::{DateTime, Utc};
use uuid::Uuid;

#[derive(Serialize, Deserialize, Debug, Clone)]
pub struct Appointment {
    pub appointment_id: Uuid,
    pub patient_id: Uuid,
    pub doctor_id: Uuid,
    pub datetime: DateTime<Utc>, // When the appointment is supposed to happen.
    pub status: String,
    pub notes: Option<String>, // 'Option' for possibility that it may be empty or not provided.
    pub created_at: Option<DateTime<Utc>>, // When the appointment was created.
    pub updated_at: Option<DateTime<Utc>>, // When the appointment was last modified or updated.
}

impl Appointment {
    /// Validates the Appointment fields according to business rules.
    pub fn validate(&self) -> Result<(), String> {
        if self.status.trim().is_empty() {
            return Err("Status cannot be empty".into());
        }
        if self.datetime < Utc::now() {
            return Err("Appointment datetime cannot be in the past".into());
        }
        // Additional validations can be added here if needed.
        Ok(())
    }
}

#[derive(Serialize, Deserialize, Debug, Clone)]
pub struct QueueEntry {
    pub queue_id: Uuid,
    pub appointment_id: Uuid,
    pub patient_id: Uuid,
    pub doctor_id: Uuid,
    pub queue_number: u32, // sequential non-negative number (>= 0) assigned to each patient in the queue for a specific doctor on a given day.
    pub status: String,
}

impl QueueEntry {
    /// Validates the QueueEntry fields according to business rules.
    pub fn validate(&self) -> Result<(), String> {
        if self.status.trim().is_empty() {
            return Err("Status cannot be empty".into());
        }
        if self.queue_number == 0 {
            return Err("Queue number must be greater than zero".into());
        }
        // Additional validations can be added here if needed.
        Ok(())
    }
}