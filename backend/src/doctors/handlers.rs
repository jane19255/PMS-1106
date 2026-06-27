use super::service::{
    CreateDoctorForm, CreateDoctorScheduleForm, DoctorError, DoctorService, UpdateDoctorForm,
};
use crate::db::FirebaseRestDb;
use crate::handlers::auth::{require_auth_and_permission, AppAction};
use actix_web::http::header;
use actix_web::{web, HttpRequest, HttpResponse, Responder};
use firebase_auth::FirebaseAuth;
use serde::Deserialize;
use serde_json::json;
use tera::{Context, Tera};

pub fn routes(config: &mut web::ServiceConfig) {
    config
        .route("/doctors", web::get().to(doctors_page))
        .route("/doctors", web::post().to(create_doctor_form))
        .route("/doctors/{doctor_id}", web::get().to(show_doctor))
        .route("/doctors/{doctor_id}", web::post().to(update_doctor_form))
        .route(
            "/doctors/{doctor_id}/delete",
            web::post().to(delete_doctor_form),
        )
        .route(
            "/doctors/{doctor_id}/undo-delete",
            web::post().to(undo_delete_doctor_form),
        )
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
        .route(
            "/api/doctors/{doctor_id}",
            web::delete().to(delete_doctor_api),
        )
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

#[derive(Deserialize)]
pub struct DoctorsPageQuery {
    deleted_doctor_id: Option<String>,
    restored: Option<String>,
}

pub async fn doctors_page(
    req: HttpRequest,
    doctor_service: web::Data<DoctorService>,
    templates: web::Data<Tera>,
    query: web::Query<DoctorsPageQuery>,
    firebase_auth: web::Data<FirebaseAuth>,
    firestore_db: web::Data<FirebaseRestDb>,
) -> impl Responder {
    if let Err(rejection) = require_doctor_admin(&req, &firebase_auth, &firestore_db).await {
        return rejection;
    }

    match doctor_service.list_doctors().await {
        Ok(doctors) => render_doctors_page(
            &templates,
            doctors,
            None,
            None,
            query.deleted_doctor_id.as_deref(),
            query.restored.as_deref(),
        ),
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
    match doctor_service.find_doctor(&doctor_id).await {
        Ok(doctor) => render_doctor_detail_page(
            &templates,
            doctor,
            doctor_service
                .list_schedules(&doctor_id)
                .await
                .unwrap_or_default(),
            None,
            None,
        ),
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

    let submitted_form = form.into_inner();

    match doctor_service.create_doctor(submitted_form.clone()).await {
        Ok(_) => redirect_to("/doctors"),
        Err(error @ DoctorError::InvalidInput(_)) => match doctor_service.list_doctors().await {
            Ok(doctors) => {
                render_doctors_page(
                    &templates,
                    doctors,
                    Some(&submitted_form),
                    Some(error),
                    None,
                    None,
                )
            }
            Err(error) => render_error(&templates, error),
        },
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
    match doctor_service.update_doctor(&doctor_id, form.into_inner()).await {
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

    let doctor_id = path.into_inner();
    match doctor_service.delete_doctor(&doctor_id).await {
        Ok(_) => redirect_to(&format!("/doctors?deleted_doctor_id={doctor_id}")),
        Err(error) => render_error(&templates, error),
    }
}

pub async fn undo_delete_doctor_form(
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

    match doctor_service.undo_delete_doctor(&path.into_inner()).await {
        Ok(_) => redirect_to("/doctors?restored=1"),
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
    let submitted_form = form.into_inner();

    match doctor_service.create_schedule(&doctor_id, submitted_form.clone()).await {
        Ok(_) => redirect_to(&format!("/doctors/{doctor_id}")),
        Err(error @ DoctorError::InvalidInput(_)) => match doctor_service.find_doctor(&doctor_id).await {
            Ok(doctor) => render_doctor_detail_page(
                &templates,
                doctor,
                doctor_service
                    .list_schedules(&doctor_id)
                    .await
                    .unwrap_or_default(),
                Some(&submitted_form),
                Some(error),
            ),
            Err(error) => render_error(&templates, error),
        },
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

    match doctor_service.delete_schedule(&path.into_inner()).await {
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
    if let Err(rejection) = require_doctor_view(&req, &firebase_auth, &firestore_db).await {
        return rejection;
    }

    match doctor_service.list_doctors().await {
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
    if let Err(rejection) = require_doctor_view(&req, &firebase_auth, &firestore_db).await {
        return rejection;
    }

    match doctor_service.find_doctor(&path.into_inner()).await {
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

    match doctor_service.create_doctor(form.into_inner()).await {
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

    match doctor_service.update_doctor(&path.into_inner(), form.into_inner()).await {
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

    match doctor_service.delete_doctor(&path.into_inner()).await {
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
    if let Err(rejection) = require_doctor_view(&req, &firebase_auth, &firestore_db).await {
        return rejection;
    }

    match doctor_service.list_schedules(&path.into_inner()).await {
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

    match doctor_service.create_schedule(&path.into_inner(), form.into_inner()).await {
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

    match doctor_service.delete_schedule(&path.into_inner()).await {
        Ok(_) => HttpResponse::Ok().json(json!({ "status": "success" })),
        Err(error) => doctor_error_response(error),
    }
}

async fn require_doctor_view(
    req: &HttpRequest,
    firebase_auth: &FirebaseAuth,
    firestore_db: &FirebaseRestDb,
) -> Result<(String, String), HttpResponse> {
    require_auth_and_permission(req, firebase_auth, firestore_db, AppAction::ViewPatient).await
}
async fn require_doctor_admin(
    req: &HttpRequest,
    firebase_auth: &FirebaseAuth,
    firestore_db: &FirebaseRestDb,
) -> Result<(String, String), HttpResponse> {
    require_auth_and_permission(req, firebase_auth, firestore_db, AppAction::ManageUsers).await
}

fn render_doctor_detail_page(
    templates: &Tera,
    doctor: super::models::Doctor,
    schedules: Vec<super::models::DoctorSchedule>,
    schedule_form: Option<&CreateDoctorScheduleForm>,
    error: Option<DoctorError>,
) -> HttpResponse {
    let mut context = Context::new();
    context.insert("doctor", &doctor);
    context.insert("schedules", &schedules);

    if let Some(form) = schedule_form {
        let day_of_week = format!("{:?}", form.day_of_week);
        context.insert("schedule_day_of_week", &day_of_week);
        context.insert("schedule_start_time", &form.start_time);
        context.insert("schedule_end_time", &form.end_time);
    }

    if let Some(error) = error {
        let message = doctor_error_message(error);
        context.insert("schedule_error_message", &message);
        context.insert(
            "invalid_schedule_start_time",
            &(message == "Start time must use HH:MM format"
                || message == "Start time must be before end time"),
        );
        context.insert(
            "invalid_schedule_end_time",
            &(message == "End time must use HH:MM format"
                || message == "Start time must be before end time"),
        );
    }

    render_template(templates, "doctors/show.html", &context)
}
fn render_doctors_page(
    templates: &Tera,
    doctors: Vec<super::models::Doctor>,
    form: Option<&CreateDoctorForm>,
    error: Option<DoctorError>,
    deleted_doctor_id: Option<&str>,
    restored: Option<&str>,
) -> HttpResponse {
    let mut context = Context::new();
    context.insert("doctors", &doctors);

    if let Some(doctor_id) = deleted_doctor_id {
        context.insert("deleted_doctor_id", doctor_id);
        context.insert(
            "success_message",
            "Doctor deleted. You can undo this action while the server is still running.",
        );
    } else if restored == Some("1") {
        context.insert("success_message", "Doctor restored successfully.");
    }

    if let Some(form) = form {
        context.insert("form_staff_id", &form.staff_id);
        context.insert("form_license_number", &form.license_number);
        context.insert("form_name", &form.name);
        context.insert("form_specialization", &form.specialization);
        context.insert("form_contact_number", &form.contact_number);
        context.insert("form_email", &form.email);
    }

    if let Some(error) = error {
        let message = doctor_error_message(error);
        context.insert("error_message", &message);
        context.insert("invalid_staff_id", &(message == "Staff ID is required"));
        context.insert("invalid_name", &(message == "Doctor name is required"));
        context.insert(
            "invalid_specialization",
            &(message == "Specialization is required"),
        );
        context.insert(
            "invalid_contact_number",
            &(message == "Contact number is required"),
        );
        context.insert("invalid_email", &(message == "Email is required"));
    }

    render_template(templates, "doctors/index.html", &context)
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
    context.insert("back_url", "/doctors");
    context.insert("back_label", "Back to Doctor Management");
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
