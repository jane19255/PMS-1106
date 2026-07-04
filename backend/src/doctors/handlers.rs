//! Doctor management HTTP handlers.
//!
//! Serves doctor pages and APIs for doctor profiles, schedules, and safe doctor
//! offboarding with future appointment reassignment.
use super::models::DoctorStatus;
use super::service::{
    CreateDoctorForm, CreateDoctorScheduleForm, DoctorError, DoctorService, UpdateDoctorForm,
};
use crate::appointments::handlers::validate_and_check_conflict;
use crate::db::{FirebaseRestDb, SupabaseRestDb};
use crate::firebase_admin::FirebaseAdmin;
use crate::auth::{require_auth_and_permission, AppAction};
use crate::staff::service::{StaffForm, StaffService};
use actix_web::http::header;
use actix_web::{web, HttpRequest, HttpResponse, Responder};
use firebase_auth::FirebaseAuth;
use serde::Deserialize;
use serde_json::json;
use tera::{Context, Tera};

/// Registers doctor management pages and doctor JSON API routes.
///
/// The page routes support normal form navigation under `/doctors`, while the
/// `/api/doctors...` routes are used by dynamic screens such as appointments,
/// staff management, and dashboard workflows.
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
            "/api/doctors/{doctor_id}/reassign",
            web::post().to(reassign_doctor_api),
        )
        // Doctor availability is represented by doctor schedule routes.
        // Appointment booking reads these schedules before accepting a time slot.
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
    deleted: Option<String>,
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
        Ok(doctors) => {
            render_doctors_page(&templates, doctors, None, None, query.deleted.as_deref())
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
    match doctor_service.find_doctor(&doctor_id).await {
        Ok(doctor) => match doctor_service.list_schedules(&doctor_id).await {
            Ok(schedules) => render_doctor_detail_page(&templates, doctor, schedules, None, None),
            Err(error) => render_error(&templates, error),
        },
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
        Ok(_) => redirect_to("/staff#doctor-management"),
        Err(error @ DoctorError::InvalidInput(_)) => match doctor_service.list_doctors().await {
            Ok(doctors) => render_doctors_page(
                &templates,
                doctors,
                Some(&submitted_form),
                Some(error),
                None,
            ),
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
    match doctor_service
        .update_doctor(&doctor_id, form.into_inner())
        .await
    {
        Ok(_) => redirect_to(&format!("/doctors/{doctor_id}")),
        Err(error) => render_error(&templates, error),
    }
}

// Actix supplies these request, service, template and authentication values.
#[allow(clippy::too_many_arguments)]
pub async fn delete_doctor_form(
    req: HttpRequest,
    doctor_service: web::Data<DoctorService>,
    staff_service: web::Data<StaffService>,
    firebase_admin: web::Data<FirebaseAdmin>,
    templates: web::Data<Tera>,
    path: web::Path<String>,
    firebase_auth: web::Data<FirebaseAuth>,
    firestore_db: web::Data<FirebaseRestDb>,
) -> impl Responder {
    if let Err(rejection) = require_doctor_admin(&req, &firebase_auth, &firestore_db).await {
        return rejection;
    }

    let doctor_id = path.into_inner();
    match delete_doctor_account(&doctor_service, &staff_service, &firebase_admin, &doctor_id).await
    {
        Ok(_) => redirect_to("/staff#doctor-management"),
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

    match doctor_service
        .create_schedule(&doctor_id, submitted_form.clone())
        .await
    {
        Ok(_) => redirect_to(&format!("/doctors/{doctor_id}")),
        Err(error @ DoctorError::InvalidInput(_)) => {
            match doctor_service.find_doctor(&doctor_id).await {
                Ok(doctor) => match doctor_service.list_schedules(&doctor_id).await {
                    Ok(schedules) => render_doctor_detail_page(
                        &templates,
                        doctor,
                        schedules,
                        Some(&submitted_form),
                        Some(error),
                    ),
                    Err(error) => render_error(&templates, error),
                },
                Err(error) => render_error(&templates, error),
            }
        }
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
        Ok(_) => redirect_to("/staff#doctor-management"),
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

    match doctor_service
        .update_doctor(&path.into_inner(), form.into_inner())
        .await
    {
        Ok(doctor) => HttpResponse::Ok().json(doctor),
        Err(error) => doctor_error_response(error),
    }
}

pub async fn delete_doctor_api(
    req: HttpRequest,
    doctor_service: web::Data<DoctorService>,
    staff_service: web::Data<StaffService>,
    firebase_admin: web::Data<FirebaseAdmin>,
    path: web::Path<String>,
    firebase_auth: web::Data<FirebaseAuth>,
    firestore_db: web::Data<FirebaseRestDb>,
) -> impl Responder {
    if let Err(rejection) = require_doctor_admin(&req, &firebase_auth, &firestore_db).await {
        return rejection;
    }

    match delete_doctor_account(
        &doctor_service,
        &staff_service,
        &firebase_admin,
        &path.into_inner(),
    )
    .await
    {
        Ok(_) => HttpResponse::Ok().json(json!({ "status": "success" })),
        Err(error) => doctor_error_response(error),
    }
}

async fn delete_doctor_account(
    doctor_service: &DoctorService,
    staff_service: &StaffService,
    firebase_admin: &FirebaseAdmin,
    doctor_id: &str,
) -> Result<(), DoctorError> {
    let doctor = doctor_service.find_doctor(doctor_id).await?;
    let staff = staff_service
        .list_staff()
        .await
        .map_err(|_| DoctorError::StorageUnavailable)?
        .into_iter()
        .find(|staff| staff.id == doctor.staff_id)
        .ok_or(DoctorError::StorageUnavailable)?;

    // Delete the doctor first because staff is the parent record in Supabase.
    doctor_service.delete_doctor(doctor_id).await?;
    staff_service
        .delete_staff(&staff.id)
        .await
        .map_err(|_| DoctorError::StorageUnavailable)?;

    if let Err(error) = firebase_admin.delete_user(&staff.firebase_uid).await {
        eprintln!("Firebase user deletion failed: {error}");
    }
    Ok(())
}

#[derive(Deserialize)]
pub struct ReassignDoctorForm {
    pub target_doctor_id: String,
}

#[allow(clippy::too_many_arguments)]
/// Reassigns a doctor's future scheduled appointments before deactivating the
/// doctor account.
///
/// This is the safer offboarding route: historical consultations remain linked
/// to the original doctor, while upcoming work is moved to another available
/// doctor.
pub async fn reassign_doctor_api(
    req: HttpRequest,
    doctor_service: web::Data<DoctorService>,
    staff_service: web::Data<StaffService>,
    firebase_admin: web::Data<FirebaseAdmin>,
    supabase_db: web::Data<SupabaseRestDb>,
    path: web::Path<String>,
    payload: web::Json<ReassignDoctorForm>,
    firebase_auth: web::Data<FirebaseAuth>,
    firestore_db: web::Data<FirebaseRestDb>,
) -> impl Responder {
    if let Err(rejection) = require_doctor_admin(&req, &firebase_auth, &firestore_db).await {
        return rejection;
    }

    let doctor_id = path.into_inner();
    match reassign_and_deactivate_doctor(
        &doctor_service,
        &staff_service,
        &firebase_admin,
        &supabase_db,
        &doctor_id,
        payload.target_doctor_id.trim(),
    )
    .await
    {
        Ok(reassigned_count) => {
            HttpResponse::Ok().json(json!({ "success": true, "reassigned": reassigned_count }))
        }
        Err(error) => doctor_error_response(error),
    }
}

// Offboards a doctor safely: moves their still-`Scheduled` appointments to
// another doctor, then deactivates the doctor/staff/login instead of
// deleting them, so past visits stay attributed to the original doctor.
async fn reassign_and_deactivate_doctor(
    doctor_service: &DoctorService,
    staff_service: &StaffService,
    firebase_admin: &FirebaseAdmin,
    db: &SupabaseRestDb,
    doctor_id: &str,
    target_doctor_id: &str,
) -> Result<usize, DoctorError> {
    if target_doctor_id.is_empty() {
        return Err(DoctorError::InvalidInput(
            "Choose a doctor to reassign appointments to".to_string(),
        ));
    }
    if doctor_id == target_doctor_id {
        return Err(DoctorError::InvalidInput(
            "Choose a different doctor to reassign appointments to".to_string(),
        ));
    }

    let doctor = doctor_service.find_doctor(doctor_id).await?;
    if doctor_service.find_doctor(target_doctor_id).await.is_err() {
        return Err(DoctorError::InvalidInput(
            "Target doctor was not found".to_string(),
        ));
    }

    let appointments_json = db
        .list_appointments_by_doctor(doctor_id)
        .await
        .map_err(|_| DoctorError::StorageUnavailable)?;
    let appointments: Vec<serde_json::Value> =
        serde_json::from_str(&appointments_json).unwrap_or_default();

    let in_progress_count = appointments
        .iter()
        .filter(|appointment| {
            matches!(
                appointment.get("status").and_then(serde_json::Value::as_str),
                Some("Checked In" | "Vitals Recorded" | "In Consultation")
            )
        })
        .count();
    if in_progress_count > 0 {
        return Err(DoctorError::InvalidInput(format!(
            "{in_progress_count} appointment(s) are currently in progress for this doctor. \
             Complete or cancel them before offboarding."
        )));
    }

    let scheduled = appointments.into_iter().filter(|appointment| {
        appointment.get("status").and_then(serde_json::Value::as_str) == Some("Scheduled")
    });

    let mut failed_appointment_ids = Vec::new();
    let mut reassigned_count = 0usize;

    for mut appointment in scheduled {
        let Some(appointment_id) = appointment
            .get("id")
            .and_then(serde_json::Value::as_str)
            .map(str::to_string)
        else {
            continue;
        };

        if let Some(object) = appointment.as_object_mut() {
            object.insert("doctor_id".to_string(), json!(target_doctor_id));
        }

        let update_result = match validate_and_check_conflict(
            db,
            doctor_service,
            appointment,
            Some(&appointment_id),
        )
        .await
        {
            Ok(updated) => db.update_appointment(&appointment_id, &updated).await,
            Err(_) => Err("conflict".to_string()),
        };

        match update_result {
            Ok(_) => reassigned_count += 1,
            Err(_) => failed_appointment_ids.push(appointment_id),
        }
    }

    if !failed_appointment_ids.is_empty() {
        return Err(DoctorError::InvalidInput(format!(
            "Could not reassign {} appointment(s) because the target doctor already has a \
             conflicting booking: {}",
            failed_appointment_ids.len(),
            failed_appointment_ids.join(", ")
        )));
    }

    let staff = staff_service
        .list_staff()
        .await
        .map_err(|_| DoctorError::StorageUnavailable)?
        .into_iter()
        .find(|staff| staff.id == doctor.staff_id)
        .ok_or(DoctorError::StorageUnavailable)?;

    doctor_service
        .update_doctor(
            doctor_id,
            UpdateDoctorForm {
                staff_id: doctor.staff_id.clone(),
                license_number: doctor.license_number.clone(),
                name: doctor.name.clone(),
                specialization: doctor.specialization.clone(),
                contact_number: doctor.contact_number.clone(),
                email: doctor.email.clone(),
                status: DoctorStatus::Unavailable,
            },
        )
        .await?;

    staff_service
        .update_staff(
            &staff.id,
            StaffForm {
                firebase_uid: staff.firebase_uid.clone(),
                first_name: staff.first_name.clone(),
                last_name: staff.last_name.clone(),
                dob: staff.dob.clone(),
                gender: staff.gender.clone(),
                nric: staff.nric.clone(),
                role: staff.role.clone(),
                phone: staff.phone.clone(),
                email: staff.email.clone(),
                status: Some("Inactive".to_string()),
                address: staff.address.clone(),
                emergency: staff.emergency.clone(),
            },
        )
        .await
        .map_err(|_| DoctorError::StorageUnavailable)?;

    if let Err(error) = firebase_admin.disable_user(&staff.firebase_uid).await {
        eprintln!("Firebase user disable failed: {error}");
    }

    Ok(reassigned_count)
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

    match doctor_service
        .create_schedule(&path.into_inner(), form.into_inner())
        .await
    {
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
    let doctor_status = format!("{:?}", doctor.status).replace("OnLeave", "On Leave");
    context.insert("doctor", &doctor);
    context.insert("doctor_status", &doctor_status);
    context.insert("schedules", &schedules);
    context.insert("schedule_day_of_week", "");
    context.insert("schedule_start_time", "");
    context.insert("schedule_end_time", "");
    context.insert("invalid_schedule_start_time", &false);
    context.insert("invalid_schedule_end_time", &false);

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
    deleted: Option<&str>,
) -> HttpResponse {
    let mut context = Context::new();
    context.insert("doctors", &doctors);
    if deleted == Some("1") {
        context.insert("success_message", "Doctor deleted successfully.");
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
        context.insert(
            "invalid_license_number",
            &(message == "License number is required"),
        );
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
    context.insert("back_url", "/staff#doctor-management");
    context.insert("back_label", "Back to Staff Management");
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
        DoctorError::DoctorHasAppointments => {
            "Cannot delete this doctor because existing appointments reference them.".to_string()
        }
        DoctorError::DoctorNotFound => "Doctor was not found.".to_string(),
        DoctorError::StorageUnavailable => "Doctor storage is unavailable.".to_string(),
    }
}

fn redirect_to(location: &str) -> HttpResponse {
    HttpResponse::SeeOther()
        .insert_header((header::LOCATION, location))
        .finish()
}

