//! Prescription HTTP handlers.
//!
//! Serves prescription pages and APIs for creating, updating, dispensing, cancelling,
//! and listing prescriptions.
use super::service::{
    CreatePrescriptionForm, PrescriptionError, PrescriptionService, UpdatePrescriptionForm,
};
use crate::db::FirebaseRestDb;
use crate::auth::{require_auth_and_permission, AppAction};
use actix_web::http::header;
use actix_web::{web, HttpRequest, HttpResponse, Responder};
use firebase_auth::FirebaseAuth;
use serde::Deserialize;
use tera::{Context, Tera};

pub fn routes(config: &mut web::ServiceConfig) {
    config
        .route("/prescriptions", web::get().to(prescriptions_page))
        .route("/prescriptions", web::post().to(create_prescription_form))
        .route(
            "/prescriptions/{prescription_id}",
            web::get().to(show_prescription),
        )
        .route(
            "/prescriptions/{prescription_id}",
            web::post().to(update_prescription_form),
        )
        .route(
            "/prescriptions/{prescription_id}/dispense",
            web::post().to(dispense_prescription_form),
        )
        .route(
            "/prescriptions/{prescription_id}/cancel",
            web::post().to(cancel_prescription_form),
        )
        .route("/api/prescriptions", web::get().to(list_prescriptions_api))
        .route(
            "/api/prescriptions",
            web::post().to(create_prescription_api),
        )
        .route(
            "/api/prescriptions/{prescription_id}",
            web::get().to(show_prescription_api),
        )
        .route(
            "/api/prescriptions/{prescription_id}",
            web::put().to(update_prescription_api),
        )
        .route(
            "/api/prescriptions/{prescription_id}/dispense",
            web::post().to(dispense_prescription_api),
        )
        .route(
            "/api/prescriptions/{prescription_id}/cancel",
            web::post().to(cancel_prescription_api),
        );
}

pub async fn prescriptions_page(
    req: HttpRequest,
    prescription_service: web::Data<PrescriptionService>,
    templates: web::Data<Tera>,
    firebase_auth: web::Data<FirebaseAuth>,
    firestore_db: web::Data<FirebaseRestDb>,
) -> impl Responder {
    if let Err(rejection) = require_prescription_permission(
        &req,
        &firebase_auth,
        &firestore_db,
        AppAction::ViewPrescriptions,
    )
    .await
    {
        return rejection;
    }

    match prescription_service.list_prescriptions().await {
        Ok(prescriptions) => render_prescriptions_page(&templates, prescriptions, None, None),
        Err(error) => render_error(&templates, error),
    }
}

pub async fn show_prescription(
    req: HttpRequest,
    prescription_service: web::Data<PrescriptionService>,
    templates: web::Data<Tera>,
    path: web::Path<String>,
    firebase_auth: web::Data<FirebaseAuth>,
    firestore_db: web::Data<FirebaseRestDb>,
) -> impl Responder {
    if let Err(rejection) = require_prescription_permission(
        &req,
        &firebase_auth,
        &firestore_db,
        AppAction::ViewPrescriptions,
    )
    .await
    {
        return rejection;
    }

    match prescription_service.find_prescription(&path.into_inner()).await {
        Ok(prescription) => {
            let mut context = Context::new();
            context.insert("prescription", &prescription);
            render_template(&templates, "prescriptions/show.html", &context)
        }
        Err(error) => render_error(&templates, error),
    }
}

pub async fn create_prescription_form(
    req: HttpRequest,
    prescription_service: web::Data<PrescriptionService>,
    templates: web::Data<Tera>,
    form: web::Form<CreatePrescriptionForm>,
    firebase_auth: web::Data<FirebaseAuth>,
    firestore_db: web::Data<FirebaseRestDb>,
) -> impl Responder {
    if let Err(rejection) = require_prescription_permission(
        &req,
        &firebase_auth,
        &firestore_db,
        AppAction::CreatePrescription,
    )
    .await
    {
        return rejection;
    }

    let submitted_form = form.into_inner();

    match prescription_service.create_prescription(submitted_form.clone()).await {
        Ok(_) => redirect_to("/prescriptions"),
        Err(error @ PrescriptionError::InvalidInput(_)) => match prescription_service.list_prescriptions().await {
            Ok(prescriptions) => render_prescriptions_page(
                &templates,
                prescriptions,
                Some(&submitted_form),
                Some(error),
            ),
            Err(error) => render_error(&templates, error),
        },
        Err(error) => render_error(&templates, error),
    }
}

pub async fn update_prescription_form(
    req: HttpRequest,
    prescription_service: web::Data<PrescriptionService>,
    templates: web::Data<Tera>,
    path: web::Path<String>,
    form: web::Form<UpdatePrescriptionForm>,
    firebase_auth: web::Data<FirebaseAuth>,
    firestore_db: web::Data<FirebaseRestDb>,
) -> impl Responder {
    if let Err(rejection) = require_prescription_permission(
        &req,
        &firebase_auth,
        &firestore_db,
        AppAction::CreatePrescription,
    )
    .await
    {
        return rejection;
    }

    let prescription_id = path.into_inner();
    match prescription_service.update_prescription(&prescription_id, form.into_inner()).await {
        Ok(_) => redirect_to(&format!("/prescriptions/{prescription_id}")),
        Err(error) => render_error(&templates, error),
    }
}

pub async fn dispense_prescription_form(
    req: HttpRequest,
    prescription_service: web::Data<PrescriptionService>,
    templates: web::Data<Tera>,
    path: web::Path<String>,
    firebase_auth: web::Data<FirebaseAuth>,
    firestore_db: web::Data<FirebaseRestDb>,
) -> impl Responder {
    if let Err(rejection) = require_prescription_permission(
        &req,
        &firebase_auth,
        &firestore_db,
        AppAction::DispenseMedicine,
    )
    .await
    {
        return rejection;
    }

    let prescription_id = path.into_inner();
    match prescription_service.dispense_prescription(&prescription_id).await {
        Ok(_) => redirect_to(&format!("/prescriptions/{prescription_id}")),
        Err(error) => render_error(&templates, error),
    }
}

pub async fn cancel_prescription_form(
    req: HttpRequest,
    prescription_service: web::Data<PrescriptionService>,
    templates: web::Data<Tera>,
    path: web::Path<String>,
    firebase_auth: web::Data<FirebaseAuth>,
    firestore_db: web::Data<FirebaseRestDb>,
) -> impl Responder {
    if let Err(rejection) = require_prescription_permission(
        &req,
        &firebase_auth,
        &firestore_db,
        AppAction::CreatePrescription,
    )
    .await
    {
        return rejection;
    }

    let prescription_id = path.into_inner();
    match prescription_service.cancel_prescription(&prescription_id).await {
        Ok(_) => redirect_to(&format!("/prescriptions/{prescription_id}")),
        Err(error) => render_error(&templates, error),
    }
}

#[derive(Deserialize)]
pub struct PrescriptionListQuery {
    pub patient_id: Option<String>,
    pub doctor_id: Option<String>,
}

pub async fn list_prescriptions_api(
    req: HttpRequest,
    prescription_service: web::Data<PrescriptionService>,
    query: web::Query<PrescriptionListQuery>,
    firebase_auth: web::Data<FirebaseAuth>,
    firestore_db: web::Data<FirebaseRestDb>,
) -> impl Responder {
    if let Err(rejection) = require_prescription_permission(
        &req,
        &firebase_auth,
        &firestore_db,
        AppAction::ViewPrescriptions,
    )
    .await
    {
        return rejection;
    }

    let prescriptions = if let Some(patient_id) = query.patient_id.as_deref() {
        prescription_service.list_prescriptions_for_patient(patient_id).await
    } else if let Some(doctor_id) = query.doctor_id.as_deref() {
        prescription_service.list_prescriptions_for_doctor(doctor_id).await
    } else {
        prescription_service.list_prescriptions().await
    };

    match prescriptions {
        Ok(prescriptions) => HttpResponse::Ok().json(prescriptions),
        Err(error) => prescription_error_response(error),
    }
}

pub async fn show_prescription_api(
    req: HttpRequest,
    prescription_service: web::Data<PrescriptionService>,
    path: web::Path<String>,
    firebase_auth: web::Data<FirebaseAuth>,
    firestore_db: web::Data<FirebaseRestDb>,
) -> impl Responder {
    if let Err(rejection) = require_prescription_permission(
        &req,
        &firebase_auth,
        &firestore_db,
        AppAction::ViewPrescriptions,
    )
    .await
    {
        return rejection;
    }

    match prescription_service.find_prescription(&path.into_inner()).await {
        Ok(prescription) => HttpResponse::Ok().json(prescription),
        Err(error) => prescription_error_response(error),
    }
}

pub async fn create_prescription_api(
    req: HttpRequest,
    prescription_service: web::Data<PrescriptionService>,
    form: web::Json<CreatePrescriptionForm>,
    firebase_auth: web::Data<FirebaseAuth>,
    firestore_db: web::Data<FirebaseRestDb>,
) -> impl Responder {
    if let Err(rejection) = require_prescription_permission(
        &req,
        &firebase_auth,
        &firestore_db,
        AppAction::CreatePrescription,
    )
    .await
    {
        return rejection;
    }

    match prescription_service.create_prescription(form.into_inner()).await {
        Ok(prescription) => HttpResponse::Created().json(prescription),
        Err(error) => prescription_error_response(error),
    }
}

pub async fn update_prescription_api(
    req: HttpRequest,
    prescription_service: web::Data<PrescriptionService>,
    path: web::Path<String>,
    form: web::Json<UpdatePrescriptionForm>,
    firebase_auth: web::Data<FirebaseAuth>,
    firestore_db: web::Data<FirebaseRestDb>,
) -> impl Responder {
    if let Err(rejection) = require_prescription_permission(
        &req,
        &firebase_auth,
        &firestore_db,
        AppAction::CreatePrescription,
    )
    .await
    {
        return rejection;
    }

    match prescription_service.update_prescription(&path.into_inner(), form.into_inner()).await {
        Ok(prescription) => HttpResponse::Ok().json(prescription),
        Err(error) => prescription_error_response(error),
    }
}

pub async fn dispense_prescription_api(
    req: HttpRequest,
    prescription_service: web::Data<PrescriptionService>,
    path: web::Path<String>,
    firebase_auth: web::Data<FirebaseAuth>,
    firestore_db: web::Data<FirebaseRestDb>,
) -> impl Responder {
    if let Err(rejection) = require_prescription_permission(
        &req,
        &firebase_auth,
        &firestore_db,
        AppAction::DispenseMedicine,
    )
    .await
    {
        return rejection;
    }

    match prescription_service.dispense_prescription(&path.into_inner()).await {
        Ok(prescription) => HttpResponse::Ok().json(prescription),
        Err(error) => prescription_error_response(error),
    }
}

pub async fn cancel_prescription_api(
    req: HttpRequest,
    prescription_service: web::Data<PrescriptionService>,
    path: web::Path<String>,
    firebase_auth: web::Data<FirebaseAuth>,
    firestore_db: web::Data<FirebaseRestDb>,
) -> impl Responder {
    if let Err(rejection) = require_prescription_permission(
        &req,
        &firebase_auth,
        &firestore_db,
        AppAction::CreatePrescription,
    )
    .await
    {
        return rejection;
    }

    match prescription_service.cancel_prescription(&path.into_inner()).await {
        Ok(prescription) => HttpResponse::Ok().json(prescription),
        Err(error) => prescription_error_response(error),
    }
}

async fn require_prescription_permission(
    req: &HttpRequest,
    firebase_auth: &FirebaseAuth,
    firestore_db: &FirebaseRestDb,
    action: AppAction,
) -> Result<(String, String), HttpResponse> {
    require_auth_and_permission(req, firebase_auth, firestore_db, action).await
}

fn render_prescriptions_page(
    templates: &Tera,
    prescriptions: Vec<super::models::Prescription>,
    form: Option<&CreatePrescriptionForm>,
    error: Option<PrescriptionError>,
) -> HttpResponse {
    let mut context = Context::new();
    context.insert("prescriptions", &prescriptions);

    if let Some(form) = form {
        context.insert("form_patient_id", &form.patient_id);
        context.insert("form_doctor_id", &form.doctor_id);
        context.insert(
            "form_medical_record_id",
            &form.medical_record_id.as_deref().unwrap_or_default(),
        );
        context.insert("form_medication_name", &form.medication_name);
        context.insert("form_dosage", &form.dosage);
        context.insert("form_frequency", &form.frequency);
        context.insert("form_duration", &form.duration);
        context.insert("form_instructions", &form.instructions);
        context.insert("form_quantity", &form.quantity);
        context.insert("form_unit_cost", &form.unit_cost);
    }

    if let Some(error) = error {
        let message = prescription_error_message(error);
        context.insert("error_message", &message);
        context.insert("show_create_modal", &true);
        context.insert("invalid_patient_id", &(message == "Patient ID is required"));
        context.insert("invalid_doctor_id", &(message == "Doctor ID is required"));
        context.insert("invalid_medication_name", &(message == "Medication name is required"));
        context.insert("invalid_dosage", &(message == "Dosage is required"));
        context.insert("invalid_frequency", &(message == "Frequency is required"));
        context.insert("invalid_duration", &(message == "Duration is required"));
        context.insert("invalid_quantity", &(message == "Quantity must be a whole number" || message == "Quantity must be greater than zero"));
        context.insert("invalid_unit_cost", &(message == "Unit cost must be a valid number" || message == "Unit cost cannot be negative"));
    }

    render_template(templates, "prescriptions/index.html", &context)
}
fn render_template(templates: &Tera, template_name: &str, context: &Context) -> HttpResponse {
    match templates.render(template_name, context) {
        Ok(body) => HttpResponse::Ok().content_type("text/html").body(body),
        Err(error) => HttpResponse::InternalServerError()
            .content_type("text/plain")
            .body(format!("Template error: {error}")),
    }
}

fn render_error(templates: &Tera, error: PrescriptionError) -> HttpResponse {
    let mut context = Context::new();
    context.insert("message", &prescription_error_message(error));
    context.insert("back_url", "/prescriptions");
    context.insert("back_label", "Back to Prescriptions");
    render_template(templates, "error.html", &context)
}

fn prescription_error_response(error: PrescriptionError) -> HttpResponse {
    let message = prescription_error_message(error);
    if message == "Prescription was not found." {
        HttpResponse::NotFound().body(message)
    } else if message == "Prescription storage is unavailable." {
        HttpResponse::InternalServerError().body(message)
    } else {
        HttpResponse::BadRequest().body(message)
    }
}

fn prescription_error_message(error: PrescriptionError) -> String {
    match error {
        PrescriptionError::InvalidInput(message) => message,
        PrescriptionError::PrescriptionNotFound => "Prescription was not found.".to_string(),
        PrescriptionError::StorageUnavailable => "Prescription storage is unavailable.".to_string(),
    }
}

fn redirect_to(location: &str) -> HttpResponse {
    HttpResponse::SeeOther()
        .insert_header((header::LOCATION, location))
        .finish()
}
