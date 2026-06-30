use actix_web::{web, HttpResponse, Responder};

use crate::db::SupabaseRestDb;
use crate::models::{CreateQueueEntryRequest, UpdateQueueStatusRequest};

const VALID_QUEUE_STATUSES: [&str; 5] = ["Waiting", "InProgress", "Completed", "Cancelled", "Skipped"];

pub fn routes(cfg: &mut web::ServiceConfig) {
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
