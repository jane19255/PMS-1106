use crate::db::{singapore_day_bounds, SupabaseRestDb};
use chrono::{NaiveDate, SecondsFormat};
use serde_json::json;

const APPOINTMENT_SELECT: &str =
    "select=*,patients(*),doctors(*,staff(*),room_status(*)),patient_queue(*)";

pub async fn get_appointments_by_date(
    db: &SupabaseRestDb,
    target_date: NaiveDate,
    today: NaiveDate,
) -> Result<String, String> {
    let (start, end) = singapore_day_bounds(target_date);
    let start = start.to_rfc3339_opts(SecondsFormat::Secs, true);
    let end = end.to_rfc3339_opts(SecondsFormat::Secs, true);

    let filter = if target_date == today {
        // Today's dashboard also carries forward patients who already checked in.
        format!(
            "{APPOINTMENT_SELECT}&or=(and(scheduled_at.gte.{start},scheduled_at.lt.{end}),and(scheduled_at.lt.{start},status.in.(Checked%20In,Vitals%20Recorded,In%20Consultation)))&order=scheduled_at.asc"
        )
    } else {
        format!(
            "{APPOINTMENT_SELECT}&scheduled_at=gte.{start}&scheduled_at=lt.{end}&order=scheduled_at.asc"
        )
    };

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

// These values map directly to the save_patient_vitals database function.
#[allow(clippy::too_many_arguments)]
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
