use super::models::AppointmentActionPayload;
use super::service::{DoctorDashboardError, DoctorDashboardService};
use crate::db::{singapore_today, FirebaseRestDb};
use crate::handlers::auth::{require_auth_and_permission, AppAction};
use actix_web::{web, HttpRequest, HttpResponse, Responder};
use firebase_auth::FirebaseAuth;
use serde_json::json;

pub fn routes(config: &mut web::ServiceConfig) {
    config.route(
        "/api/doctor-dashboard/appointments",
        web::get().to(list_dashboard_appointments_api),
    );
    config.route(
        "/api/doctor-dashboard/start-consultation",
        web::post().to(start_consultation_api),
    );
    config.route(
        "/api/doctor-dashboard/extend-consultation",
        web::post().to(extend_consultation_api),
    );
    config.route(
        "/api/doctor-dashboard/complete-consultation",
        web::post().to(complete_consultation_api),
    );
}

pub async fn list_dashboard_appointments_api(
    req: HttpRequest,
    dashboard_service: web::Data<DoctorDashboardService>,
    firebase_auth: web::Data<FirebaseAuth>,
    firestore_db: web::Data<FirebaseRestDb>,
    date_query: web::Query<std::collections::HashMap<String, String>>,
) -> impl Responder {
    let (firebase_uid, _) = match require_auth_and_permission(
        &req,
        &firebase_auth,
        &firestore_db,
        AppAction::ManageDoctorAppt,
    )
    .await
    {
        Ok(data) => data,
        Err(rejection) => return rejection,
    };

    let today = singapore_today();
    let default_date = today.format("%Y-%m-%d").to_string();

    let date_str = date_query.get("date").unwrap_or(&default_date);

    let target_date = chrono::NaiveDate::parse_from_str(date_str, "%Y-%m-%d")
        .unwrap_or(today);

    match dashboard_service
        .list_appointments(&firebase_uid, target_date)
        .await
    {
        Ok(appointments) => HttpResponse::Ok().json(appointments),
        Err(error) => dashboard_error_response(error),
    }
}

pub async fn start_consultation_api(
    req: HttpRequest,
    dashboard_service: web::Data<DoctorDashboardService>,
    firebase_auth: web::Data<FirebaseAuth>,
    firestore_db: web::Data<FirebaseRestDb>,
    payload: web::Json<AppointmentActionPayload>,
) -> impl Responder {
    let (firebase_uid, _) = match require_auth_and_permission(
        &req,
        &firebase_auth,
        &firestore_db,
        AppAction::ManageDoctorAppt,
    )
    .await
    {
        Ok(data) => data,
        Err(rejection) => return rejection,
    };

    match dashboard_service
        .start_consultation(&firebase_uid, &payload.appointment_id)
        .await
    {
        Ok(_) => HttpResponse::Ok().json(json!({ "success": true })),
        Err(error) => dashboard_error_response(error),
    }
}

pub async fn extend_consultation_api(
    req: HttpRequest,
    dashboard_service: web::Data<DoctorDashboardService>,
    firebase_auth: web::Data<FirebaseAuth>,
    firestore_db: web::Data<FirebaseRestDb>,
    payload: web::Json<AppointmentActionPayload>,
) -> impl Responder {
    let (firebase_uid, _) = match require_auth_and_permission(
        &req,
        &firebase_auth,
        &firestore_db,
        AppAction::ManageDoctorAppt,
    )
    .await
    {
        Ok(data) => data,
        Err(rejection) => return rejection,
    };

    match dashboard_service
        .extend_consultation(&firebase_uid, &payload.appointment_id)
        .await
    {
        Ok(_) => HttpResponse::Ok().json(json!({ "success": true })),
        Err(error) => dashboard_error_response(error),
    }
}

pub async fn complete_consultation_api(
    req: HttpRequest,
    dashboard_service: web::Data<DoctorDashboardService>,
    firebase_auth: web::Data<FirebaseAuth>,
    firestore_db: web::Data<FirebaseRestDb>,
    payload: web::Json<AppointmentActionPayload>,
) -> impl Responder {
    let (firebase_uid, _) = match require_auth_and_permission(
        &req,
        &firebase_auth,
        &firestore_db,
        AppAction::ManageDoctorAppt,
    )
    .await
    {
        Ok(data) => data,
        Err(rejection) => return rejection,
    };

    match dashboard_service
        .complete_consultation(&firebase_uid, &payload.appointment_id)
        .await
    {
        Ok(_) => HttpResponse::Ok().json(json!({ "success": true })),
        Err(error) => dashboard_error_response(error),
    }
}

fn dashboard_error_response(error: DoctorDashboardError) -> HttpResponse {
    match error {
        DoctorDashboardError::StorageUnavailable => HttpResponse::ServiceUnavailable()
            .json(json!({ "error": "Doctor dashboard storage is unavailable" })),
        DoctorDashboardError::Rejected(message) => {
            HttpResponse::BadRequest().json(json!({ "error": message }))
        }
    }
}
