use actix_web::{delete, get, post, put, web, HttpResponse, Responder};
use crate::db::SupabaseRestDb;

// ===============================
// Appointment scheduling handlers
// These async functions handle HTTP requests related to appointment management.
// Each handler calls the corresponding database method from SupabaseRestDb,
// processes the result, and returns an appropriate HTTP response.
// ===============================

/// Handler to list all appointments.
/// Responds to GET /appointments.
/// Calls db.list_appointments() to fetch all appointments ordered by datetime.
#[get("/appointments")]
pub async fn list_appointments(db: web::Data<SupabaseRestDb>) -> impl Responder {
    match db.list_appointments().await {
        Ok(result) => HttpResponse::Ok().body(result), // Return JSON string on success
        Err(err) => HttpResponse::InternalServerError().body(err), // Return error message on failure
    }
}

/// Handler to get details of a single appointment by ID.
/// Responds to GET /appointments/{appointment_id}.
/// Extracts appointment_id from URL path and calls db.get_appointment().
#[get("/appointments/{appointment_id}")]
pub async fn get_appointment(
    db: web::Data<SupabaseRestDb>,
    path: web::Path<String>,
) -> impl Responder {
    let appointment_id = path.into_inner();
    match db.get_appointment(&appointment_id).await {
        Ok(result) => HttpResponse::Ok().body(result),
        Err(err) => HttpResponse::InternalServerError().body(err),
    }
}

/// Handler to create a new appointment.
/// Responds to POST /appointments.
/// Expects JSON payload representing appointment data.
/// Calls db.create_appointment() with the payload.
#[post("/appointments")]
pub async fn create_appointment(
    db: web::Data<SupabaseRestDb>,
    payload: web::Json<serde_json::Value>,
) -> impl Responder {
    match db.create_appointment(&payload).await {
        Ok(result) => HttpResponse::Ok().body(result),
        Err(err) => HttpResponse::InternalServerError().body(err),
    }
}

/// Handler to update an existing appointment by ID.
/// Responds to PUT /appointments/{appointment_id}.
/// Extracts appointment_id from path and expects JSON payload with updated data.
/// Calls db.update_appointment().
#[put("/appointments/{appointment_id}")]
pub async fn update_appointment(
    db: web::Data<SupabaseRestDb>,
    path: web::Path<String>,
    payload: web::Json<serde_json::Value>,
) -> impl Responder {
    let appointment_id = path.into_inner();
    match db.update_appointment(&appointment_id, &payload).await {
        Ok(result) => HttpResponse::Ok().body(result),
        Err(err) => HttpResponse::InternalServerError().body(err),
    }
}

/// Handler to delete an appointment by ID.
/// Responds to DELETE /appointments/{appointment_id}.
/// Extracts appointment_id from path and calls db.delete_appointment().
#[delete("/appointments/{appointment_id}")]
pub async fn delete_appointment(
    db: web::Data<SupabaseRestDb>,
    path: web::Path<String>,
) -> impl Responder {
    let appointment_id = path.into_inner();
    match db.delete_appointment(&appointment_id).await {
        Ok(result) => HttpResponse::Ok().body(result),
        Err(err) => HttpResponse::InternalServerError().body(err),
    }
}