use super::service::{StaffError, StaffForm, StaffService};
use crate::db::FirebaseRestDb;
use crate::doctors::service::{DoctorError, DoctorService};
use crate::firebase_admin::FirebaseAdmin;
use crate::auth::{require_auth_and_permission, AppAction};
use actix_web::{web, HttpRequest, HttpResponse, Responder};
use firebase_auth::FirebaseAuth;
use serde::Deserialize;

pub fn routes(config: &mut web::ServiceConfig) {
    config
        .route("/api/staff", web::get().to(list_staff_api))
        .route("/api/staff", web::post().to(create_staff_api))
        .route("/api/staff/{staff_id}", web::put().to(update_staff_api))
        .route("/api/staff/{staff_id}", web::delete().to(delete_staff_api))
        .route(
            "/api/staff/{staff_id}/password",
            web::put().to(set_staff_password_api),
        );
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
    firebase_admin: web::Data<FirebaseAdmin>,
    form: web::Json<StaffForm>,
    firebase_auth: web::Data<FirebaseAuth>,
    firestore_db: web::Data<FirebaseRestDb>,
) -> impl Responder {
    if let Err(rejection) = require_staff_admin(&req, &firebase_auth, &firestore_db).await {
        return rejection;
    }

    // Create the Firebase login first so its UID can be stored with the staff row.
    let mut form = form.into_inner();
    let full_name = format!("{} {}", form.first_name.trim(), form.last_name.trim());
    let uid = match firebase_admin.create_user(&form.email, &full_name).await {
        Ok(uid) => uid,
        Err(message) => return HttpResponse::BadRequest().body(message),
    };
    form.firebase_uid = uid.clone();

    match staff_service.create_staff(form).await {
        Ok(staff) => {
            // The role profile is separate because authentication and staff data use different services.
            if let Err(error) = firebase_admin
                .save_staff_profile(
                    &staff.firebase_uid,
                    &staff.first_name,
                    &staff.last_name,
                    &staff.role,
                    &staff.status,
                )
                .await
            {
                // Do not leave a staff login half-created if its role cannot be read.
                let _ = staff_service.delete_staff(&staff.id).await;
                let _ = firebase_admin.delete_user(&uid).await;
                return HttpResponse::InternalServerError().body(error);
            }
            if let Err(error) = firebase_admin.send_password_setup(&staff.email).await {
                eprintln!("Firebase password setup email failed: {error}");
            }
            HttpResponse::Created().json(staff)
        }
        Err(error) => {
            if let Err(rollback_error) = firebase_admin.delete_user(&uid).await {
                eprintln!("Firebase user rollback failed: {rollback_error}");
            }
            staff_error_response(error)
        }
    }
}

pub async fn update_staff_api(
    req: HttpRequest,
    staff_service: web::Data<StaffService>,
    firebase_admin: web::Data<FirebaseAdmin>,
    path: web::Path<String>,
    form: web::Json<StaffForm>,
    firebase_auth: web::Data<FirebaseAuth>,
    firestore_db: web::Data<FirebaseRestDb>,
) -> impl Responder {
    if let Err(rejection) = require_staff_admin(&req, &firebase_auth, &firestore_db).await {
        return rejection;
    }

    match staff_service
        .update_staff(&path.into_inner(), form.into_inner())
        .await
    {
        Ok(staff) => {
            if let Err(error) = firebase_admin
                .update_email(&staff.firebase_uid, &staff.email)
                .await
            {
                return HttpResponse::InternalServerError().body(error);
            }
            if let Err(error) = firebase_admin
                .save_staff_profile(
                    &staff.firebase_uid,
                    &staff.first_name,
                    &staff.last_name,
                    &staff.role,
                    &staff.status,
                )
                .await
            {
                return HttpResponse::InternalServerError().body(error);
            }
            HttpResponse::Ok().json(staff)
        }
        Err(error) => staff_error_response(error),
    }
}

pub async fn delete_staff_api(
    req: HttpRequest,
    staff_service: web::Data<StaffService>,
    doctor_service: web::Data<DoctorService>,
    firebase_admin: web::Data<FirebaseAdmin>,
    path: web::Path<String>,
    firebase_auth: web::Data<FirebaseAuth>,
    firestore_db: web::Data<FirebaseRestDb>,
) -> impl Responder {
    if let Err(rejection) = require_staff_admin(&req, &firebase_auth, &firestore_db).await {
        return rejection;
    }

    let staff_id = path.into_inner();
    let firebase_uid = staff_service
        .list_staff()
        .await
        .ok()
        .and_then(|staff| staff.into_iter().find(|item| item.id == staff_id))
        .map(|staff| staff.firebase_uid);

    let linked_doctor = match doctor_service.list_doctors().await {
        Ok(doctors) => doctors
            .into_iter()
            .find(|doctor| doctor.staff_id == staff_id),
        Err(error) => return doctor_delete_error(error),
    };
    // Remove the doctor profile first because it points to the staff record.
    if let Some(doctor) = linked_doctor {
        if let Err(error) = doctor_service.delete_doctor(&doctor.id).await {
            return doctor_delete_error(error);
        }
    }

    match staff_service.delete_staff(&staff_id).await {
        Ok(()) => {
            if let Some(uid) = firebase_uid {
                if let Err(error) = firebase_admin.delete_staff_profile(&uid).await {
                    eprintln!("Firebase staff profile deletion failed: {error}");
                }
                if let Err(error) = firebase_admin.delete_user(&uid).await {
                    eprintln!("Firebase user deletion failed: {error}");
                }
            }
            HttpResponse::NoContent().finish()
        }
        Err(error) => staff_error_response(error),
    }
}

#[derive(Deserialize)]
pub struct SetPasswordForm {
    pub new_password: String,
}

pub async fn set_staff_password_api(
    req: HttpRequest,
    staff_service: web::Data<StaffService>,
    firebase_admin: web::Data<FirebaseAdmin>,
    path: web::Path<String>,
    form: web::Json<SetPasswordForm>,
    firebase_auth: web::Data<FirebaseAuth>,
    firestore_db: web::Data<FirebaseRestDb>,
) -> impl Responder {
    if let Err(rejection) = require_staff_admin(&req, &firebase_auth, &firestore_db).await {
        return rejection;
    }

    let new_password = form.new_password.trim();
    if new_password.chars().count() < 6 {
        return HttpResponse::BadRequest().body("Password must be at least 6 characters.");
    }

    let staff_id = path.into_inner();
    let firebase_uid = match staff_service.list_staff().await {
        Ok(staff) => staff
            .into_iter()
            .find(|item| item.id == staff_id)
            .map(|item| item.firebase_uid),
        Err(error) => return staff_error_response(error),
    };

    let Some(firebase_uid) = firebase_uid else {
        return HttpResponse::NotFound().body("Staff member was not found.");
    };

    match firebase_admin.set_password(&firebase_uid, new_password).await {
        Ok(()) => HttpResponse::Ok().finish(),
        Err(error) => HttpResponse::BadRequest().body(error),
    }
}

fn doctor_delete_error(error: DoctorError) -> HttpResponse {
    match error {
        DoctorError::DoctorHasAppointments => HttpResponse::BadRequest()
            .body("This doctor cannot be deleted because appointments still reference them."),
        DoctorError::DoctorNotFound => HttpResponse::NotFound().body("Doctor was not found."),
        DoctorError::InvalidInput(message) => HttpResponse::BadRequest().body(message),
        DoctorError::StorageUnavailable => {
            HttpResponse::InternalServerError().body("Doctor storage is unavailable.")
        }
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
        StaffError::StaffHasDoctorProfile => {
            "The linked doctor profile could not be deleted.".to_string()
        }
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
