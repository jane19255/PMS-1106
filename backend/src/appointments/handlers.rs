use actix_web::{web, HttpResponse, Responder};
use serde_json::{json, Value};
use uuid::Uuid;
use chrono::{DateTime, Duration, Utc};

use crate::appointments::interval::AppointmentInterval;
use crate::appointments::scheduler::AppointmentScheduler;
use crate::db::SupabaseRestDb;
use crate::doctors::service::DoctorService;

pub async fn list_appointments(db: web::Data<SupabaseRestDb>) -> impl Responder {
    match db.list_appointments().await {
        Ok(result) => HttpResponse::Ok().content_type("application/json").body(result),
        Err(err) => HttpResponse::InternalServerError().body(err),
    }
}

pub async fn get_appointment(
    db: web::Data<SupabaseRestDb>,
    path: web::Path<String>,
) -> impl Responder {
    let appointment_id = path.into_inner();
    match db.get_appointment(&appointment_id).await {
        Ok(result) => HttpResponse::Ok().content_type("application/json").body(result),
        Err(err) => HttpResponse::InternalServerError().body(err),
    }
}

pub async fn create_appointment(
    db: web::Data<SupabaseRestDb>,
    doctor_service: web::Data<DoctorService>,
    payload: web::Json<Value>,
) -> impl Responder {
    let mut appointment = payload.into_inner();
    let Some(doctor_id) = appointment
        .get("doctor_id")
        .and_then(Value::as_str)
        .map(str::trim)
        .filter(|value| !value.is_empty())
        .map(str::to_string)
    else {
        return HttpResponse::BadRequest().body("doctor_id is required");
    };

    if doctor_service.find_doctor(&doctor_id).await.is_err() {
        return HttpResponse::BadRequest().body("Selected doctor does not exist");
    }

    normalize_appointment_payload(&mut appointment, &doctor_id);

    let appointments_json = match db.list_appointments_by_doctor(&doctor_id).await {
        Ok(v) => v,
        Err(e) => {
            return HttpResponse::InternalServerError().body(e)
        }
    };

    let existing: Vec<Value> = serde_json::from_str(&appointments_json).unwrap_or_default();

    let mut intervals = Vec::new();

    for apt in existing {
        let Some(scheduled) = apt.get("scheduled_at").and_then(Value::as_str) else {
            continue;
        };

        let duration = apt.get("duration_minutes").and_then(Value::as_i64).unwrap_or(30);

        if let Ok(start) = DateTime::parse_from_rfc3339(scheduled) {
            intervals.push(
                AppointmentInterval::new(
                    apt.get("id").and_then(Value::as_str).unwrap_or("").to_string(),
                    start.with_timezone(&Utc),
                    duration,
                ),
            );
        }
    }

    let scheduler = AppointmentScheduler::new(intervals);

    let scheduled_at = appointment["scheduled_at"].as_str().unwrap();

    let duration = appointment["duration_minutes"].as_i64().unwrap_or(30);

    let requested_start = DateTime::parse_from_rfc3339(scheduled_at)
        .unwrap()
        .with_timezone(&Utc);

    let requested = AppointmentInterval::new("".to_string(), requested_start, duration);

    if scheduler.has_conflict(&requested) {
        let suggestions = scheduler.suggest_slots(duration);

        return HttpResponse::Conflict().json(
            json!({
                "success": false,
                "message": "Appointment overlaps with existing booking.",
                "suggestions": suggestions
                    .into_iter()
                    .take(3)
                    .map(|(s,e)| {
                        json!({
                            "start": s,
                            "end": e
                        })
                    })
                    .collect::<Vec<_>>()
            }),
        );
    }

    match db.create_appointment(&appointment).await {
        Ok(result) => {
            HttpResponse::Created()
                .content_type("application/json")
                .body(result)
        }
        Err(err) => {
            HttpResponse::InternalServerError().body(err)
        }
    }
}

pub async fn update_appointment(
    db: web::Data<SupabaseRestDb>,
    path: web::Path<String>,
    payload: web::Json<Value>,
) -> impl Responder {
    let appointment_id = path.into_inner();
    let mut appointment = payload.into_inner();

    let Some(doctor_id) = appointment
        .get("doctor_id")
        .and_then(Value::as_str)
        .map(str::trim)
        .filter(|value| !value.is_empty())
        .map(str::to_string)
    else {
        return HttpResponse::BadRequest().body("doctor_id is required");
    };

    normalize_appointment_payload(&mut appointment, &doctor_id);

    let appointments_json = match db.list_appointments_by_doctor(&doctor_id).await {
        Ok(v) => v,
        Err(e) => {
            return HttpResponse::InternalServerError().body(e);
        }
    };

    let existing: Vec<Value> = serde_json::from_str(&appointments_json).unwrap_or_default();

    let mut intervals = Vec::new();

    for apt in existing {
        let Some(id) = apt.get("id").and_then(Value::as_str) else {
            continue;
        };
        if id == appointment_id {
            continue;
        }

        let Some(scheduled) = apt.get("scheduled_at").and_then(Value::as_str) else {
            continue;
        };

        let duration = apt.get("duration_minutes").and_then(Value::as_i64).unwrap_or(30);

        if let Ok(start) = DateTime::parse_from_rfc3339(scheduled) {
            intervals.push(
                AppointmentInterval::new(
                    id.to_string(),
                    start.with_timezone(&Utc),
                    duration,
                ),
            );
        }
    }

    let scheduler = AppointmentScheduler::new(intervals);

    let scheduled_at = match appointment.get("scheduled_at").and_then(Value::as_str) {
        Some(s) => s,
        None => return HttpResponse::BadRequest().body("scheduled_at is required"),
    };

    let duration = appointment.get("duration_minutes").and_then(Value::as_i64).unwrap_or(30);

    let requested_start = match DateTime::parse_from_rfc3339(scheduled_at) {
        Ok(dt) => dt.with_timezone(&Utc),
        Err(_) => return HttpResponse::BadRequest().body("Invalid scheduled_at format"),
    };

    let requested = AppointmentInterval::new("".to_string(), requested_start, duration);

    if scheduler.has_conflict(&requested) {
        let suggestions = scheduler.suggest_slots(duration);

        return HttpResponse::Conflict().json(
            json!({
                "success": false,
                "message": "Appointment overlaps with existing booking.",
                "suggestions": suggestions
                    .into_iter()
                    .take(3)
                    .map(|(s,e)| {
                        json!({
                            "start": s,
                            "end": e
                        })
                    })
                    .collect::<Vec<_>>()
            }),
        );
    }

    match db.update_appointment(&appointment_id, &appointment).await {
        Ok(result) => HttpResponse::Ok().content_type("application/json").body(result),
        Err(err) => HttpResponse::InternalServerError().body(err),
    }
}

pub async fn delete_appointment(
    db: web::Data<SupabaseRestDb>,
    path: web::Path<String>,
) -> impl Responder {
    let appointment_id = path.into_inner();
    match db.delete_appointment(&appointment_id).await {
        Ok(result) => HttpResponse::Ok().content_type("application/json").body(result),
        Err(err) => HttpResponse::InternalServerError().body(err),
    }
}

fn normalize_appointment_payload(appointment: &mut Value, doctor_id: &str) {
    let Some(object) = appointment.as_object_mut() else {
        *appointment = json!({});
        return normalize_appointment_payload(appointment, doctor_id);
    };

    object.entry("id").or_insert_with(|| json!(format!("APT-{}", Uuid::new_v4())));
    object.insert("doctor_id".to_string(), json!(doctor_id));

    if !object.contains_key("scheduled_at") {
        if let Some(appointment_datetime) = object.remove("appointment_datetime") {
            object.insert("scheduled_at".to_string(), appointment_datetime);
        }
    }

    object.entry("reason").or_insert_with(|| json!("Appointment"));
}