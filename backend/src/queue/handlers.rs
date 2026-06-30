use actix_web::http::header;
use actix_web::{web, HttpRequest, HttpResponse, Responder};
use chrono::Utc;
use firebase_auth::FirebaseAuth;
use serde::Deserialize;
use serde_json::{json, Value};
use tera::{Context, Tera};

use crate::db::{FirebaseRestDb, SupabaseRestDb};
use crate::handlers::auth::{require_auth_and_permission, AppAction};
use crate::models::{CreateQueueEntryRequest, UpdateQueueStatusRequest};

const VALID_QUEUE_STATUSES: [&str; 5] = ["Waiting", "InProgress", "Completed", "Cancelled", "Skipped"];

// Statuses a staff member may set from the generic queue overview page.
// "Called" / "In Consultation" / "Completed" are owned by the admin dashboard's
// "send to room" action and the doctor dashboard's consultation flow respectively,
// so this page only offers the no-show/cancel actions to avoid clashing with those.
const PATIENT_QUEUE_FORM_STATUSES: [&str; 2] = ["Skipped", "Cancelled"];

pub fn routes(cfg: &mut web::ServiceConfig) {
    // SSR pages
    cfg.route("/queue", web::get().to(queue_page));
    cfg.route("/queue/{queue_id}/status", web::post().to(update_queue_status_form));

    // JSON API
    cfg.route("/api/queue/doctor/{doctor_id}", web::get().to(list_queue_for_doctor));
    cfg.route("/api/queue", web::post().to(create_queue_entry));
    cfg.route("/api/queue/{queue_id}/status", web::put().to(update_queue_status));
}

// ── JSON API handlers ────────────────────────────────────────────────────────

pub async fn list_queue_for_doctor(
    db: web::Data<SupabaseRestDb>,
    path: web::Path<String>,
) -> impl Responder {
    let doctor_id = path.into_inner();
    match db.list_queue_by_doctor(&doctor_id).await {
        Ok(result) => HttpResponse::Ok().content_type("application/json").body(result),
        Err(err) => HttpResponse::InternalServerError().body(err),
    }
}

pub async fn create_queue_entry(
    db: web::Data<SupabaseRestDb>,
    payload: web::Json<CreateQueueEntryRequest>,
) -> impl Responder {
    let payload = payload.into_inner();
    if !(1..=4).contains(&payload.priority) {
        return HttpResponse::BadRequest().body("priority must be between 1 and 4");
    }

    match serde_json::to_value(payload) {
        Ok(payload) => match db.create_queue_entry(&payload).await {
            Ok(result) => HttpResponse::Created().content_type("application/json").body(result),
            Err(err) => HttpResponse::InternalServerError().body(err),
        },
        Err(_) => HttpResponse::BadRequest().body("Invalid queue entry payload"),
    }
}

pub async fn update_queue_status(
    db: web::Data<SupabaseRestDb>,
    path: web::Path<String>,
    payload: web::Json<UpdateQueueStatusRequest>,
) -> impl Responder {
    let queue_id = path.into_inner();
    if !VALID_QUEUE_STATUSES.contains(&payload.status.as_str()) {
        return HttpResponse::BadRequest().body("Invalid queue status");
    }

    let payload = serde_json::json!({ "status": payload.status });
    match db.update_queue_entry(&queue_id, &payload).await {
        Ok(result) => HttpResponse::Ok().content_type("application/json").body(result),
        Err(err) => HttpResponse::InternalServerError().body(err),
    }
}

// ── SSR page handlers ────────────────────────────────────────────────────────

pub async fn queue_page(
    req: HttpRequest,
    tera: web::Data<Tera>,
    firebase_auth: web::Data<FirebaseAuth>,
    firestore_db: web::Data<FirebaseRestDb>,
    db: web::Data<SupabaseRestDb>,
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

    let today = Utc::now().date_naive().format("%Y-%m-%d").to_string();
    let filter = format!(
        "select=*,patients(first_name,last_name),appointments(doctor_id,reason,doctors(specialty,staff(full_name)))&queue_date=eq.{today}&status=neq.Completed&status=neq.Cancelled&order=checked_in_at.asc"
    );

    let mut entries = match db.fetch_table("patient_queue", &filter).await {
        Ok(body) => serde_json::from_str::<Vec<Value>>(&body).unwrap_or_default(),
        Err(err) => return render_error(&tera, format!("Failed to load queue: {err}")),
    };

    // Flatten the joined appointment's doctor_id to a top-level field so the
    // template can group entries by doctor.
    for entry in entries.iter_mut() {
        let doctor_id = entry
            .get("appointments")
            .and_then(|a| a.get("doctor_id"))
            .and_then(Value::as_str)
            .unwrap_or("Unassigned")
            .to_string();
        if let Some(object) = entry.as_object_mut() {
            object.insert("doctor_id".to_string(), json!(doctor_id));
        }
    }

    let mut ctx = Context::new();
    ctx.insert("entries", &entries);
    render_template(&tera, "queue/index.html", &ctx)
}

#[derive(Deserialize)]
pub struct QueueStatusFormInput {
    pub status: String,
}

pub async fn update_queue_status_form(
    req: HttpRequest,
    firebase_auth: web::Data<FirebaseAuth>,
    firestore_db: web::Data<FirebaseRestDb>,
    db: web::Data<SupabaseRestDb>,
    path: web::Path<String>,
    form: web::Form<QueueStatusFormInput>,
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

    let queue_id = path.into_inner();
    let status = form.into_inner().status;

    if PATIENT_QUEUE_FORM_STATUSES.contains(&status.as_str()) {
        let encoded_id = urlencoding::encode(&queue_id);
        let payload = json!({ "status": status });
        let _ = db.patch_row(&format!("id=eq.{encoded_id}"), &payload).await;
    }

    HttpResponse::SeeOther()
        .insert_header((header::LOCATION, "/queue"))
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
