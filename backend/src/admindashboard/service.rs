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

    let updated_raw = repository::mark_vitals_recorded(db, &payload.appointment_id).await?;
    let updated: Value = from_str(&updated_raw).unwrap_or(Value::Null);

    Ok(updated)
}