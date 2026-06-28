use super::service::{DoctorDashboardError, DoctorDashboardService};
use crate::db::FirebaseRestDb;
use crate::handlers::auth::{require_auth_and_permission, AppAction};
use actix_web::{web, HttpRequest, HttpResponse, Responder};
use firebase_auth::FirebaseAuth;
use serde_json::json;

pub fn routes(config: &mut web::ServiceConfig) {
    config.route(
        "/api/doctor-dashboard/appointments",
        web::get().to(list_dashboard_appointments_api),
    );
}

pub async fn list_dashboard_appointments_api(
    req: HttpRequest,
    dashboard_service: web::Data<DoctorDashboardService>,
    firebase_auth: web::Data<FirebaseAuth>,
    firestore_db: web::Data<FirebaseRestDb>,
) -> impl Responder {
    if let Err(rejection) = require_auth_and_permission(
        &req,
        &firebase_auth,
        &firestore_db,
        AppAction::ManageDoctorAppt,
    )
    .await
    {
        return rejection;
    }

    match dashboard_service.list_appointments().await {
        Ok(appointments) => HttpResponse::Ok().json(appointments),
        Err(error) => dashboard_error_response(error),
    }
}

fn dashboard_error_response(error: DoctorDashboardError) -> HttpResponse {
    match error {
        DoctorDashboardError::StorageUnavailable => HttpResponse::ServiceUnavailable()
            .json(json!({ "error": "Doctor dashboard storage is unavailable" })),
    }
}