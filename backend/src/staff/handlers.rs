use super::service::{StaffError, StaffForm, StaffService};
use crate::db::FirebaseRestDb;
use crate::handlers::auth::{require_auth_and_permission, AppAction};
use actix_web::{web, HttpRequest, HttpResponse, Responder};
use firebase_auth::FirebaseAuth;

pub fn routes(config: &mut web::ServiceConfig) {
    config
        .route("/api/staff", web::get().to(list_staff_api))
        .route("/api/staff", web::post().to(create_staff_api))
        .route("/api/staff/{staff_id}", web::put().to(update_staff_api));
}

pub async fn list_staff_api(
    req: HttpRequest,
    staff_service: web::Data<StaffService>,
    firebase_auth: web::Data<FirebaseAuth>,
    firestore_db: web::Data<FirebaseRestDb>,
) -> impl Responder {
    if let Err(rejection) = require_staff_admin(&req, &firebase_auth, &firestore_db).await {
        return rejection;
    }

    match staff_service.list_staff().await {
        Ok(staff) => HttpResponse::Ok().json(staff),
        Err(error) => staff_error_response(error),
    }
}

pub async fn create_staff_api(
    req: HttpRequest,
    staff_service: web::Data<StaffService>,
    form: web::Json<StaffForm>,
    firebase_auth: web::Data<FirebaseAuth>,
    firestore_db: web::Data<FirebaseRestDb>,
) -> impl Responder {
    if let Err(rejection) = require_staff_admin(&req, &firebase_auth, &firestore_db).await {
        return rejection;
    }

    match staff_service.create_staff(form.into_inner()).await {
        Ok(staff) => HttpResponse::Created().json(staff),
        Err(error) => staff_error_response(error),
    }
}

pub async fn update_staff_api(
    req: HttpRequest,
    staff_service: web::Data<StaffService>,
    path: web::Path<String>,
    form: web::Json<StaffForm>,
    firebase_auth: web::Data<FirebaseAuth>,
    firestore_db: web::Data<FirebaseRestDb>,
) -> impl Responder {
    if let Err(rejection) = require_staff_admin(&req, &firebase_auth, &firestore_db).await {
        return rejection;
    }

    match staff_service.update_staff(&path.into_inner(), form.into_inner()).await {
        Ok(staff) => HttpResponse::Ok().json(staff),
        Err(error) => staff_error_response(error),
    }
}

async fn require_staff_admin(
    req: &HttpRequest,
    firebase_auth: &FirebaseAuth,
    firestore_db: &FirebaseRestDb,
) -> Result<(String, String), HttpResponse> {
    require_auth_and_permission(req, firebase_auth, firestore_db, AppAction::ManageUsers).await
}

fn staff_error_response(error: StaffError) -> HttpResponse {
    let message = match error {
        StaffError::InvalidInput(message) => message,
        StaffError::StaffNotFound => "Staff member was not found.".to_string(),
        StaffError::StorageUnavailable => "Staff storage is unavailable.".to_string(),
    };

    if message == "Staff member was not found." {
        HttpResponse::NotFound().body(message)
    } else if message == "Staff storage is unavailable." {
        HttpResponse::InternalServerError().body(message)
    } else {
        HttpResponse::BadRequest().body(message)
    }
}
