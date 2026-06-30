use crate::admindashboard::models::{MarkArrivedPayload, SaveVitalPayload};
use crate::admindashboard::repository;
use crate::db::SupabaseRestDb;
use chrono::NaiveDate;
use serde_json::{from_str, Value};

pub async fn mark_patient_arrived(
    db: &SupabaseRestDb,
    payload: MarkArrivedPayload,
    today: NaiveDate,
) -> Result<Value, String> {
    let apt_raw = repository::get_appointments_by_date(db, today).await?;
    let apt_list: Vec<Value> = from_str(&apt_raw).unwrap_or_default();

    let apt = apt_list
        .iter()
        .find(|a| a["id"].as_str() == Some(&payload.appointment_id))
        .ok_or("Appointment not found")?;

    if apt["status"].as_str() != Some("Scheduled") {
        return Err("Only scheduled appointments can be marked as arrived".to_string());
    }

    let patient_id = apt["patient_id"]
        .as_str()
        .ok_or("Appointment is missing patient_id")?;

    let queue_raw = repository::get_today_queue(db, today).await?;
    let queue_list: Vec<Value> = from_str(&queue_raw).unwrap_or_default();

    let exists = queue_list
        .iter()
        .any(|q| q["appointment_id"].as_str() == Some(&payload.appointment_id));

    if !exists {
        let queue_number = queue_list.len() as i32 + 1;

        repository::create_queue_entry(
            db,
            &payload.appointment_id,
            patient_id,
            today,
            queue_number,
        )
        .await?;
    }

    let updated_raw = repository::mark_patient_arrived(db, &payload.appointment_id).await?;
    let updated: Value = from_str(&updated_raw).unwrap_or(Value::Null);

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

    repository::record_vitals(
        db,
        &payload.appointment_id,
        &payload.bp,
        payload.temp,
        payload.pulse,
        payload.height,
        payload.weight,
    )
    .await?;

    repository::update_queue_priority(
        db,
        &payload.appointment_id,
        &priority,
        priority_reason.as_deref(),
    )
    .await?;

    let updated_raw = repository::mark_vitals_recorded(db, &payload.appointment_id).await?;
    let updated: Value = from_str(&updated_raw).unwrap_or(Value::Null);

    Ok(updated)
}