use actix_web::{web, HttpResponse, Responder};
use serde_json::{json, Value};
use uuid::Uuid;

use crate::db::SupabaseRestDb;
use crate::doctors::service::DoctorService;

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
    let mut appointment = payload.into_inner();
    let Some(doctor_id) = appointment
        .get("doctor_id")
        .and_then(Value::as_str)
        .map(str::trim)
        .filter(|value| !value.is_empty())
        .map(str::to_string)
    else {
        return HttpResponse::BadRequest().body("doctor_id is required");
    };

    if doctor_service.find_doctor(&doctor_id).await.is_err() {
        return HttpResponse::BadRequest().body("Selected doctor does not exist");
    }

    normalize_appointment_payload(&mut appointment, &doctor_id);

    match db.create_appointment(&appointment).await {
        Ok(result) => HttpResponse::Created().content_type("application/json").body(result),
        Err(err) => HttpResponse::InternalServerError().body(err),
    }
}

pub async fn update_appointment(
    db: web::Data<SupabaseRestDb>,
    path: web::Path<String>,
    payload: web::Json<Value>,
) -> impl Responder {
    let appointment_id = path.into_inner();
    match db.update_appointment(&appointment_id, &payload.into_inner()).await {
        Ok(result) => HttpResponse::Ok().content_type("application/json").body(result),
        Err(err) => HttpResponse::InternalServerError().body(err),
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
