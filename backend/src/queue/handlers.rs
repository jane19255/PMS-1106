use actix_web::{get, post, put, web, HttpResponse, Responder};

use crate::db::SupabaseRestDb;
use crate::models::{CreateQueueEntryRequest, UpdateQueueStatusRequest};
use crate::queue::repository::QueueRepository;
use crate::queue::service::QueueService;

// =========================================================
// Queue management handlers
// Handles queue display, queue entry creation, and
// patient status changes.
// =========================================================

#[get("/queue/doctor/{doctor_id}")]
pub async fn list_queue_for_doctor(
    db: web::Data<SupabaseRestDb>,
    path: web::Path<String>,
) -> impl Responder {
    let doctor_id = path.into_inner();
    let repository = QueueRepository::new(db.get_ref().clone());
    let service = QueueService::new(repository);

    match service.get_ordered_queue_for_doctor(&doctor_id).await {
        Ok(queue) => HttpResponse::Ok().json(queue),
        Err(err) => HttpResponse::InternalServerError().body(err),
    }
}

#[post("/queue")]
pub async fn create_queue_entry(
    db: web::Data<SupabaseRestDb>,
    payload: web::Json<CreateQueueEntryRequest>,
) -> impl Responder {
    let repository = QueueRepository::new(db.get_ref().clone());

    match repository.create_entry(&payload.into_inner()).await {
        Ok(result) => HttpResponse::Ok().body(result),
        Err(err) => HttpResponse::InternalServerError().body(err),
    }
}

#[put("/queue/{queue_id}/status")]
pub async fn update_queue_status(
    db: web::Data<SupabaseRestDb>,
    path: web::Path<String>,
    payload: web::Json<UpdateQueueStatusRequest>,
) -> impl Responder {
    let queue_id = path.into_inner();
    let repository = QueueRepository::new(db.get_ref().clone());

    match repository.update_status(&queue_id, &payload.status).await {
        Ok(result) => HttpResponse::Ok().body(result),
        Err(err) => HttpResponse::InternalServerError().body(err),
    }
}