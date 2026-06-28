use actix_web::{web, HttpResponse, Responder};

use crate::db::SupabaseRestDb;
use crate::models::{CreateQueueEntryRequest, UpdateQueueStatusRequest};

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
    match serde_json::to_value(payload.into_inner()) {
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
    let payload = serde_json::json!({ "status": payload.status });
    match db.update_queue_entry(&queue_id, &payload).await {
        Ok(result) => HttpResponse::Ok().content_type("application/json").body(result),
        Err(err) => HttpResponse::InternalServerError().body(err),
    }
}
