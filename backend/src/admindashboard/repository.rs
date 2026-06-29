use crate::db::SupabaseRestDb;
use chrono::{NaiveDate, Utc};
use serde_json::json;

pub async fn get_appointments_by_date(
    db: &SupabaseRestDb,
    target_date: NaiveDate,
) -> Result<String, String> {

    let next_day = target_date.succ_opt().unwrap();

    let filter = format!(
        "select=*,patients(*),doctors(*,staff(*),room_status(*))&scheduled_at=gte.{}T00:00:00Z&scheduled_at=lt.{}T00:00:00Z&order=scheduled_at.asc",
        target_date,
        next_day
    );

    let raw = db.fetch_table(
        "appointments",
        &filter
    ).await?;
    Ok(raw)
}

pub async fn get_today_queue(db: &SupabaseRestDb, today: NaiveDate) -> Result<String, String> {
    let query = format!(
        "patient_queue?select=*&queue_date=eq.{}",
        today.format("%Y-%m-%d")
    );
    let url = db.rest_url(&query);
    let res = db.authed(db.http_client.get(url))
        .send()
        .await
        .map_err(|e| e.to_string())?;
    SupabaseRestDb::read_response(res).await
}

pub async fn create_queue_entry(
    db: &SupabaseRestDb,
    apt_id: &str,
    patient_id: &str,
    today: NaiveDate,
    queue_number: i32,
) -> Result<String, String> {
    let payload = json!({
        "id": format!("Q{}", Utc::now().timestamp_millis()),
        "appointment_id": apt_id,
        "patient_id": patient_id,
        "queue_date": today.format("%Y-%m-%d").to_string(),
        "queue_number": queue_number,
        "status": "Waiting",
        "checked_in_at": Utc::now().to_rfc3339()
    });

    let url = db.rest_url("patient_queue");

    let res = db.authed(db.http_client.post(url))
        .header("Prefer", "return=representation")
        .json(&payload)
        .send()
        .await
        .map_err(|e| e.to_string())?;

    SupabaseRestDb::read_response(res).await
}

pub async fn mark_patient_arrived(
    db: &SupabaseRestDb,
    appointment_id: &str,
) -> Result<String, String> {
    let payload = json!({
        "status": "Checked In",
        "updated_at": Utc::now().to_rfc3339()
    });

    let filter = format!("id=eq.{}", urlencoding::encode(appointment_id));
    let url = db.rest_url(&format!("appointments?{}", filter));

    let res = db.authed(db.http_client.patch(url))
        .header("Prefer", "return=representation")
        .json(&payload)
        .send()
        .await
        .map_err(|e| e.to_string())?;

    SupabaseRestDb::read_response(res).await
}

pub async fn record_vitals(
    db: &SupabaseRestDb,
    appointment_id: &str,
    bp: &str,
    temp: f32,
    pulse: i32,
    height: f32,
    weight: f32,
) -> Result<String, String> {
    let payload = json!({
        "id": uuid::Uuid::new_v4().to_string(),
        "appointment_id": appointment_id,
        "bp": bp,
        "temp": temp,
        "pulse": pulse,
        "height": height,
        "weight": weight,
        "recorded_at": Utc::now().to_rfc3339()
    });

    let url = db.rest_url("patient_vitals");

    let res = db.authed(db.http_client.post(url))
        .header("Prefer", "return=representation")
        .json(&payload)
        .send()
        .await
        .map_err(|e| e.to_string())?;

    SupabaseRestDb::read_response(res).await
}

pub async fn mark_vitals_recorded(
    db: &SupabaseRestDb,
    appointment_id: &str,
) -> Result<String, String> {
    let payload = json!({
        "status": "Vitals Recorded",
        "updated_at": Utc::now().to_rfc3339()
    });

    let filter = format!("id=eq.{}", urlencoding::encode(appointment_id));
    let url = db.rest_url(&format!("appointments?{}", filter));

    let res = db.authed(db.http_client.patch(url))
        .header("Prefer", "return=representation")
        .json(&payload)
        .send()
        .await
        .map_err(|e| e.to_string())?;

    SupabaseRestDb::read_response(res).await
}

pub async fn send_to_room(
    db: &SupabaseRestDb,
    appointment_id: &str,
    doctor_id: &str,
) -> Result<String, String> {
    let now = Utc::now().to_rfc3339();

    let appointment_payload = json!({
        "status": "In Consultation",
        "updated_at": now
    });

    let apt_url = db.rest_url(&format!(
        "appointments?id=eq.{}",
        urlencoding::encode(appointment_id)
    ));

    db.authed(db.http_client.patch(apt_url))
        .header("Prefer", "return=representation")
        .json(&appointment_payload)
        .send()
        .await
        .map_err(|e| e.to_string())?;

    let queue_payload = json!({
        "status": "Called",
        "called_at": now,
        "updated_at": now
    });

    let queue_url = db.rest_url(&format!(
        "patient_queue?appointment_id=eq.{}",
        urlencoding::encode(appointment_id)
    ));

    db.authed(db.http_client.patch(queue_url))
        .header("Prefer", "return=representation")
        .json(&queue_payload)
        .send()
        .await
        .map_err(|e| e.to_string())?;

    let room_payload = json!({
        "status": "Occupied",
        "current_appointment_id": appointment_id,
        "updated_at": now
    });

    let room_url = db.rest_url(&format!(
        "room_status?doctor_id=eq.{}",
        urlencoding::encode(doctor_id)
    ));

    let res = db.authed(db.http_client.patch(room_url))
        .header("Prefer", "return=representation")
        .json(&room_payload)
        .send()
        .await
        .map_err(|e| e.to_string())?;

    SupabaseRestDb::read_response(res).await
}

pub async fn get_room_statuses(db: &SupabaseRestDb) -> Result<String, String> {
    let filter = "select=*,doctors(*,staff(*))&order=room.asc";

    db.fetch_table("room_status", filter).await
}