use actix_web::http::header;
use actix_web::{web, HttpRequest, HttpResponse, Responder};
use chrono::{DateTime, Utc};
use firebase_auth::FirebaseAuth;
use serde::Deserialize;
use serde_json::{json, Value};
use tera::{Context, Tera};
use uuid::Uuid;

use crate::appointments::interval::AppointmentInterval;
use crate::appointments::scheduler::AppointmentScheduler;
use crate::db::{FirebaseRestDb, SupabaseRestDb};
use crate::doctors::service::DoctorService;
use crate::handlers::auth::{require_auth_and_permission, AppAction};

const APPOINTMENT_DETAILS_SELECT: &str =
    "select=*,patients(first_name,last_name,nric),doctors(specialty,staff(full_name))";

pub fn routes(cfg: &mut web::ServiceConfig) {
    // SSR pages
    cfg.route("/appointments", web::get().to(list_appointments_page));
    cfg.route("/appointments/new", web::get().to(new_appointment_page));
    cfg.route("/appointments", web::post().to(create_appointment_form));
    cfg.route("/appointments/{appointment_id}/edit", web::get().to(edit_appointment_page));
    cfg.route("/appointments/{appointment_id}", web::post().to(update_appointment_form));
    cfg.route("/appointments/{appointment_id}/delete", web::post().to(delete_appointment_form));

    // JSON API
    cfg.route("/api/appointments", web::get().to(list_appointments));
    cfg.route("/api/appointments", web::post().to(create_appointment));
    cfg.route("/api/appointments/{appointment_id}", web::get().to(get_appointment));
    cfg.route("/api/appointments/{appointment_id}", web::put().to(update_appointment));
    cfg.route("/api/appointments/{appointment_id}", web::delete().to(delete_appointment));
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

// ── SSR page handlers ────────────────────────────────────────────────────────

#[derive(Deserialize)]
pub struct AppointmentListQuery {
    pub date: Option<String>,
    pub doctor_id: Option<String>,
}

pub async fn list_appointments_page(
    req: HttpRequest,
    tera: web::Data<Tera>,
    firebase_auth: web::Data<FirebaseAuth>,
    firestore_db: web::Data<FirebaseRestDb>,
    db: web::Data<SupabaseRestDb>,
    query: web::Query<AppointmentListQuery>,
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

    let mut filter = APPOINTMENT_DETAILS_SELECT.to_string();

    if let Some(date) = query.date.as_deref().filter(|v| !v.trim().is_empty()) {
        filter.push_str(&format!(
            "&scheduled_at=gte.{date}T00:00:00Z&scheduled_at=lt.{date}T23:59:59Z"
        ));
    }

    if let Some(doctor_id) = query.doctor_id.as_deref().filter(|v| !v.trim().is_empty()) {
        filter.push_str(&format!("&doctor_id=eq.{}", urlencoding::encode(doctor_id)));
    }

    filter.push_str("&order=scheduled_at.asc");

    let appointments = match db.fetch_table("appointments", &filter).await {
        Ok(body) => serde_json::from_str::<Vec<Value>>(&body).unwrap_or_default(),
        Err(err) => return render_error(&tera, format!("Failed to load appointments: {err}")),
    };

    let mut ctx = Context::new();
    ctx.insert("appointments", &appointments);
    ctx.insert("filter_date", query.date.as_deref().unwrap_or(""));
    ctx.insert("filter_doctor_id", query.doctor_id.as_deref().unwrap_or(""));
    render_template(&tera, "appointments/index.html", &ctx)
}

#[derive(Deserialize)]
pub struct NewAppointmentQuery {
    pub patient_id: Option<String>,
    pub doctor_id: Option<String>,
}

pub async fn new_appointment_page(
    req: HttpRequest,
    tera: web::Data<Tera>,
    firebase_auth: web::Data<FirebaseAuth>,
    firestore_db: web::Data<FirebaseRestDb>,
    doctor_service: web::Data<DoctorService>,
    query: web::Query<NewAppointmentQuery>,
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

    let doctors = doctor_service.list_doctors().await.unwrap_or_default();

    let mut ctx = Context::new();
    ctx.insert("doctors", &doctors);
    ctx.insert("prefill_patient_id", query.patient_id.as_deref().unwrap_or(""));
    ctx.insert("prefill_doctor_id", query.doctor_id.as_deref().unwrap_or(""));
    ctx.insert("prefill_scheduled_at", "");
    ctx.insert("prefill_duration_minutes", "30");
    ctx.insert("prefill_reason", "");
    ctx.insert("prefill_notes", "");
    render_template(&tera, "appointments/new.html", &ctx)
}

#[derive(Deserialize, Clone)]
pub struct AppointmentFormInput {
    pub patient_id: String,
    pub doctor_id: String,
    pub scheduled_at: String,
    pub duration_minutes: Option<String>,
    pub reason: String,
    pub notes: Option<String>,
}

fn normalize_datetime_local(value: &str) -> String {
    let value = value.trim();
    if value.len() == 16 {
        format!("{value}:00Z")
    } else if value.len() == 19 && !value.ends_with('Z') {
        format!("{value}Z")
    } else {
        value.to_string()
    }
}

fn to_datetime_local_value(rfc3339: &str) -> String {
    DateTime::parse_from_rfc3339(rfc3339)
        .map(|dt| dt.format("%Y-%m-%dT%H:%M").to_string())
        .unwrap_or_default()
}

fn appointment_form_value(input: &AppointmentFormInput) -> Value {
    json!({
        "patient_id": input.patient_id.trim(),
        "doctor_id": input.doctor_id.trim(),
        "scheduled_at": normalize_datetime_local(&input.scheduled_at),
        "duration_minutes": input
            .duration_minutes
            .as_deref()
            .and_then(|s| s.trim().parse::<i64>().ok())
            .unwrap_or(30),
        "reason": input.reason.trim(),
        "notes": input.notes.as_deref().map(str::trim).filter(|s| !s.is_empty()),
    })
}

async fn render_appointment_form_error(
    tera: &Tera,
    doctor_service: &DoctorService,
    template: &str,
    appointment_id: Option<&str>,
    input: &AppointmentFormInput,
    error_message: String,
    suggestions: Vec<(DateTime<Utc>, DateTime<Utc>)>,
) -> HttpResponse {
    let doctors = doctor_service.list_doctors().await.unwrap_or_default();

    let mut ctx = Context::new();
    ctx.insert("doctors", &doctors);
    ctx.insert("prefill_patient_id", input.patient_id.trim());
    ctx.insert("prefill_doctor_id", input.doctor_id.trim());
    ctx.insert("prefill_scheduled_at", input.scheduled_at.trim());
    ctx.insert(
        "prefill_duration_minutes",
        input.duration_minutes.as_deref().unwrap_or("30"),
    );
    ctx.insert("prefill_reason", input.reason.trim());
    ctx.insert("prefill_notes", input.notes.as_deref().unwrap_or(""));
    ctx.insert("error_message", &error_message);
    if let Some(id) = appointment_id {
        ctx.insert("appointment_id", id);
    }

    if !suggestions.is_empty() {
        let formatted: Vec<Value> = suggestions
            .into_iter()
            .take(3)
            .map(|(s, e)| {
                json!({
                    "start": s.format("%d %b %Y, %H:%M").to_string(),
                    "end": e.format("%H:%M").to_string(),
                })
            })
            .collect();
        ctx.insert("suggestions", &formatted);
    }

    render_template(tera, template, &ctx)
}

pub async fn create_appointment_form(
    req: HttpRequest,
    tera: web::Data<Tera>,
    firebase_auth: web::Data<FirebaseAuth>,
    firestore_db: web::Data<FirebaseRestDb>,
    db: web::Data<SupabaseRestDb>,
    doctor_service: web::Data<DoctorService>,
    form: web::Form<AppointmentFormInput>,
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

    let input = form.into_inner();
    let value = appointment_form_value(&input);

    match validate_and_check_conflict(&db, &doctor_service, value, None).await {
        Ok(appointment) => match db.create_appointment(&appointment).await {
            Ok(_) => HttpResponse::SeeOther()
                .insert_header((header::LOCATION, "/appointments"))
                .finish(),
            Err(err) => {
                render_appointment_form_error(
                    &tera,
                    &doctor_service,
                    "appointments/new.html",
                    None,
                    &input,
                    format!("Failed to save appointment: {err}"),
                    Vec::new(),
                )
                .await
            }
        },
        Err(AppointmentError::BadRequest(msg)) => {
            render_appointment_form_error(&tera, &doctor_service, "appointments/new.html", None, &input, msg, Vec::new())
                .await
        }
        Err(AppointmentError::Conflict(suggestions)) => {
            render_appointment_form_error(
                &tera,
                &doctor_service,
                "appointments/new.html",
                None,
                &input,
                "Appointment overlaps with an existing booking for this doctor.".to_string(),
                suggestions,
            )
            .await
        }
        Err(AppointmentError::ServerError(err)) => {
            render_appointment_form_error(&tera, &doctor_service, "appointments/new.html", None, &input, err, Vec::new())
                .await
        }
    }
}

pub async fn edit_appointment_page(
    req: HttpRequest,
    tera: web::Data<Tera>,
    firebase_auth: web::Data<FirebaseAuth>,
    firestore_db: web::Data<FirebaseRestDb>,
    db: web::Data<SupabaseRestDb>,
    doctor_service: web::Data<DoctorService>,
    path: web::Path<String>,
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

    let appointment_id = path.into_inner();
    let body = match db.get_appointment(&appointment_id).await {
        Ok(body) => body,
        Err(err) => return render_error(&tera, format!("Failed to load appointment: {err}")),
    };

    let rows: Vec<Value> = serde_json::from_str(&body).unwrap_or_default();
    let Some(appointment) = rows.into_iter().next() else {
        return render_error(&tera, "Appointment was not found.".to_string());
    };

    let doctors = doctor_service.list_doctors().await.unwrap_or_default();

    let mut ctx = Context::new();
    ctx.insert("doctors", &doctors);
    ctx.insert("appointment_id", &appointment_id);
    ctx.insert(
        "prefill_patient_id",
        appointment.get("patient_id").and_then(Value::as_str).unwrap_or(""),
    );
    ctx.insert(
        "prefill_doctor_id",
        appointment.get("doctor_id").and_then(Value::as_str).unwrap_or(""),
    );
    let scheduled_at = appointment.get("scheduled_at").and_then(Value::as_str).unwrap_or("");
    ctx.insert("prefill_scheduled_at", &to_datetime_local_value(scheduled_at));
    ctx.insert(
        "prefill_duration_minutes",
        &appointment
            .get("duration_minutes")
            .and_then(Value::as_i64)
            .unwrap_or(30)
            .to_string(),
    );
    ctx.insert(
        "prefill_reason",
        appointment.get("reason").and_then(Value::as_str).unwrap_or(""),
    );
    ctx.insert(
        "prefill_notes",
        appointment.get("notes").and_then(Value::as_str).unwrap_or(""),
    );
    render_template(&tera, "appointments/edit.html", &ctx)
}

pub async fn update_appointment_form(
    req: HttpRequest,
    tera: web::Data<Tera>,
    firebase_auth: web::Data<FirebaseAuth>,
    firestore_db: web::Data<FirebaseRestDb>,
    db: web::Data<SupabaseRestDb>,
    doctor_service: web::Data<DoctorService>,
    path: web::Path<String>,
    form: web::Form<AppointmentFormInput>,
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

    let appointment_id = path.into_inner();
    let input = form.into_inner();
    let value = appointment_form_value(&input);

    match validate_and_check_conflict(&db, &doctor_service, value, Some(&appointment_id)).await {
        Ok(appointment) => match db.update_appointment(&appointment_id, &appointment).await {
            Ok(_) => HttpResponse::SeeOther()
                .insert_header((header::LOCATION, "/appointments"))
                .finish(),
            Err(err) => {
                render_appointment_form_error(
                    &tera,
                    &doctor_service,
                    "appointments/edit.html",
                    Some(&appointment_id),
                    &input,
                    format!("Failed to save appointment: {err}"),
                    Vec::new(),
                )
                .await
            }
        },
        Err(AppointmentError::BadRequest(msg)) => {
            render_appointment_form_error(
                &tera,
                &doctor_service,
                "appointments/edit.html",
                Some(&appointment_id),
                &input,
                msg,
                Vec::new(),
            )
            .await
        }
        Err(AppointmentError::Conflict(suggestions)) => {
            render_appointment_form_error(
                &tera,
                &doctor_service,
                "appointments/edit.html",
                Some(&appointment_id),
                &input,
                "Appointment overlaps with an existing booking for this doctor.".to_string(),
                suggestions,
            )
            .await
        }
        Err(AppointmentError::ServerError(err)) => {
            render_appointment_form_error(
                &tera,
                &doctor_service,
                "appointments/edit.html",
                Some(&appointment_id),
                &input,
                err,
                Vec::new(),
            )
            .await
        }
    }
}

pub async fn delete_appointment_form(
    req: HttpRequest,
    firebase_auth: web::Data<FirebaseAuth>,
    firestore_db: web::Data<FirebaseRestDb>,
    db: web::Data<SupabaseRestDb>,
    path: web::Path<String>,
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

    let appointment_id = path.into_inner();
    let _ = db.delete_appointment(&appointment_id).await;
    HttpResponse::SeeOther()
        .insert_header((header::LOCATION, "/appointments"))
        .finish()
}

// ── Helpers ──────────────────────────────────────────────────────────────────

fn render_template(tera: &Tera, template_name: &str, context: &Context) -> HttpResponse {
    match tera.render(template_name, context) {
        Ok(body) => HttpResponse::Ok().content_type("text/html").body(body),
        Err(error) => HttpResponse::InternalServerError()
            .content_type("text/plain")
            .body(format!("Template error: {error}")),
    }
}

fn render_error(tera: &Tera, message: String) -> HttpResponse {
    let mut context = Context::new();
    context.insert("message", &message);
    context.insert("back_url", "/appointments");
    context.insert("back_label", "Back to Appointments");
    render_template(tera, "error.html", &context)
}
