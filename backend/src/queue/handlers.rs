use actix_web::{web, HttpRequest, HttpResponse, Responder};
use chrono::Utc;
use firebase_auth::FirebaseAuth;
use serde_json::{json, Value};

use crate::admindashboard::repository::enqueue_patient;
use crate::db::{pg_error_message, FirebaseRestDb, SupabaseRestDb};
use crate::handlers::auth::{require_auth_and_permission, AppAction};
use crate::models::{CreateQueueEntryRequest, UpdateQueueStatusRequest};

const VALID_QUEUE_STATUSES: [&str; 6] = [
    "Waiting",
    "Called",
    "In Consultation",
    "Completed",
    "Skipped",
    "Cancelled",
];

pub fn routes(cfg: &mut web::ServiceConfig) {
    cfg.route(
        "/api/queue/doctor/{doctor_id}",
        web::get().to(list_queue_for_doctor),
    );
    cfg.route("/api/queue", web::post().to(create_queue_entry));
    cfg.route(
        "/api/queue/{queue_id}/status",
        web::put().to(update_queue_status),
    );
}

// ── JSON API handlers ────────────────────────────────────────────────────────

pub async fn list_queue_for_doctor(
    req: HttpRequest,
    db: web::Data<SupabaseRestDb>,
    path: web::Path<String>,
    firebase_auth: web::Data<FirebaseAuth>,
    firestore_db: web::Data<FirebaseRestDb>,
) -> impl Responder {
    if let Err(rejection) = require_queue_permission(&req, &firebase_auth, &firestore_db).await {
        return rejection;
    }
    let doctor_id = path.into_inner();
    match db.list_queue_by_doctor(&doctor_id).await {
        Ok(result) => HttpResponse::Ok()
            .content_type("application/json")
            .body(result),
        Err(err) => HttpResponse::InternalServerError().body(err),
    }
}

pub async fn create_queue_entry(
    req: HttpRequest,
    db: web::Data<SupabaseRestDb>,
    payload: web::Json<CreateQueueEntryRequest>,
    firebase_auth: web::Data<FirebaseAuth>,
    firestore_db: web::Data<FirebaseRestDb>,
) -> impl Responder {
    if let Err(rejection) = require_queue_permission(&req, &firebase_auth, &firestore_db).await {
        return rejection;
    }

    let request = payload.into_inner();
    let appointment_id = request.appointment_id.trim();
    if appointment_id.is_empty() {
        return HttpResponse::BadRequest().body("appointmentId is required");
    }

    let (priority, priority_reason) = match validate_priority(
        request.priority.as_deref(),
        request.priority_reason.as_deref(),
    ) {
        Ok(values) => values,
        Err(message) => return HttpResponse::BadRequest().body(message),
    };

    let patient_id = match appointment_patient_id(&db, appointment_id).await {
        Ok(patient_id) => patient_id,
        Err(response) => return response,
    };

    match enqueue_patient(
        &db,
        appointment_id,
        &patient_id,
        Utc::now().date_naive(),
        &priority,
        priority_reason.as_deref(),
    )
    .await
    {
        Ok(result) => HttpResponse::Created()
            .content_type("application/json")
            .body(result),
        Err(err) if err.contains("23505") || err.contains("duplicate key") => {
            HttpResponse::Conflict().body("Appointment is already in the queue")
        }
        Err(err) => match pg_error_message(&err) {
            Some(message) => HttpResponse::BadRequest().body(message),
            None => HttpResponse::InternalServerError().body(err),
        },
    }
}

pub async fn update_queue_status(
    req: HttpRequest,
    db: web::Data<SupabaseRestDb>,
    path: web::Path<String>,
    payload: web::Json<UpdateQueueStatusRequest>,
    firebase_auth: web::Data<FirebaseAuth>,
    firestore_db: web::Data<FirebaseRestDb>,
) -> impl Responder {
    if let Err(rejection) = require_queue_permission(&req, &firebase_auth, &firestore_db).await {
        return rejection;
    }
    let queue_id = path.into_inner();
    if !VALID_QUEUE_STATUSES.contains(&payload.status.as_str()) {
        return HttpResponse::BadRequest().body("Invalid queue status");
    }

    let now = Utc::now().to_rfc3339();
    let mut update = json!({
        "status": payload.status,
        "updated_at": now,
    });
    if payload.status == "Called" {
        update["called_at"] = json!(now);
    } else if payload.status == "Completed" {
        update["completed_at"] = json!(now);
    }

    match db.update_queue_entry(&queue_id, &update).await {
        Ok(result) => HttpResponse::Ok()
            .content_type("application/json")
            .body(result),
        Err(err) => HttpResponse::InternalServerError().body(err),
    }
}

async fn require_queue_permission(
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

async fn appointment_patient_id(
    db: &SupabaseRestDb,
    appointment_id: &str,
) -> Result<String, HttpResponse> {
    let body = db
        .get_appointment(appointment_id)
        .await
        .map_err(|err| HttpResponse::InternalServerError().body(err))?;
    let rows: Vec<Value> = serde_json::from_str(&body).unwrap_or_default();
    let Some(appointment) = rows.first() else {
        return Err(HttpResponse::NotFound().body("Appointment not found"));
    };

    appointment
        .get("patient_id")
        .and_then(Value::as_str)
        .filter(|value| !value.trim().is_empty())
        .map(str::to_string)
        .ok_or_else(|| HttpResponse::BadRequest().body("Appointment has no patient"))
}

fn validate_priority(
    priority: Option<&str>,
    priority_reason: Option<&str>,
) -> Result<(String, Option<String>), String> {
    let priority = priority.unwrap_or("Normal").trim();
    if !matches!(priority, "Normal" | "Urgent" | "Emergency") {
        return Err("priority must be Normal, Urgent, or Emergency".to_string());
    }

    let reason = priority_reason
        .map(str::trim)
        .filter(|value| !value.is_empty())
        .map(str::to_string);
    if matches!(priority, "Urgent" | "Emergency") && reason.is_none() {
        return Err("priorityReason is required for urgent or emergency cases".to_string());
    }

    Ok((priority.to_string(), reason))
}

#[cfg(test)]
mod tests {
    use super::{validate_priority, VALID_QUEUE_STATUSES};

    #[test]
    fn queue_statuses_match_the_patient_queue_schema() {
        assert!(VALID_QUEUE_STATUSES.contains(&"Called"));
        assert!(VALID_QUEUE_STATUSES.contains(&"In Consultation"));
        assert!(!VALID_QUEUE_STATUSES.contains(&"InProgress"));
    }

    #[test]
    fn urgent_priority_requires_a_reason() {
        assert!(validate_priority(Some("Urgent"), None).is_err());
        assert!(validate_priority(Some("Urgent"), Some("High fever")).is_ok());
    }
}
