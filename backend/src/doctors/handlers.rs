use super::service::{CreateDoctorForm, CreateDoctorScheduleForm, DoctorError, DoctorService, UpdateDoctorForm};
use crate::db::FirebaseRestDb;
use crate::handlers::auth::{require_auth_and_permission, AppAction};
use actix_web::http::header;
use actix_web::{web, HttpRequest, HttpResponse, Responder};
use firebase_auth::FirebaseAuth;
use serde_json::json;
use tera::{Context, Tera};

pub fn routes(config: &mut web::ServiceConfig) {
    config
        .route("/doctors", web::get().to(doctors_page))
        .route("/doctors", web::post().to(create_doctor_form))
        .route("/doctors/{doctor_id}", web::get().to(show_doctor))
        .route("/doctors/{doctor_id}", web::post().to(update_doctor_form))
        .route("/doctors/{doctor_id}/delete", web::post().to(delete_doctor_form))
        .route(
            "/doctors/{doctor_id}/schedules",
            web::post().to(create_schedule_form),
        )
        .route(
            "/doctors/schedules/{schedule_id}/delete",
            web::post().to(delete_schedule_form),
        )
        .route("/api/doctors", web::get().to(list_doctors_api))
        .route("/api/doctors", web::post().to(create_doctor_api))
        .route("/api/doctors/{doctor_id}", web::get().to(show_doctor_api))
        .route("/api/doctors/{doctor_id}", web::put().to(update_doctor_api))
        .route("/api/doctors/{doctor_id}", web::delete().to(delete_doctor_api))
        .route(
            "/api/doctors/{doctor_id}/schedules",
            web::get().to(list_schedules_api),
        )
        .route(
            "/api/doctors/{doctor_id}/schedules",
            web::post().to(create_schedule_api),
        )
        .route(
            "/api/doctors/schedules/{schedule_id}",
            web::delete().to(delete_schedule_api),
        );
}

pub async fn doctors_page(
    req: HttpRequest,
    doctor_service: web::Data<DoctorService>,
    templates: web::Data<Tera>,
    firebase_auth: web::Data<FirebaseAuth>,
    firestore_db: web::Data<FirebaseRestDb>,
) -> impl Responder {
    if let Err(rejection) = require_doctor_admin(&req, &firebase_auth, &firestore_db).await {
        return rejection;
    }

    match doctor_service.list_doctors() {
        Ok(doctors) => {
            let mut context = Context::new();
            context.insert("doctors", &doctors);
            render_template(&templates, "doctors/index.html", &context)
        }
        Err(error) => render_error(&templates, error),
    }
}

pub async fn show_doctor(
    req: HttpRequest,
    doctor_service: web::Data<DoctorService>,
    templates: web::Data<Tera>,
    path: web::Path<String>,
    firebase_auth: web::Data<FirebaseAuth>,
    firestore_db: web::Data<FirebaseRestDb>,
) -> impl Responder {
    if let Err(rejection) = require_doctor_admin(&req, &firebase_auth, &firestore_db).await {
        return rejection;
    }

    let doctor_id = path.into_inner();
    match doctor_service.find_doctor(&doctor_id) {
        Ok(doctor) => {
            let mut context = Context::new();
            context.insert("doctor", &doctor);
            context.insert(
                "schedules",
                &doctor_service.list_schedules(&doctor_id).unwrap_or_default(),
            );
            render_template(&templates, "doctors/show.html", &context)
        }
        Err(error) => render_error(&templates, error),
    }
}

pub async fn create_doctor_form(
    req: HttpRequest,
    doctor_service: web::Data<DoctorService>,
    templates: web::Data<Tera>,
    form: web::Form<CreateDoctorForm>,
    firebase_auth: web::Data<FirebaseAuth>,
    firestore_db: web::Data<FirebaseRestDb>,
) -> impl Responder {
    if let Err(rejection) = require_doctor_admin(&req, &firebase_auth, &firestore_db).await {
        return rejection;
    }

    match doctor_service.create_doctor(form.into_inner()) {
        Ok(_) => redirect_to("/doctors"),
        Err(error) => render_error(&templates, error),
    }
}

pub async fn update_doctor_form(
    req: HttpRequest,
    doctor_service: web::Data<DoctorService>,
    templates: web::Data<Tera>,
    path: web::Path<String>,
    form: web::Form<UpdateDoctorForm>,
    firebase_auth: web::Data<FirebaseAuth>,
    firestore_db: web::Data<FirebaseRestDb>,
) -> impl Responder {
    if let Err(rejection) = require_doctor_admin(&req, &firebase_auth, &firestore_db).await {
        return rejection;
    }

    let doctor_id = path.into_inner();
    match doctor_service.update_doctor(&doctor_id, form.into_inner()) {
        Ok(_) => redirect_to(&format!("/doctors/{doctor_id}")),
        Err(error) => render_error(&templates, error),
    }
}

pub async fn delete_doctor_form(
    req: HttpRequest,
    doctor_service: web::Data<DoctorService>,
    templates: web::Data<Tera>,
    path: web::Path<String>,
    firebase_auth: web::Data<FirebaseAuth>,
    firestore_db: web::Data<FirebaseRestDb>,
) -> impl Responder {
    if let Err(rejection) = require_doctor_admin(&req, &firebase_auth, &firestore_db).await {
        return rejection;
    }

    match doctor_service.delete_doctor(&path.into_inner()) {
        Ok(_) => redirect_to("/doctors"),
        Err(error) => render_error(&templates, error),
    }
}

pub async fn create_schedule_form(
    req: HttpRequest,
    doctor_service: web::Data<DoctorService>,
    templates: web::Data<Tera>,
    path: web::Path<String>,
    form: web::Form<CreateDoctorScheduleForm>,
    firebase_auth: web::Data<FirebaseAuth>,
    firestore_db: web::Data<FirebaseRestDb>,
) -> impl Responder {
    if let Err(rejection) = require_doctor_admin(&req, &firebase_auth, &firestore_db).await {
        return rejection;
    }

    let doctor_id = path.into_inner();
    match doctor_service.create_schedule(&doctor_id, form.into_inner()) {
        Ok(_) => redirect_to(&format!("/doctors/{doctor_id}")),
        Err(error) => render_error(&templates, error),
    }
}

pub async fn delete_schedule_form(
    req: HttpRequest,
    doctor_service: web::Data<DoctorService>,
    templates: web::Data<Tera>,
    path: web::Path<String>,
    firebase_auth: web::Data<FirebaseAuth>,
    firestore_db: web::Data<FirebaseRestDb>,
) -> impl Responder {
    if let Err(rejection) = require_doctor_admin(&req, &firebase_auth, &firestore_db).await {
        return rejection;
    }

    match doctor_service.delete_schedule(&path.into_inner()) {
        Ok(_) => redirect_to("/doctors"),
        Err(error) => render_error(&templates, error),
    }
}

pub async fn list_doctors_api(
    req: HttpRequest,
    doctor_service: web::Data<DoctorService>,
    firebase_auth: web::Data<FirebaseAuth>,
    firestore_db: web::Data<FirebaseRestDb>,
) -> impl Responder {
    if let Err(rejection) = require_doctor_admin(&req, &firebase_auth, &firestore_db).await {
        return rejection;
    }

    match doctor_service.list_doctors() {
        Ok(doctors) => HttpResponse::Ok().json(doctors),
        Err(error) => doctor_error_response(error),
    }
}

pub async fn show_doctor_api(
    req: HttpRequest,
    doctor_service: web::Data<DoctorService>,
    path: web::Path<String>,
    firebase_auth: web::Data<FirebaseAuth>,
    firestore_db: web::Data<FirebaseRestDb>,
) -> impl Responder {
    if let Err(rejection) = require_doctor_admin(&req, &firebase_auth, &firestore_db).await {
        return rejection;
    }

    match doctor_service.find_doctor(&path.into_inner()) {
        Ok(doctor) => HttpResponse::Ok().json(doctor),
        Err(error) => doctor_error_response(error),
    }
}

pub async fn create_doctor_api(
    req: HttpRequest,
    doctor_service: web::Data<DoctorService>,
    form: web::Json<CreateDoctorForm>,
    firebase_auth: web::Data<FirebaseAuth>,
    firestore_db: web::Data<FirebaseRestDb>,
) -> impl Responder {
    if let Err(rejection) = require_doctor_admin(&req, &firebase_auth, &firestore_db).await {
        return rejection;
    }

    match doctor_service.create_doctor(form.into_inner()) {
        Ok(doctor) => HttpResponse::Created().json(doctor),
        Err(error) => doctor_error_response(error),
    }
}

pub async fn update_doctor_api(
    req: HttpRequest,
    doctor_service: web::Data<DoctorService>,
    path: web::Path<String>,
    form: web::Json<UpdateDoctorForm>,
    firebase_auth: web::Data<FirebaseAuth>,
    firestore_db: web::Data<FirebaseRestDb>,
) -> impl Responder {
    if let Err(rejection) = require_doctor_admin(&req, &firebase_auth, &firestore_db).await {
        return rejection;
    }

    match doctor_service.update_doctor(&path.into_inner(), form.into_inner()) {
        Ok(doctor) => HttpResponse::Ok().json(doctor),
        Err(error) => doctor_error_response(error),
    }
}

pub async fn delete_doctor_api(
    req: HttpRequest,
    doctor_service: web::Data<DoctorService>,
    path: web::Path<String>,
    firebase_auth: web::Data<FirebaseAuth>,
    firestore_db: web::Data<FirebaseRestDb>,
) -> impl Responder {
    if let Err(rejection) = require_doctor_admin(&req, &firebase_auth, &firestore_db).await {
        return rejection;
    }

    match doctor_service.delete_doctor(&path.into_inner()) {
        Ok(_) => HttpResponse::Ok().json(json!({ "status": "success" })),
        Err(error) => doctor_error_response(error),
    }
}

pub async fn list_schedules_api(
    req: HttpRequest,
    doctor_service: web::Data<DoctorService>,
    path: web::Path<String>,
    firebase_auth: web::Data<FirebaseAuth>,
    firestore_db: web::Data<FirebaseRestDb>,
) -> impl Responder {
    if let Err(rejection) = require_doctor_admin(&req, &firebase_auth, &firestore_db).await {
        return rejection;
    }

    match doctor_service.list_schedules(&path.into_inner()) {
        Ok(schedules) => HttpResponse::Ok().json(schedules),
        Err(error) => doctor_error_response(error),
    }
}

pub async fn create_schedule_api(
    req: HttpRequest,
    doctor_service: web::Data<DoctorService>,
    path: web::Path<String>,
    form: web::Json<CreateDoctorScheduleForm>,
    firebase_auth: web::Data<FirebaseAuth>,
    firestore_db: web::Data<FirebaseRestDb>,
) -> impl Responder {
    if let Err(rejection) = require_doctor_admin(&req, &firebase_auth, &firestore_db).await {
        return rejection;
    }

    match doctor_service.create_schedule(&path.into_inner(), form.into_inner()) {
        Ok(schedule) => HttpResponse::Created().json(schedule),
        Err(error) => doctor_error_response(error),
    }
}

pub async fn delete_schedule_api(
    req: HttpRequest,
    doctor_service: web::Data<DoctorService>,
    path: web::Path<String>,
    firebase_auth: web::Data<FirebaseAuth>,
    firestore_db: web::Data<FirebaseRestDb>,
) -> impl Responder {
    if let Err(rejection) = require_doctor_admin(&req, &firebase_auth, &firestore_db).await {
        return rejection;
    }

    match doctor_service.delete_schedule(&path.into_inner()) {
        Ok(_) => HttpResponse::Ok().json(json!({ "status": "success" })),
        Err(error) => doctor_error_response(error),
    }
}

async fn require_doctor_admin(
    req: &HttpRequest,
    firebase_auth: &FirebaseAuth,
    firestore_db: &FirebaseRestDb,
) -> Result<(String, String), HttpResponse> {
    require_auth_and_permission(req, firebase_auth, firestore_db, AppAction::ManageUsers).await
}

fn render_template(templates: &Tera, template_name: &str, context: &Context) -> HttpResponse {
    match templates.render(template_name, context) {
        Ok(body) => HttpResponse::Ok().content_type("text/html").body(body),
        Err(error) => HttpResponse::InternalServerError()
            .content_type("text/plain")
            .body(format!("Template error: {error}")),
    }
}

fn render_error(templates: &Tera, error: DoctorError) -> HttpResponse {
    let mut context = Context::new();
    context.insert("message", &doctor_error_message(error));
    render_template(templates, "error.html", &context)
}

fn doctor_error_response(error: DoctorError) -> HttpResponse {
    let message = doctor_error_message(error);
    if message == "Doctor was not found." {
        HttpResponse::NotFound().body(message)
    } else if message == "Doctor storage is unavailable." {
        HttpResponse::InternalServerError().body(message)
    } else {
        HttpResponse::BadRequest().body(message)
    }
}

fn doctor_error_message(error: DoctorError) -> String {
    match error {
        DoctorError::InvalidInput(message) => message,
        DoctorError::DoctorNotFound => "Doctor was not found.".to_string(),
        DoctorError::StorageUnavailable => "Doctor storage is unavailable.".to_string(),
    }
}

fn redirect_to(location: &str) -> HttpResponse {
    HttpResponse::SeeOther()
        .insert_header((header::LOCATION, location))
        .finish()
}
