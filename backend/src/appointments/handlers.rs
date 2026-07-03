use actix_web::{web, HttpRequest, HttpResponse, Responder};
use chrono::{DateTime, Datelike, Duration, NaiveTime, Utc, Weekday};
use firebase_auth::FirebaseAuth;
use serde_json::{json, Value};
use tera::{Context, Tera};
use uuid::Uuid;

use crate::appointments::interval::AppointmentInterval;
use crate::appointments::scheduler::AppointmentScheduler;
use crate::db::{singapore_offset, singapore_today, FirebaseRestDb, SupabaseRestDb};
use crate::doctors::models::{DayOfWeek, DoctorSchedule, DoctorStatus};
use crate::doctors::service::DoctorService;
use crate::handlers::auth::{require_auth_and_permission, AppAction};

pub fn routes(cfg: &mut web::ServiceConfig) {
    // SSR page shell — the appointment list/table, stat cards, and modals are
    // populated client-side from the JSON API below (see assets/js/appointments.js).
    cfg.route("/appointments", web::get().to(appointments_page));

    // JSON API
    cfg.route("/api/appointments", web::get().to(list_appointments));
    cfg.route("/api/appointments", web::post().to(create_appointment));
    cfg.route(
        "/api/appointments/{appointment_id}",
        web::get().to(get_appointment),
    );
    cfg.route(
        "/api/appointments/{appointment_id}",
        web::put().to(update_appointment),
    );
    cfg.route(
        "/api/appointments/{appointment_id}",
        web::delete().to(delete_appointment),
    );
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

    let ctx = Context::new();

    match tera.render("Appointments.html", &ctx) {
        Ok(html) => HttpResponse::Ok()
            .content_type("text/html; charset=utf-8")
            .body(html),
        Err(e) => {
            eprintln!("Template error: {e}");
            HttpResponse::InternalServerError().body("Template rendering failed")
        }
    }
}

// ── Shared validation/conflict logic ─────────────────────────────────────────

pub(crate) enum AppointmentError {
    BadRequest(String),
    Conflict(Vec<(DateTime<Utc>, DateTime<Utc>)>),
    ServerError(String),
}

pub(crate) async fn validate_and_check_conflict(
    db: &SupabaseRestDb,
    doctor_service: &DoctorService,
    mut appointment: Value,
    exclude_id: Option<&str>,
) -> Result<Value, AppointmentError> {
    // Check the linked records before checking the requested time slot.
    let Some(doctor_id) = appointment
        .get("doctor_id")
        .and_then(Value::as_str)
        .map(str::trim)
        .filter(|value| !value.is_empty())
        .map(str::to_string)
    else {
        return Err(AppointmentError::BadRequest(
            "doctor_id is required".to_string(),
        ));
    };

    let doctor = doctor_service.find_doctor(&doctor_id).await.map_err(|_| {
        AppointmentError::BadRequest("Selected doctor does not exist".to_string())
    })?;
    if doctor.status != DoctorStatus::Available {
        return Err(AppointmentError::BadRequest(
            "Selected doctor is not currently available".to_string(),
        ));
    }

    let Some(patient_id) = appointment
        .get("patient_id")
        .and_then(Value::as_str)
        .map(str::trim)
        .filter(|value| !value.is_empty())
        .map(str::to_string)
    else {
        return Err(AppointmentError::BadRequest(
            "patient_id is required".to_string(),
        ));
    };

    match db.get_patient(&patient_id).await {
        Ok(body) => {
            let rows: Vec<Value> = serde_json::from_str(&body).unwrap_or_default();
            if rows.is_empty() {
                return Err(AppointmentError::BadRequest(
                    "Selected patient does not exist".to_string(),
                ));
            }
        }
        Err(e) => return Err(AppointmentError::ServerError(e)),
    }

    normalize_appointment_payload(&mut appointment, &doctor_id, exclude_id.is_none());

    let scheduled_at = match appointment.get("scheduled_at").and_then(Value::as_str) {
        Some(s) if !s.trim().is_empty() => s.to_string(),
        _ => {
            return Err(AppointmentError::BadRequest(
                "scheduled_at is required".to_string(),
            ))
        }
    };

    let requested_start = match DateTime::parse_from_rfc3339(&scheduled_at) {
        Ok(dt) => dt.with_timezone(&Utc),
        Err(_) => {
            return Err(AppointmentError::BadRequest(
                "Invalid scheduled_at format".to_string(),
            ))
        }
    };
    if requested_start < Utc::now() {
        return Err(AppointmentError::BadRequest(
            "Appointments cannot be scheduled in the past".to_string(),
        ));
    }

    let duration = appointment
        .get("duration_minutes")
        .and_then(Value::as_i64)
        .unwrap_or(30);
    if !(5..=480).contains(&duration) {
        return Err(AppointmentError::BadRequest(
            "duration_minutes must be between 5 and 480".to_string(),
        ));
    }

    let schedules = doctor_service
        .list_schedules(&doctor_id)
        .await
        .map_err(|_| AppointmentError::ServerError("Could not load doctor schedule".to_string()))?;
    if !appointment_fits_schedule(requested_start, duration, &schedules) {
        return Err(AppointmentError::BadRequest(
            "Selected time is outside the doctor's working schedule".to_string(),
        ));
    }

    let reason_ok = appointment
        .get("reason")
        .and_then(Value::as_str)
        .map(|s| !s.trim().is_empty())
        .unwrap_or(false);
    if !reason_ok {
        return Err(AppointmentError::BadRequest(
            "reason is required".to_string(),
        ));
    }

    // Load this doctor's bookings so the same time cannot be booked twice.
    let appointments_json = db
        .list_appointments_by_doctor(&doctor_id)
        .await
        .map_err(AppointmentError::ServerError)?;

    let existing: Vec<Value> = serde_json::from_str(&appointments_json).unwrap_or_default();

    let mut intervals = Vec::new();
    for apt in existing {
        // Cancelled and no-show appointments no longer reserve their time slot.
        if !appointment_blocks_time_slot(&apt) {
            continue;
        }

        let Some(id) = apt.get("id").and_then(Value::as_str) else {
            continue;
        };
        if Some(id) == exclude_id {
            continue;
        }

        let Some(scheduled) = apt.get("scheduled_at").and_then(Value::as_str) else {
            continue;
        };

        let apt_duration = apt
            .get("duration_minutes")
            .and_then(Value::as_i64)
            .unwrap_or(30);

        if let Ok(start) = DateTime::parse_from_rfc3339(scheduled) {
            intervals.push(AppointmentInterval::new(
                start.with_timezone(&Utc),
                apt_duration,
            ));
        }
    }

    // The interval tree finds an overlap without checking every pair manually.
    let scheduler = AppointmentScheduler::new(intervals);
    let requested = AppointmentInterval::new(requested_start, duration);

    if scheduler.has_conflict(&requested) {
        let suggestions = scheduler
            .suggest_slots(duration)
            .into_iter()
            .filter(|(start, _)| appointment_fits_schedule(*start, duration, &schedules))
            .collect();
        return Err(AppointmentError::Conflict(suggestions));
    }

    Ok(appointment)
}

fn appointment_fits_schedule(
    start_utc: DateTime<Utc>,
    duration_minutes: i64,
    schedules: &[DoctorSchedule],
) -> bool {
    let local_start = start_utc.with_timezone(&singapore_offset());
    let local_end = local_start + Duration::minutes(duration_minutes);

    // Doctors without a custom schedule use the clinic's normal opening hours.
    if schedules.is_empty() {
        let morning_start = NaiveTime::from_hms_opt(8, 0, 0).unwrap();
        let morning_end = NaiveTime::from_hms_opt(12, 0, 0).unwrap();
        let afternoon_start = NaiveTime::from_hms_opt(13, 0, 0).unwrap();
        let clinic_close = NaiveTime::from_hms_opt(19, 0, 0).unwrap();
        return local_start.date_naive() == local_end.date_naive()
            && ((local_start.time() >= morning_start && local_end.time() <= morning_end)
                || (local_start.time() >= afternoon_start && local_end.time() <= clinic_close));
    }

    // A booking must start and finish within one configured working period.
    schedules.iter().any(|schedule| {
        if weekday_for_schedule(&schedule.day_of_week) != local_start.weekday() {
            return false;
        }
        let Some(schedule_start) = parse_schedule_time(&schedule.start_time) else {
            return false;
        };
        let Some(schedule_end) = parse_schedule_time(&schedule.end_time) else {
            return false;
        };

        local_start.date_naive() == local_end.date_naive()
            && local_start.time() >= schedule_start
            && local_end.time() <= schedule_end
    })
}

fn parse_schedule_time(value: &str) -> Option<NaiveTime> {
    NaiveTime::parse_from_str(value, "%H:%M:%S")
        .or_else(|_| NaiveTime::parse_from_str(value, "%H:%M"))
        .ok()
}

fn weekday_for_schedule(day: &DayOfWeek) -> Weekday {
    match day {
        DayOfWeek::Monday => Weekday::Mon,
        DayOfWeek::Tuesday => Weekday::Tue,
        DayOfWeek::Wednesday => Weekday::Wed,
        DayOfWeek::Thursday => Weekday::Thu,
        DayOfWeek::Friday => Weekday::Fri,
        DayOfWeek::Saturday => Weekday::Sat,
        DayOfWeek::Sunday => Weekday::Sun,
    }
}

fn appointment_blocks_time_slot(appointment: &Value) -> bool {
    !matches!(
        appointment.get("status").and_then(Value::as_str),
        Some("Cancelled" | "No Show")
    )
}

fn normalize_appointment_payload(appointment: &mut Value, doctor_id: &str, is_new: bool) {
    let Some(object) = appointment.as_object_mut() else {
        *appointment = json!({});
        return normalize_appointment_payload(appointment, doctor_id, is_new);
    };

    if is_new {
        object
            .entry("id")
            .or_insert_with(|| json!(format!("APT-{}", Uuid::new_v4())));
    } else {
        // The URL identifies the existing row, so an edit must not replace its ID.
        object.remove("id");
    }
    // Workflow status is changed only by the check-in and consultation actions.
    object.remove("status");
    object.remove("created_at");
    object.remove("updated_at");
    object.remove("scheduled_end_at");
    object.insert("doctor_id".to_string(), json!(doctor_id));

    if !object.contains_key("scheduled_at") {
        if let Some(appointment_datetime) = object.remove("appointment_datetime") {
            object.insert("scheduled_at".to_string(), appointment_datetime);
        }
    }

    object
        .entry("reason")
        .or_insert_with(|| json!("Appointment"));
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

pub async fn list_appointments(
    req: HttpRequest,
    db: web::Data<SupabaseRestDb>,
    firebase_auth: web::Data<FirebaseAuth>,
    firestore_db: web::Data<FirebaseRestDb>,
) -> impl Responder {
    if let Err(rejection) =
        require_appointments_permission(&req, &firebase_auth, &firestore_db).await
    {
        return rejection;
    }
    if let Err(error) = db.reconcile_overdue_appointments(singapore_today()).await {
        return HttpResponse::InternalServerError().body(error);
    }
    match db.list_appointments().await {
        Ok(result) => HttpResponse::Ok()
            .content_type("application/json")
            .body(result),
        Err(err) => HttpResponse::InternalServerError().body(err),
    }
}

pub async fn get_appointment(
    req: HttpRequest,
    db: web::Data<SupabaseRestDb>,
    path: web::Path<String>,
    firebase_auth: web::Data<FirebaseAuth>,
    firestore_db: web::Data<FirebaseRestDb>,
) -> impl Responder {
    if let Err(rejection) =
        require_appointments_permission(&req, &firebase_auth, &firestore_db).await
    {
        return rejection;
    }
    let appointment_id = path.into_inner();
    match db.get_appointment(&appointment_id).await {
        Ok(result) => HttpResponse::Ok()
            .content_type("application/json")
            .body(result),
        Err(err) => HttpResponse::InternalServerError().body(err),
    }
}

pub async fn create_appointment(
    req: HttpRequest,
    db: web::Data<SupabaseRestDb>,
    doctor_service: web::Data<DoctorService>,
    payload: web::Json<Value>,
    firebase_auth: web::Data<FirebaseAuth>,
    firestore_db: web::Data<FirebaseRestDb>,
) -> impl Responder {
    if let Err(rejection) =
        require_appointments_permission(&req, &firebase_auth, &firestore_db).await
    {
        return rejection;
    }
    match validate_and_check_conflict(&db, &doctor_service, payload.into_inner(), None).await {
        Ok(appointment) => match db.create_appointment(&appointment).await {
            Ok(result) => HttpResponse::Created()
                .content_type("application/json")
                .body(result),
            Err(err) if is_database_schedule_conflict(&err) => {
                HttpResponse::Conflict().json(conflict_json(Vec::new()))
            }
            Err(err) => HttpResponse::InternalServerError().body(err),
        },
        Err(AppointmentError::BadRequest(msg)) => HttpResponse::BadRequest().body(msg),
        Err(AppointmentError::Conflict(suggestions)) => {
            HttpResponse::Conflict().json(conflict_json(suggestions))
        }
        Err(AppointmentError::ServerError(err)) => HttpResponse::InternalServerError().body(err),
    }
}

pub async fn update_appointment(
    req: HttpRequest,
    db: web::Data<SupabaseRestDb>,
    doctor_service: web::Data<DoctorService>,
    path: web::Path<String>,
    payload: web::Json<Value>,
    firebase_auth: web::Data<FirebaseAuth>,
    firestore_db: web::Data<FirebaseRestDb>,
) -> impl Responder {
    if let Err(rejection) =
        require_appointments_permission(&req, &firebase_auth, &firestore_db).await
    {
        return rejection;
    }
    let appointment_id = path.into_inner();
    // Once check-in starts, schedule edits could break the queue and room records.
    match appointment_status(&db, &appointment_id).await {
        Ok(Some(status)) if status == "Scheduled" => {}
        Ok(Some(status)) => {
            return HttpResponse::BadRequest().body(format!(
                "Cannot edit an appointment that is already \"{status}\"."
            ))
        }
        Ok(None) => return HttpResponse::NotFound().body("Appointment was not found."),
        Err(error) => return HttpResponse::InternalServerError().body(error),
    }

    match validate_and_check_conflict(
        &db,
        &doctor_service,
        payload.into_inner(),
        Some(&appointment_id),
    )
    .await
    {
        Ok(appointment) => match db.update_appointment(&appointment_id, &appointment).await {
            Ok(result) => HttpResponse::Ok()
                .content_type("application/json")
                .body(result),
            Err(err) if is_database_schedule_conflict(&err) => {
                HttpResponse::Conflict().json(conflict_json(Vec::new()))
            }
            Err(err) => HttpResponse::InternalServerError().body(err),
        },
        Err(AppointmentError::BadRequest(msg)) => HttpResponse::BadRequest().body(msg),
        Err(AppointmentError::Conflict(suggestions)) => {
            HttpResponse::Conflict().json(conflict_json(suggestions))
        }
        Err(AppointmentError::ServerError(err)) => HttpResponse::InternalServerError().body(err),
    }
}

const NON_CANCELLABLE_STATUSES: [&str; 4] = [
    "Checked In",
    "Vitals Recorded",
    "In Consultation",
    "Completed",
];

async fn appointment_status(
    db: &SupabaseRestDb,
    appointment_id: &str,
) -> Result<Option<String>, String> {
    let body = db.get_appointment(appointment_id).await?;
    let rows: Vec<Value> = serde_json::from_str(&body).unwrap_or_default();
    Ok(rows.into_iter().next().and_then(|row| {
        row.get("status")
            .and_then(Value::as_str)
            .map(str::to_string)
    }))
}

pub async fn delete_appointment(
    req: HttpRequest,
    db: web::Data<SupabaseRestDb>,
    path: web::Path<String>,
    firebase_auth: web::Data<FirebaseAuth>,
    firestore_db: web::Data<FirebaseRestDb>,
) -> impl Responder {
    if let Err(rejection) =
        require_appointments_permission(&req, &firebase_auth, &firestore_db).await
    {
        return rejection;
    }
    let appointment_id = path.into_inner();

    let status = match appointment_status(&db, &appointment_id).await {
        Ok(status) => status,
        Err(err) => return HttpResponse::InternalServerError().body(err),
    };

    // Only a future or untouched scheduled appointment can be cancelled.
    if let Some(status) = status.as_deref() {
        if NON_CANCELLABLE_STATUSES.contains(&status) {
            return HttpResponse::BadRequest().body(format!(
                "Cannot cancel an appointment that is already \"{status}\"."
            ));
        }
        if matches!(status, "Cancelled" | "No Show") {
            return HttpResponse::BadRequest()
                .body(format!("Appointment is already marked as \"{status}\"."));
        }
    } else {
        return HttpResponse::NotFound().body("Appointment was not found.");
    }

    // Keep the appointment for audit/history instead of deleting the row.
    match db
        .update_appointment(&appointment_id, &json!({ "status": "Cancelled" }))
        .await
    {
        Ok(result) => HttpResponse::Ok()
            .content_type("application/json")
            .body(result),
        Err(err) => {
            eprintln!("Failed to cancel appointment {appointment_id}: {err}");
            HttpResponse::InternalServerError().body("Failed to cancel appointment.")
        }
    }
}

async fn require_appointments_permission(
    req: &HttpRequest,
    firebase_auth: &FirebaseAuth,
    firestore_db: &FirebaseRestDb,
) -> Result<(String, String), HttpResponse> {
    require_auth_and_permission(
        req,
        firebase_auth,
        firestore_db,
        AppAction::ManageAppointments,
    )
    .await
}

fn is_database_schedule_conflict(error: &str) -> bool {
    error.contains("23P01")
        || error.contains("appointments_doctor_time_excl")
        || error.contains("conflicting key value violates exclusion constraint")
}

#[cfg(test)]
mod tests {
    use serde_json::json;

    use super::{
        appointment_blocks_time_slot, appointment_fits_schedule, is_database_schedule_conflict,
        normalize_appointment_payload, NON_CANCELLABLE_STATUSES,
    };
    use crate::doctors::models::{DayOfWeek, DoctorSchedule};
    use chrono::{DateTime, Utc};

    #[test]
    fn cancelled_and_no_show_appointments_do_not_block_slots() {
        assert!(!appointment_blocks_time_slot(
            &json!({ "status": "Cancelled" })
        ));
        assert!(!appointment_blocks_time_slot(
            &json!({ "status": "No Show" })
        ));
    }

    #[test]
    fn active_appointments_block_slots() {
        assert!(appointment_blocks_time_slot(
            &json!({ "status": "Scheduled" })
        ));
        assert!(appointment_blocks_time_slot(
            &json!({ "status": "Checked In" })
        ));
        assert!(appointment_blocks_time_slot(&json!({})));
    }

    #[test]
    fn identifies_database_overlap_errors() {
        assert!(is_database_schedule_conflict(
            "23P01: appointments_doctor_time_excl"
        ));
        assert!(!is_database_schedule_conflict(
            "23503: foreign key violation"
        ));
    }

    #[test]
    fn appointment_update_cannot_replace_id_or_status() {
        let mut payload = json!({
            "id": "NEW-ID",
            "status": "Completed",
            "doctor_id": "DOC-OLD"
        });

        normalize_appointment_payload(&mut payload, "DOC-1", false);

        assert!(payload.get("id").is_none());
        assert!(payload.get("status").is_none());
        assert_eq!(payload["doctor_id"], "DOC-1");
    }

    #[test]
    fn appointments_with_vitals_cannot_be_cancelled() {
        assert!(NON_CANCELLABLE_STATUSES.contains(&"Vitals Recorded"));
    }

    #[test]
    fn appointment_must_fit_inside_doctor_schedule() {
        let schedule = DoctorSchedule {
            id: "S-1".to_string(),
            doctor_id: "D-1".to_string(),
            day_of_week: DayOfWeek::Monday,
            start_time: "09:00".to_string(),
            end_time: "12:00".to_string(),
        };
        let inside = DateTime::parse_from_rfc3339("2026-07-06T10:00:00+08:00")
            .unwrap()
            .with_timezone(&Utc);
        let too_late = DateTime::parse_from_rfc3339("2026-07-06T11:45:00+08:00")
            .unwrap()
            .with_timezone(&Utc);

        assert!(appointment_fits_schedule(
            inside,
            30,
            std::slice::from_ref(&schedule)
        ));
        assert!(!appointment_fits_schedule(too_late, 30, &[schedule]));
    }

    #[test]
    fn doctor_without_schedule_uses_default_clinic_hours() {
        let morning = DateTime::parse_from_rfc3339("2026-07-06T09:00:00+08:00")
            .unwrap()
            .with_timezone(&Utc);
        let lunch = DateTime::parse_from_rfc3339("2026-07-06T12:00:00+08:00")
            .unwrap()
            .with_timezone(&Utc);

        assert!(appointment_fits_schedule(morning, 30, &[]));
        assert!(!appointment_fits_schedule(lunch, 30, &[]));
    }
}
