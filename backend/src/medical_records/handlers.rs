use super::service::{
    CreateMedicalRecordForm, MedicalRecordError, MedicalRecordService, UpdateMedicalRecordForm,
};
use crate::db::{FirebaseRestDb, SupabaseRestDb};
use crate::handlers::auth::{require_auth_and_permission, AppAction};
use crate::models::{PatientView, SupabasePatientRow};
use actix_web::http::header;
use actix_web::{web, HttpRequest, HttpResponse, Responder};
use firebase_auth::FirebaseAuth;
use serde::Deserialize;
use tera::{Context, Tera};

pub fn routes(config: &mut web::ServiceConfig) {
    config
        // SSR pages
        .route("/medical-records", web::get().to(list_records_page))
        .route(
            "/medical-records/new",
            web::get().to(new_record_page),
        )
        .route("/medical-records", web::post().to(create_record_form))
        .route(
            "/medical-records/{record_id}",
            web::get().to(show_record_page),
        )
        .route(
            "/medical-records/{record_id}",
            web::post().to(update_record_form),
        )
        .route(
            "/medical-records/{record_id}/delete",
            web::post().to(delete_record_form),
        )
        .route(
            "/patients/{patient_id}/timeline",
            web::get().to(patient_timeline_page),
        )
        // JSON API
        .route("/api/medical-records", web::get().to(list_records_api))
        .route("/api/medical-records", web::post().to(create_record_api))
        .route(
            "/api/medical-records/{record_id}",
            web::get().to(show_record_api),
        )
        .route(
            "/api/medical-records/{record_id}",
            web::put().to(update_record_api),
        )
        .route(
            "/api/medical-records/{record_id}",
            web::delete().to(delete_record_api),
        )
        .route(
            "/api/patients/{patient_id}/timeline",
            web::get().to(patient_timeline_api),
        );
}

// ── SSR page handlers ────────────────────────────────────────────────────────

pub async fn list_records_page(
    req: HttpRequest,
    service: web::Data<MedicalRecordService>,
    templates: web::Data<Tera>,
    firebase_auth: web::Data<FirebaseAuth>,
    firestore_db: web::Data<FirebaseRestDb>,
    query: web::Query<RecordListQuery>,
) -> impl Responder {
    if let Err(rejection) = require_records_permission(
        &req,
        &firebase_auth,
        &firestore_db,
        AppAction::ViewMedicalRecords,
    )
    .await
    {
        return rejection;
    }

    let records_result = if let Some(patient_id) = query.patient_id.as_deref() {
        service.list_records_for_patient(patient_id).await
    } else {
        service.list_records().await
    };

    match records_result {
        Ok(records) => {
            let mut context = Context::new();
            context.insert("records", &records);
            context.insert(
                "filter_patient_id",
                query.patient_id.as_deref().unwrap_or(""),
            );
            render_template(&templates, "medical_records/index.html", &context)
        }
        Err(error) => render_error(&templates, error),
    }
}

pub async fn new_record_page(
    req: HttpRequest,
    templates: web::Data<Tera>,
    firebase_auth: web::Data<FirebaseAuth>,
    firestore_db: web::Data<FirebaseRestDb>,
    query: web::Query<NewRecordQuery>,
    supabase_db: web::Data<SupabaseRestDb>,
) -> impl Responder {
    if let Err(rejection) = require_records_permission(
        &req,
        &firebase_auth,
        &firestore_db,
        AppAction::EditMedicalRecords,
    )
    .await
    {
        return rejection;
    }

    let patient_id = query.patient_id.as_deref().unwrap_or("");
    let appointment_id = query.appointment_id.as_deref().unwrap_or("");

    let mut context = Context::new();
    context.insert("prefill_patient_id", patient_id);
    context.insert("prefill_appointment_id", appointment_id);

    // Attempt to load patient info for display
    if !patient_id.is_empty() {
        if let Ok(patient) = fetch_patient(&supabase_db, patient_id).await {
            context.insert("patient", &patient);
        }
    }

    render_template(&templates, "medical_records/new.html", &context)
}

pub async fn create_record_form(
    req: HttpRequest,
    service: web::Data<MedicalRecordService>,
    templates: web::Data<Tera>,
    firebase_auth: web::Data<FirebaseAuth>,
    firestore_db: web::Data<FirebaseRestDb>,
    form: web::Form<CreateMedicalRecordForm>,
) -> impl Responder {
    if let Err(rejection) = require_records_permission(
        &req,
        &firebase_auth,
        &firestore_db,
        AppAction::EditMedicalRecords,
    )
    .await
    {
        return rejection;
    }

    let submitted = form.into_inner();
    let patient_id = submitted.patient_id.clone();

    match service.create_record(submitted).await {
        Ok(record) => redirect_to(&format!("/medical-records/{}", record.id)),
        Err(error @ MedicalRecordError::InvalidInput(_)) => {
            let mut context = Context::new();
            context.insert("error_message", &record_error_message(error));
            context.insert("prefill_patient_id", &patient_id);
            render_template(&templates, "medical_records/new.html", &context)
        }
        Err(error) => render_error(&templates, error),
    }
}

pub async fn show_record_page(
    req: HttpRequest,
    service: web::Data<MedicalRecordService>,
    templates: web::Data<Tera>,
    firebase_auth: web::Data<FirebaseAuth>,
    firestore_db: web::Data<FirebaseRestDb>,
    path: web::Path<String>,
    supabase_db: web::Data<SupabaseRestDb>,
) -> impl Responder {
    if let Err(rejection) = require_records_permission(
        &req,
        &firebase_auth,
        &firestore_db,
        AppAction::ViewMedicalRecords,
    )
    .await
    {
        return rejection;
    }

    let record_id = path.into_inner();
    match service.find_record(&record_id).await {
        Ok(record) => {
            let mut context = Context::new();

            // Load patient information for context
            if let Ok(patient) = fetch_patient(&supabase_db, &record.patient_id).await {
                context.insert("patient", &patient);
            }

            context.insert("record", &record);
            render_template(&templates, "medical_records/show.html", &context)
        }
        Err(error) => render_error(&templates, error),
    }
}

pub async fn update_record_form(
    req: HttpRequest,
    service: web::Data<MedicalRecordService>,
    templates: web::Data<Tera>,
    firebase_auth: web::Data<FirebaseAuth>,
    firestore_db: web::Data<FirebaseRestDb>,
    path: web::Path<String>,
    form: web::Form<UpdateMedicalRecordForm>,
) -> impl Responder {
    if let Err(rejection) = require_records_permission(
        &req,
        &firebase_auth,
        &firestore_db,
        AppAction::EditMedicalRecords,
    )
    .await
    {
        return rejection;
    }

    let record_id = path.into_inner();
    match service.update_record(&record_id, form.into_inner()).await {
        Ok(_) => redirect_to(&format!("/medical-records/{record_id}")),
        Err(error) => render_error(&templates, error),
    }
}

pub async fn delete_record_form(
    req: HttpRequest,
    service: web::Data<MedicalRecordService>,
    templates: web::Data<Tera>,
    firebase_auth: web::Data<FirebaseAuth>,
    firestore_db: web::Data<FirebaseRestDb>,
    path: web::Path<String>,
) -> impl Responder {
    if let Err(rejection) = require_records_permission(
        &req,
        &firebase_auth,
        &firestore_db,
        AppAction::EditMedicalRecords,
    )
    .await
    {
        return rejection;
    }

    let record_id = path.into_inner();
    match service.delete_record(&record_id).await {
        Ok(_) => redirect_to("/medical-records"),
        Err(error) => render_error(&templates, error),
    }
}

pub async fn patient_timeline_page(
    req: HttpRequest,
    service: web::Data<MedicalRecordService>,
    templates: web::Data<Tera>,
    firebase_auth: web::Data<FirebaseAuth>,
    firestore_db: web::Data<FirebaseRestDb>,
    path: web::Path<String>,
    supabase_db: web::Data<SupabaseRestDb>,
) -> impl Responder {
    if let Err(rejection) = require_records_permission(
        &req,
        &firebase_auth,
        &firestore_db,
        AppAction::ViewMedicalRecords,
    )
    .await
    {
        return rejection;
    }

    let patient_id = path.into_inner();
    let timeline_result = service.patient_timeline(&patient_id).await;
    let patient = fetch_patient(&supabase_db, &patient_id).await.ok();

    match timeline_result {
        Ok(entries) => {
            let mut context = Context::new();
            context.insert("entries", &entries);
            context.insert("patient_id", &patient_id);
            if let Some(patient) = patient {
                context.insert("patient", &patient);
            }
            render_template(&templates, "medical_records/timeline.html", &context)
        }
        Err(error) => render_error(&templates, error),
    }
}

// ── JSON API handlers ────────────────────────────────────────────────────────

pub async fn list_records_api(
    req: HttpRequest,
    service: web::Data<MedicalRecordService>,
    firebase_auth: web::Data<FirebaseAuth>,
    firestore_db: web::Data<FirebaseRestDb>,
    query: web::Query<RecordListQuery>,
) -> impl Responder {
    if let Err(rejection) = require_records_permission(
        &req,
        &firebase_auth,
        &firestore_db,
        AppAction::ViewMedicalRecords,
    )
    .await
    {
        return rejection;
    }

    let result = if let Some(patient_id) = query.patient_id.as_deref() {
        service.list_records_for_patient(patient_id).await
    } else {
        service.list_records().await
    };

    match result {
        Ok(records) => HttpResponse::Ok().json(records),
        Err(error) => record_error_response(error),
    }
}

pub async fn create_record_api(
    req: HttpRequest,
    service: web::Data<MedicalRecordService>,
    firebase_auth: web::Data<FirebaseAuth>,
    firestore_db: web::Data<FirebaseRestDb>,
    body: web::Json<CreateMedicalRecordForm>,
) -> impl Responder {
    if let Err(rejection) = require_records_permission(
        &req,
        &firebase_auth,
        &firestore_db,
        AppAction::EditMedicalRecords,
    )
    .await
    {
        return rejection;
    }

    match service.create_record(body.into_inner()).await {
        Ok(record) => HttpResponse::Created().json(record),
        Err(error) => record_error_response(error),
    }
}

pub async fn show_record_api(
    req: HttpRequest,
    service: web::Data<MedicalRecordService>,
    firebase_auth: web::Data<FirebaseAuth>,
    firestore_db: web::Data<FirebaseRestDb>,
    path: web::Path<String>,
) -> impl Responder {
    if let Err(rejection) = require_records_permission(
        &req,
        &firebase_auth,
        &firestore_db,
        AppAction::ViewMedicalRecords,
    )
    .await
    {
        return rejection;
    }

    match service.find_record(&path.into_inner()).await {
        Ok(record) => HttpResponse::Ok().json(record),
        Err(error) => record_error_response(error),
    }
}

pub async fn update_record_api(
    req: HttpRequest,
    service: web::Data<MedicalRecordService>,
    firebase_auth: web::Data<FirebaseAuth>,
    firestore_db: web::Data<FirebaseRestDb>,
    path: web::Path<String>,
    body: web::Json<UpdateMedicalRecordForm>,
) -> impl Responder {
    if let Err(rejection) = require_records_permission(
        &req,
        &firebase_auth,
        &firestore_db,
        AppAction::EditMedicalRecords,
    )
    .await
    {
        return rejection;
    }

    match service
        .update_record(&path.into_inner(), body.into_inner())
        .await
    {
        Ok(record) => HttpResponse::Ok().json(record),
        Err(error) => record_error_response(error),
    }
}

pub async fn delete_record_api(
    req: HttpRequest,
    service: web::Data<MedicalRecordService>,
    firebase_auth: web::Data<FirebaseAuth>,
    firestore_db: web::Data<FirebaseRestDb>,
    path: web::Path<String>,
) -> impl Responder {
    if let Err(rejection) = require_records_permission(
        &req,
        &firebase_auth,
        &firestore_db,
        AppAction::EditMedicalRecords,
    )
    .await
    {
        return rejection;
    }

    match service.delete_record(&path.into_inner()).await {
        Ok(_) => HttpResponse::NoContent().finish(),
        Err(error) => record_error_response(error),
    }
}

pub async fn patient_timeline_api(
    req: HttpRequest,
    service: web::Data<MedicalRecordService>,
    firebase_auth: web::Data<FirebaseAuth>,
    firestore_db: web::Data<FirebaseRestDb>,
    supabase_db: web::Data<SupabaseRestDb>,
    path: web::Path<String>,
) -> impl Responder {
    if let Err(rejection) = require_records_permission(
        &req,
        &firebase_auth,
        &firestore_db,
        AppAction::ViewMedicalRecords,
    )
    .await
    {
        return rejection;
    }

    match service.patient_timeline(&path.into_inner()).await {
        Ok(mut entries) => {
            for entry in entries.iter_mut() {
                if let Some(ref doctor_id) = entry.record.doctor_id {
                    if let Ok(name) = fetch_doctor_name(&supabase_db, doctor_id).await {
                        entry.doctor_name = Some(name);
                    }
                }
            }
            HttpResponse::Ok().json(entries)
        }
        Err(error) => record_error_response(error),
    }
}

// ── Query parameter structs ──────────────────────────────────────────────────

#[derive(Deserialize)]
pub struct RecordListQuery {
    pub patient_id: Option<String>,
}

#[derive(Deserialize)]
pub struct NewRecordQuery {
    pub patient_id: Option<String>,
    pub appointment_id: Option<String>,
}

// ── Helpers ──────────────────────────────────────────────────────────────────

async fn fetch_patient(
    supabase_db: &SupabaseRestDb,
    patient_id: &str,
) -> Result<PatientView, ()> {
    let body = supabase_db.get_patient(patient_id).await.map_err(|_| ())?;
    let rows: Vec<SupabasePatientRow> = serde_json::from_str(&body).map_err(|_| ())?;
    rows.into_iter().next().map(PatientView::from).ok_or(())
}

async fn fetch_doctor_name(supabase_db: &SupabaseRestDb, doctor_id: &str) -> Result<String, ()> {
    #[derive(serde::Deserialize)]
    struct StaffEmbed {
        full_name: String,
    }
    #[derive(serde::Deserialize)]
    struct DoctorRow {
        staff: StaffEmbed,
    }

    let encoded_id = urlencoding::encode(doctor_id);
    let url = format!(
        "{}/rest/v1/doctors?id=eq.{}&select=staff:staff_id(full_name)",
        supabase_db.url, encoded_id
    );
    let response = supabase_db
        .http_client
        .get(&url)
        .header("apikey", &supabase_db.key)
        .header("Authorization", format!("Bearer {}", supabase_db.key))
        .send()
        .await
        .map_err(|_| ())?;

    let rows: Vec<DoctorRow> = response.json().await.map_err(|_| ())?;
    rows.into_iter().next().map(|r| r.staff.full_name).ok_or(())
}

async fn require_records_permission(
    req: &HttpRequest,
    firebase_auth: &FirebaseAuth,
    firestore_db: &FirebaseRestDb,
    action: AppAction,
) -> Result<(String, String), HttpResponse> {
    require_auth_and_permission(req, firebase_auth, firestore_db, action).await
}

fn render_template(templates: &Tera, template_name: &str, context: &Context) -> HttpResponse {
    match templates.render(template_name, context) {
        Ok(body) => HttpResponse::Ok().content_type("text/html").body(body),
        Err(error) => HttpResponse::InternalServerError()
            .content_type("text/plain")
            .body(format!("Template error: {error}")),
    }
}

fn render_error(templates: &Tera, error: MedicalRecordError) -> HttpResponse {
    let mut context = Context::new();
    context.insert("message", &record_error_message(error));
    context.insert("back_url", "/medical-records");
    context.insert("back_label", "Back to Medical Records");
    render_template(templates, "error.html", &context)
}

fn record_error_response(error: MedicalRecordError) -> HttpResponse {
    let message = record_error_message(error);
    if message.contains("not found") {
        HttpResponse::NotFound().body(message)
    } else if message.contains("unavailable") {
        HttpResponse::InternalServerError().body(message)
    } else {
        HttpResponse::BadRequest().body(message)
    }
}

fn record_error_message(error: MedicalRecordError) -> String {
    match error {
        MedicalRecordError::InvalidInput(msg) => msg,
        MedicalRecordError::RecordNotFound => "Medical record was not found.".to_string(),
        MedicalRecordError::StorageUnavailable => {
            "Medical record storage is unavailable.".to_string()
        }
    }
}

fn redirect_to(location: &str) -> HttpResponse {
    HttpResponse::SeeOther()
        .insert_header((header::LOCATION, location))
        .finish()
}
