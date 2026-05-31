use actix_web::{web, HttpRequest, HttpResponse, Responder};
use serde_json::json;
use tera::{Context, Tera};
use chrono::Utc;

use crate::db::FirebaseRestDb;
use crate::models::{Appointment, QueueEntry};
use crate::auth::{require_auth, require_permission, AppAction};

pub fn routes(cfg: &mut web::ServiceConfig) {
    cfg.route("/appointments", web::get().to(appointments_page));

    cfg.route(
        "/api/appointments/create",
        web::post().to(create_appointment),
    );

    cfg.route(
        "/api/appointments/update/{id}",
        web::put().to(update_appointment),
    );

    cfg.route(
        "/api/appointments/delete/{id}",
        web::delete().to(delete_appointment),
    );

    cfg.route(
        "/api/queue/checkin",
        web::post().to(check_in_patient),
    );
}

pub async fn appointments_page(
    tera: web::Data<Tera>,
) -> impl Responder {
    let ctx = Context::new();

    match tera.render("Appointments.html", &ctx) {
        Ok(html) => HttpResponse::Ok()
            .content_type("text/html")
            .body(html),

        Err(e) => HttpResponse::InternalServerError()
            .body(format!("Template Error: {}", e)),
    }
}

impl Appointment {
    pub fn validate(&self) -> Result<(), String> {
        if self.status.trim().is_empty() {
            return Err("Status cannot be empty".into());
        }
        if self.datetime < Utc::now() {
            return Err("Appointment datetime cannot be in the past".into());
        }
        Ok(())
    }
}

impl QueueEntry {
    pub fn validate(&self) -> Result<(), String> {
        if self.status.trim().is_empty() {
            return Err("Status cannot be empty".into());
        }
        if self.queue_number == 0 {
            return Err("Queue number must be greater than zero".into());
        }
        Ok(())
    }
}

pub async fn create_appointment(
    req: HttpRequest,
    firestore_db: web::Data<FirebaseRestDb>,
    appointment: web::Json<Appointment>,
) -> impl Responder {
    // Authentication
    let _uid = match require_auth(&req, &firestore_db.firebase_auth).await {
        Ok(uid) => uid,
        Err(resp) => return resp,
    };

    // Authorization
    if let Err(resp) = require_permission(&req, AppAction::ManageAppointments) {
        return resp;
    }

    // Validation
    if let Err(e) = appointment.validate() {
        return HttpResponse::BadRequest().body(format!("Validation error: {}", e));
    }

    let payload = json!({
        "fields": {
            "appointmentId": { "stringValue": appointment.appointment_id.to_string() },
            "patientId": { "stringValue": appointment.patient_id.to_string() },
            "doctorId": { "stringValue": appointment.doctor_id.to_string() },
            "datetime": { "stringValue": appointment.datetime.to_rfc3339() },
            "status": { "stringValue": appointment.status },
            "notes": appointment.notes.as_ref().map(|n| json!({"stringValue": n})).unwrap_or(json!(null)),
            "createdAt": appointment.created_at.map(|dt| json!({"stringValue": dt.to_rfc3339()})).unwrap_or(json!(null)),
            "updatedAt": appointment.updated_at.map(|dt| json!({"stringValue": dt.to_rfc3339()})).unwrap_or(json!(null)),
        }
    });

    match firestore_db
        .create_document("appointments", &appointment.appointment_id.to_string(), &payload)
        .await
    {
        Ok(_) => HttpResponse::Ok().json(json!({ "status": "success" })),
        Err(e) => HttpResponse::InternalServerError().body(e.to_string()),
    }
}

pub async fn update_appointment(
    req: HttpRequest,
    path: web::Path<String>,
    firestore_db: web::Data<FirebaseRestDb>,
    appointment: web::Json<Appointment>,
) -> impl Responder {
    let id = path.into_inner();

    // Authentication
    let _uid = match require_auth(&req, &firestore_db.firebase_auth).await {
        Ok(uid) => uid,
        Err(resp) => return resp,
    };

    // Authorization
    if let Err(resp) = require_permission(&req, AppAction::ManageAppointments) {
        return resp;
    }

    // Validation
    if let Err(e) = appointment.validate() {
        return HttpResponse::BadRequest().body(format!("Validation error: {}", e));
    }

    let payload = json!({
        "fields": {
            "patientId": { "stringValue": appointment.patient_id.to_string() },
            "doctorId": { "stringValue": appointment.doctor_id.to_string() },
            "datetime": { "stringValue": appointment.datetime.to_rfc3339() },
            "status": { "stringValue": appointment.status },
            "notes": appointment.notes.as_ref().map(|n| json!({"stringValue": n})).unwrap_or(json!(null)),
            "updatedAt": Some(json!({"stringValue": Utc::now().to_rfc3339()})).unwrap_or(json!(null)),
        }
    });

    match firestore_db
        .update_document("appointments", &id, &payload)
        .await
    {
        Ok(_) => HttpResponse::Ok().body("updated"),
        Err(e) => HttpResponse::InternalServerError().body(e.to_string()),
    }
}

pub async fn delete_appointment(
    req: HttpRequest,
    path: web::Path<String>,
    firestore_db: web::Data<FirebaseRestDb>,
) -> impl Responder {
    let id = path.into_inner();

    // Authentication
    let _uid = match require_auth(&req, &firestore_db.firebase_auth).await {
        Ok(uid) => uid,
        Err(resp) => return resp,
    };

    // Authorization
    if let Err(resp) = require_permission(&req, AppAction::ManageAppointments) {
        return resp;
    }

    match firestore_db.delete_document("appointments", &id).await {
        Ok(_) => HttpResponse::Ok().body("deleted"),
        Err(e) => HttpResponse::InternalServerError().body(e.to_string()),
    }
}

pub async fn check_in_patient(
    req: HttpRequest,
    firestore_db: web::Data<FirebaseRestDb>,
    queue: web::Json<QueueEntry>,
) -> impl Responder {
    // Authentication
    let _uid = match require_auth(&req, &firestore_db.firebase_auth).await {
        Ok(uid) => uid,
        Err(resp) => return resp,
    };

    // Authorization
    if let Err(resp) = require_permission(&req, AppAction::ManageAppointments) {
        return resp;
    }

    // Validation
    if let Err(e) = queue.validate() {
        return HttpResponse::BadRequest().body(format!("Validation error: {}", e));
    }

    let payload = json!({
        "fields": {
            "appointmentId": { "stringValue": queue.appointment_id.to_string() },
            "patientId": { "stringValue": queue.patient_id.to_string() },
            "doctorId": { "stringValue": queue.doctor_id.to_string() },
            "queueNumber": { "integerValue": queue.queue_number },
            "status": { "stringValue": queue.status },
        }
    });

    match firestore_db
        .create_document("queue", &queue.queue_id.to_string(), &payload)
        .await
    {
        Ok(_) => HttpResponse::Ok().json(json!({ "status": "checked_in" })),
        Err(e) => HttpResponse::InternalServerError().body(e.to_string()),
    }
}