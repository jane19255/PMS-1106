use crate::db::SupabaseRestDb;
use chrono::NaiveDate;
use serde_json::json;

pub async fn get_appointments_by_date(
    db: &SupabaseRestDb,
    target_date: NaiveDate,
) -> Result<String, String> {
    let next_day = target_date.succ_opt().unwrap();

    let filter = format!(
        "select=*,patients(*),doctors(*,staff(*),room_status(*)),patient_queue(*)&scheduled_at=gte.{}T00:00:00Z&scheduled_at=lt.{}T00:00:00Z&order=scheduled_at.asc",
        target_date,
        next_day
    );

    let raw = db.fetch_table("appointments", &filter).await?;
    Ok(raw)
}

pub async fn enqueue_patient(
    db: &SupabaseRestDb,
    apt_id: &str,
    patient_id: &str,
    today: NaiveDate,
    priority: &str,
    priority_reason: Option<&str>,
) -> Result<String, String> {
    let payload = json!({
        "p_appointment_id": apt_id,
        "p_patient_id": patient_id,
        "p_queue_date": today.format("%Y-%m-%d").to_string(),
        "p_priority": priority,
        "p_priority_reason": priority_reason,
    });

    let url = db.rest_url("rpc/enqueue_patient");

    let res = db
        .authed(db.http_client.post(url))
        .header("Prefer", "return=representation")
        .json(&payload)
        .send()
        .await
        .map_err(|e| e.to_string())?;

    SupabaseRestDb::read_response(res).await
}

pub async fn save_vitals(
    db: &SupabaseRestDb,
    appointment_id: &str,
    bp: &str,
    temp: f32,
    pulse: i32,
    height: f32,
    weight: f32,
    priority: &str,
    priority_reason: Option<&str>,
) -> Result<String, String> {
    let payload = json!({
        "p_appointment_id": appointment_id,
        "p_bp": bp,
        "p_temp": temp,
        "p_pulse": pulse,
        "p_height": height,
        "p_weight": weight,
        "p_priority": priority,
        "p_priority_reason": priority_reason,
    });

    let url = db.rest_url("rpc/save_patient_vitals");

    let res = db
        .authed(db.http_client.post(url))
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
    let payload = json!({
        "p_appointment_id": appointment_id,
        "p_doctor_id": doctor_id,
    });
    let url = db.rest_url("rpc/send_patient_to_room");
    let res = db
        .authed(db.http_client.post(url))
        .header("Prefer", "return=representation")
        .json(&payload)
        .send()
        .await
        .map_err(|e| e.to_string())?;

    SupabaseRestDb::read_response(res).await
}

pub async fn get_room_statuses(db: &SupabaseRestDb) -> Result<String, String> {
    let filter = "select=*,doctors(*,staff(*))&order=room.asc";

    db.fetch_table("room_status", filter).await
}
