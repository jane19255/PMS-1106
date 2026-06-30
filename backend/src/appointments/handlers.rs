use actix_web::{web, HttpRequest, HttpResponse, Responder};
use chrono::{DateTime, Utc};
use firebase_auth::FirebaseAuth;
use serde_json::{json, Value};
use tera::{Context, Tera};
use uuid::Uuid;

use crate::appointments::interval::AppointmentInterval;
use crate::appointments::scheduler::AppointmentScheduler;
use crate::db::{FirebaseRestDb, SupabaseRestDb};
use crate::doctors::service::DoctorService;
use crate::handlers::auth::{require_auth_and_permission, AppAction};

const VALID_PRIORITIES: [&str; 4] = ["Emergency", "Urgent", "Normal", "Follow-up"];
const VALID_APPOINTMENT_TYPES: [&str; 4] =
    ["Routine Checkup", "Follow-up", "New Consultation", "Emergency"];

pub fn routes(cfg: &mut web::ServiceConfig) {
    // SSR page shell — the appointment list/table, stat cards, and modals are
    // populated client-side from the JSON API below (see assets/js/appointments.js).
    cfg.route("/appointments", web::get().to(appointments_page));

    // JSON API
    cfg.route("/api/appointments", web::get().to(list_appointments));
    cfg.route("/api/appointments", web::post().to(create_appointment));
    cfg.route("/api/appointments/{appointment_id}", web::get().to(get_appointment));
    cfg.route("/api/appointments/{appointment_id}", web::put().to(update_appointment));
    cfg.route("/api/appointments/{appointment_id}", web::delete().to(delete_appointment));
}

// ── SSR page shell ───────────────────────────────────────────────────────────

pub async fn appointments_page(
    req: HttpRequest,
    tera: web::Data<Tera>,
    firebase_auth: web::Data<FirebaseAuth>,
    firestore_db: web::Data<FirebaseRestDb>,
) -> impl Responder {
    if let Err(rejection) = require_auth_and_permission(
        &req,
        &firebase_auth,
        &firestore_db,
        AppAction::ManageAppointments,
    )
    .await
    {
        return rejection;
    }

    let mut ctx = Context::new();
    ctx.insert(
        "firebase_api_key",
        &std::env::var("FIREBASE_API_KEY").unwrap_or_default(),
    );
    ctx.insert(
        "firebase_project_id",
        &std::env::var("FIREBASE_PROJECT_ID").unwrap_or_default(),
    );

    match tera.render("Appointments.html", &ctx) {
        Ok(html) => HttpResponse::Ok().content_type("text/html; charset=utf-8").body(html),
        Err(e) => {
            eprintln!("Template error: {e}");
            HttpResponse::InternalServerError().body("Template rendering failed")
        }
    }
}

// ── Shared validation/conflict logic ─────────────────────────────────────────

enum AppointmentError {
    BadRequest(String),
    Conflict(Vec<(DateTime<Utc>, DateTime<Utc>)>),
    ServerError(String),
}

async fn validate_and_check_conflict(
    db: &SupabaseRestDb,
    doctor_service: &DoctorService,
    mut appointment: Value,
    exclude_id: Option<&str>,
) -> Result<Value, AppointmentError> {
    let Some(doctor_id) = appointment
        .get("doctor_id")
        .and_then(Value::as_str)
        .map(str::trim)
        .filter(|value| !value.is_empty())
        .map(str::to_string)
    else {
        return Err(AppointmentError::BadRequest("doctor_id is required".to_string()));
    };

    if doctor_service.find_doctor(&doctor_id).await.is_err() {
        return Err(AppointmentError::BadRequest("Selected doctor does not exist".to_string()));
    }

    let Some(patient_id) = appointment
        .get("patient_id")
        .and_then(Value::as_str)
        .map(str::trim)
        .filter(|value| !value.is_empty())
        .map(str::to_string)
    else {
        return Err(AppointmentError::BadRequest("patient_id is required".to_string()));
    };

    match db.get_patient(&patient_id).await {
        Ok(body) => {
            let rows: Vec<Value> = serde_json::from_str(&body).unwrap_or_default();
            if rows.is_empty() {
                return Err(AppointmentError::BadRequest("Selected patient does not exist".to_string()));
            }
        }
        Err(e) => return Err(AppointmentError::ServerError(e)),
    }

    normalize_appointment_payload(&mut appointment, &doctor_id);

    let scheduled_at = match appointment.get("scheduled_at").and_then(Value::as_str) {
        Some(s) if !s.trim().is_empty() => s.to_string(),
        _ => return Err(AppointmentError::BadRequest("scheduled_at is required".to_string())),
    };

    let requested_start = match DateTime::parse_from_rfc3339(&scheduled_at) {
        Ok(dt) => dt.with_timezone(&Utc),
        Err(_) => return Err(AppointmentError::BadRequest("Invalid scheduled_at format".to_string())),
    };

    let duration = appointment.get("duration_minutes").and_then(Value::as_i64).unwrap_or(30);
    if !(5..=480).contains(&duration) {
        return Err(AppointmentError::BadRequest(
            "duration_minutes must be between 5 and 480".to_string(),
        ));
    }

    let reason_ok = appointment
        .get("reason")
        .and_then(Value::as_str)
        .map(|s| !s.trim().is_empty())
        .unwrap_or(false);
    if !reason_ok {
        return Err(AppointmentError::BadRequest("reason is required".to_string()));
    }

    let priority = appointment
        .get("priority")
        .and_then(Value::as_str)
        .unwrap_or("Normal");
    if !VALID_PRIORITIES.contains(&priority) {
        return Err(AppointmentError::BadRequest(format!(
            "priority must be one of: {}",
            VALID_PRIORITIES.join(", ")
        )));
    }

    let appointment_type = appointment
        .get("appointment_type")
        .and_then(Value::as_str)
        .unwrap_or("Routine Checkup");
    if !VALID_APPOINTMENT_TYPES.contains(&appointment_type) {
        return Err(AppointmentError::BadRequest(format!(
            "appointment_type must be one of: {}",
            VALID_APPOINTMENT_TYPES.join(", ")
        )));
    }

    let appointments_json = db
        .list_appointments_by_doctor(&doctor_id)
        .await
        .map_err(AppointmentError::ServerError)?;

    let existing: Vec<Value> = serde_json::from_str(&appointments_json).unwrap_or_default();

    let mut intervals = Vec::new();
    for apt in existing {
        let Some(id) = apt.get("id").and_then(Value::as_str) else {
            continue;
        };
        if Some(id) == exclude_id {
            continue;
        }

        let Some(scheduled) = apt.get("scheduled_at").and_then(Value::as_str) else {
            continue;
        };

        let apt_duration = apt.get("duration_minutes").and_then(Value::as_i64).unwrap_or(30);

        if let Ok(start) = DateTime::parse_from_rfc3339(scheduled) {
            intervals.push(AppointmentInterval::new(
                id.to_string(),
                start.with_timezone(&Utc),
                apt_duration,
            ));
        }
    }

    let scheduler = AppointmentScheduler::new(intervals);
    let requested = AppointmentInterval::new(
        exclude_id.unwrap_or("").to_string(),
        requested_start,
        duration,
    );

    if scheduler.has_conflict(&requested) {
        return Err(AppointmentError::Conflict(scheduler.suggest_slots(duration)));
    }

    Ok(appointment)
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
    object.entry("priority").or_insert_with(|| json!("Normal"));
    object.entry("appointment_type").or_insert_with(|| json!("Routine Checkup"));
    object.entry("special_requirements").or_insert_with(|| json!([]));
}

fn conflict_json(suggestions: Vec<(DateTime<Utc>, DateTime<Utc>)>) -> Value {
    json!({
        "success": false,
        "message": "Appointment overlaps with existing booking.",
        "suggestions": suggestions
            .into_iter()
            .take(3)
            .map(|(s, e)| json!({ "start": s, "end": e }))
            .collect::<Vec<_>>()
    })
}

// ── JSON API handlers ────────────────────────────────────────────────────────

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
    match validate_and_check_conflict(&db, &doctor_service, payload.into_inner(), None).await {
        Ok(appointment) => match db.create_appointment(&appointment).await {
            Ok(result) => HttpResponse::Created().content_type("application/json").body(result),
            Err(err) => HttpResponse::InternalServerError().body(err),
        },
        Err(AppointmentError::BadRequest(msg)) => HttpResponse::BadRequest().body(msg),
        Err(AppointmentError::Conflict(suggestions)) => HttpResponse::Conflict().json(conflict_json(suggestions)),
        Err(AppointmentError::ServerError(err)) => HttpResponse::InternalServerError().body(err),
    }
}

pub async fn update_appointment(
    db: web::Data<SupabaseRestDb>,
    doctor_service: web::Data<DoctorService>,
    path: web::Path<String>,
    payload: web::Json<Value>,
) -> impl Responder {
    let appointment_id = path.into_inner();
    match validate_and_check_conflict(&db, &doctor_service, payload.into_inner(), Some(&appointment_id)).await {
        Ok(appointment) => match db.update_appointment(&appointment_id, &appointment).await {
            Ok(result) => HttpResponse::Ok().content_type("application/json").body(result),
            Err(err) => HttpResponse::InternalServerError().body(err),
        },
        Err(AppointmentError::BadRequest(msg)) => HttpResponse::BadRequest().body(msg),
        Err(AppointmentError::Conflict(suggestions)) => HttpResponse::Conflict().json(conflict_json(suggestions)),
        Err(AppointmentError::ServerError(err)) => HttpResponse::InternalServerError().body(err),
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
