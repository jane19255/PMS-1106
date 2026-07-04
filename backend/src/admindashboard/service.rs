//! Admin dashboard service functions.
//!
//! Validates dashboard actions and coordinates repository updates for patient arrival
//! and vitals workflows.
use crate::admindashboard::models::{MarkArrivedPayload, SaveVitalPayload};
use crate::admindashboard::repository;
use crate::db::{singapore_offset, SupabaseRestDb};
use chrono::{DateTime, NaiveDate};
use serde_json::{from_str, Value};

pub async fn mark_patient_arrived(
    db: &SupabaseRestDb,
    payload: MarkArrivedPayload,
    today: NaiveDate,
) -> Result<Value, String> {
    let apt_raw = db.get_appointment(&payload.appointment_id).await?;
    let apt_list: Vec<Value> = from_str(&apt_raw).unwrap_or_default();

    let apt = apt_list.first().ok_or("Appointment not found")?;

    if apt["status"].as_str() != Some("Scheduled") {
        return Err("Only scheduled appointments can be marked as arrived".to_string());
    }

    let scheduled_date = apt["scheduled_at"]
        .as_str()
        .and_then(|value| DateTime::parse_from_rfc3339(value).ok())
        .map(|value| value.with_timezone(&singapore_offset()).date_naive());
    if scheduled_date != Some(today) {
        return Err("Only today's appointments can be marked as arrived".to_string());
    }

    let patient_id = apt["patient_id"]
        .as_str()
        .ok_or("Appointment is missing patient_id")?;

    // Supabase assigns the next number while holding a database lock, so two
    // receptionists cannot create the same queue number at the same time.
    let updated_raw = repository::enqueue_patient(
        db,
        &payload.appointment_id,
        patient_id,
        today,
        "Normal",
        None,
    )
    .await?;
    let updated: Value = from_str(&updated_raw)
        .map_err(|_| "Check-in succeeded but the queue entry response was unreadable".to_string())?;

    Ok(updated)
}

pub async fn save_patient_vitals(
    db: &SupabaseRestDb,
    payload: SaveVitalPayload,
) -> Result<Value, String> {
    if payload.temp < 35.0 || payload.temp > 42.0 {
        return Err("Temperature out of valid range (35-42°C)".to_string());
    }
    if payload.pulse < 40 || payload.pulse > 180 {
        return Err("Pulse out of valid range (40-180 bpm)".to_string());
    }
    if payload.height < 50.0 || payload.height > 250.0 {
        return Err("Height value invalid".to_string());
    }
    if payload.weight < 1.0 || payload.weight > 300.0 {
        return Err("Weight value invalid".to_string());
    }

    let priority = payload.priority.trim().to_string();

    if !matches!(priority.as_str(), "Normal" | "Urgent" | "Emergency") {
        return Err("Queue priority must be Normal, Urgent, or Emergency".to_string());
    }

    let priority_reason = payload
        .priority_reason
        .as_ref()
        .map(|reason| reason.trim().to_string())
        .filter(|reason| !reason.is_empty());

    if matches!(priority.as_str(), "Urgent" | "Emergency") && priority_reason.is_none() {
        return Err("Reason is required for urgent or emergency priority".to_string());
    }

    let updated_raw = repository::save_vitals(
        db,
        &payload.appointment_id,
        &payload.bp,
        payload.temp,
        payload.pulse,
        payload.height,
        payload.weight,
        &priority,
        priority_reason.as_deref(),
    )
    .await?;
    let updated: Value = from_str(&updated_raw)
        .map_err(|_| "Vitals saved but the response was unreadable".to_string())?;

    Ok(updated)
}
