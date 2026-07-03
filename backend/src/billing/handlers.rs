use super::models::{ClinicalSummary, CreateInvoiceForm, PaymentStatus, RecordPaymentForm};
use super::pdf::render_medical_report_pdf;
use super::service::{BillingError, BillingService};
use crate::db::{FirebaseRestDb, SupabaseRestDb};
use crate::doctors::service::DoctorService;
use crate::handlers::auth::{require_auth_and_permission, AppAction};
use crate::models::{PatientView, SupabasePatientRow};
use actix_web::http::header;
use actix_web::{web, HttpRequest, HttpResponse, Responder};
use firebase_auth::FirebaseAuth;
use std::collections::HashMap;
use tera::{Context, Tera};

pub async fn list_invoices(
    req: HttpRequest,
    billing_service: web::Data<BillingService>,
    templates: web::Data<Tera>,
    firebase_auth: web::Data<FirebaseAuth>,
    firestore_db: web::Data<FirebaseRestDb>,
) -> impl Responder {
    if let Err(rejection) =
        require_auth_and_permission(&req, &firebase_auth, &firestore_db, AppAction::ViewBilling)
            .await
    {
        return rejection;
    }

    match billing_service.list_invoices().await {
        Ok(invoices) => {
            let mut context = Context::new();
            let pending_count = invoices
                .iter()
                .filter(|invoice| invoice.status == PaymentStatus::Pending)
                .count();
            let paid_count = invoices
                .iter()
                .filter(|invoice| invoice.status == PaymentStatus::Paid)
                .count();
            let cancelled_count = invoices
                .iter()
                .filter(|invoice| invoice.status == PaymentStatus::Cancelled)
                .count();
            let paid_revenue: f64 = invoices
                .iter()
                .filter(|invoice| invoice.status == PaymentStatus::Paid)
                .map(|invoice| invoice.total)
                .fold(0.0, |total, invoice_total| total + invoice_total);

            context.insert("invoices", &invoices);
            context.insert("pending_count", &pending_count);
            context.insert("paid_count", &paid_count);
            context.insert("cancelled_count", &cancelled_count);
            context.insert("paid_revenue", &paid_revenue);
            context.insert("medicine_options", &billing_service.medicine_catalog());
            render_template(&templates, "billing/index.html", &context)
        }
        Err(error) => render_error(&templates, error),
    }
}

pub async fn create_invoice(
    req: HttpRequest,
    billing_service: web::Data<BillingService>,
    templates: web::Data<Tera>,
    body: web::Bytes,
    firebase_auth: web::Data<FirebaseAuth>,
    firestore_db: web::Data<FirebaseRestDb>,
    supabase_db: web::Data<SupabaseRestDb>,
) -> impl Responder {
    if let Err(rejection) = require_auth_and_permission(
        &req,
        &firebase_auth,
        &firestore_db,
        AppAction::CreateInvoice,
    )
    .await
    {
        return rejection;
    }

    let form = match parse_create_invoice_form(&body) {
        Ok(form) => form,
        Err(error) => return render_error(&templates, error),
    };

    if let Err(error) = validate_billable_appointment(&supabase_db, &form).await {
        return render_error(&templates, error);
    }

    match billing_service.create_invoice(form).await {
        Ok(_) => redirect_to("/billing"),
        Err(error) => render_error(&templates, error),
    }
}

async fn validate_billable_appointment(
    db: &SupabaseRestDb,
    form: &CreateInvoiceForm,
) -> Result<(), BillingError> {
    // An invoice without an appointment is allowed for other clinic charges.
    let Some(appointment_id) = form
        .appointment_id
        .as_deref()
        .map(str::trim)
        .filter(|value| !value.is_empty())
    else {
        return Ok(());
    };

    let body = db
        .get_appointment(appointment_id)
        .await
        .map_err(|_| BillingError::StorageUnavailable)?;
    let rows: Vec<serde_json::Value> =
        serde_json::from_str(&body).map_err(|_| BillingError::StorageUnavailable)?;
    let appointment = rows.first().ok_or_else(|| {
        BillingError::InvalidInput("The selected appointment was not found.".to_string())
    })?;

    // Stop users from attaching another patient's appointment to this invoice.
    if appointment.get("patient_id").and_then(serde_json::Value::as_str)
        != Some(form.patient_id.trim())
    {
        return Err(BillingError::InvalidInput(
            "The selected appointment belongs to a different patient.".to_string(),
        ));
    }
    // Appointment charges should only be billed after the visit is finished.
    if appointment.get("status").and_then(serde_json::Value::as_str) != Some("Completed") {
        return Err(BillingError::InvalidInput(
            "The appointment must be completed before an invoice is created.".to_string(),
        ));
    }

    Ok(())
}

pub async fn show_invoice(
    req: HttpRequest,
    billing_service: web::Data<BillingService>,
    templates: web::Data<Tera>,
    path: web::Path<String>,
    firebase_auth: web::Data<FirebaseAuth>,
    firestore_db: web::Data<FirebaseRestDb>,
    supabase_db: web::Data<SupabaseRestDb>,
) -> impl Responder {
    if let Err(rejection) =
        require_auth_and_permission(&req, &firebase_auth, &firestore_db, AppAction::ViewBilling)
            .await
    {
        return rejection;
    }

    match billing_service.find_invoice(&path.into_inner()).await {
        Ok(invoice) => {
            let mut context = Context::new();
            context.insert("invoice", &invoice);
            context.insert(
                "patient",
                &find_patient(&supabase_db, &invoice.patient_id).await,
            );
            render_template(&templates, "billing/show.html", &context)
        }
        Err(error) => render_error(&templates, error),
    }
}

pub async fn record_payment(
    req: HttpRequest,
    billing_service: web::Data<BillingService>,
    templates: web::Data<Tera>,
    path: web::Path<String>,
    form: web::Form<RecordPaymentForm>,
    firebase_auth: web::Data<FirebaseAuth>,
    firestore_db: web::Data<FirebaseRestDb>,
) -> impl Responder {
    if let Err(rejection) = require_auth_and_permission(
        &req,
        &firebase_auth,
        &firestore_db,
        AppAction::RecordPayment,
    )
    .await
    {
        return rejection;
    }

    let invoice_id = path.into_inner();

    match billing_service
        .record_payment(&invoice_id, form.into_inner())
        .await
    {
        Ok(_) => redirect_to(&format!("/billing/invoices/{invoice_id}")),
        Err(error) => render_error(&templates, error),
    }
}

pub async fn cancel_invoice(
    req: HttpRequest,
    billing_service: web::Data<BillingService>,
    templates: web::Data<Tera>,
    path: web::Path<String>,
    firebase_auth: web::Data<FirebaseAuth>,
    firestore_db: web::Data<FirebaseRestDb>,
) -> impl Responder {
    if let Err(rejection) = require_auth_and_permission(
        &req,
        &firebase_auth,
        &firestore_db,
        AppAction::CancelInvoice,
    )
    .await
    {
        return rejection;
    }

    let invoice_id = path.into_inner();

    match billing_service.cancel_invoice(&invoice_id).await {
        Ok(_) => redirect_to(&format!("/billing/invoices/{invoice_id}")),
        Err(error) => render_error(&templates, error),
    }
}

#[allow(clippy::too_many_arguments)]
pub async fn show_medical_report(
    req: HttpRequest,
    billing_service: web::Data<BillingService>,
    doctor_service: web::Data<DoctorService>,
    templates: web::Data<Tera>,
    path: web::Path<String>,
    firebase_auth: web::Data<FirebaseAuth>,
    firestore_db: web::Data<FirebaseRestDb>,
    supabase_db: web::Data<SupabaseRestDb>,
) -> impl Responder {
    if let Err(rejection) = require_auth_and_permission(
        &req,
        &firebase_auth,
        &firestore_db,
        AppAction::GenerateBillingReport,
    )
    .await
    {
        return rejection;
    }

    match billing_service
        .generate_medical_report(&path.into_inner())
        .await
    {
        Ok(report) => {
            let mut context = Context::new();
            let patient = find_patient(&supabase_db, &report.invoice.patient_id).await;
            let clinical_record = find_clinical_summary(
                &supabase_db,
                &doctor_service,
                &report.invoice.patient_id,
                report.invoice.appointment_id.as_deref(),
            )
            .await;
            context.insert("report", &report);
            context.insert("patient", &patient);
            context.insert("clinical_record", &clinical_record);
            render_template(&templates, "billing/report.html", &context)
        }
        Err(error) => render_error(&templates, error),
    }
}

#[allow(clippy::too_many_arguments)]
pub async fn download_medical_report_pdf(
    req: HttpRequest,
    billing_service: web::Data<BillingService>,
    doctor_service: web::Data<DoctorService>,
    templates: web::Data<Tera>,
    path: web::Path<String>,
    firebase_auth: web::Data<FirebaseAuth>,
    firestore_db: web::Data<FirebaseRestDb>,
    supabase_db: web::Data<SupabaseRestDb>,
) -> impl Responder {
    if let Err(rejection) = require_auth_and_permission(
        &req,
        &firebase_auth,
        &firestore_db,
        AppAction::GenerateBillingReport,
    )
    .await
    {
        return rejection;
    }

    match billing_service
        .generate_medical_report(&path.into_inner())
        .await
    {
        Ok(report) => {
            let patient = find_patient(&supabase_db, &report.invoice.patient_id).await;
            let clinical_record = find_clinical_summary(
                &supabase_db,
                &doctor_service,
                &report.invoice.patient_id,
                report.invoice.appointment_id.as_deref(),
            )
            .await;

            match render_medical_report_pdf(&report, patient.as_ref(), clinical_record.as_ref()) {
                Ok(pdf) => HttpResponse::Ok()
                    .insert_header((header::CONTENT_TYPE, "application/pdf"))
                    .insert_header((
                        header::CONTENT_DISPOSITION,
                        format!(
                            "attachment; filename=\"{}-medical-report.pdf\"",
                            report.invoice.id
                        ),
                    ))
                    .body(pdf),
                Err(_) => HttpResponse::InternalServerError()
                    .content_type("text/plain")
                    .body("Unable to generate PDF report."),
            }
        }
        Err(error) => render_error(&templates, error),
    }
}

async fn find_patient(database: &SupabaseRestDb, patient_id: &str) -> Option<PatientView> {
    let body = database.get_patient(patient_id).await.ok()?;
    serde_json::from_str::<Vec<SupabasePatientRow>>(&body)
        .ok()?
        .into_iter()
        .next()
        .map(PatientView::from)
}

async fn find_clinical_summary(
    database: &SupabaseRestDb,
    doctor_service: &DoctorService,
    patient_id: &str,
    appointment_id: Option<&str>,
) -> Option<ClinicalSummary> {
    let body = database
        .get_latest_medical_record(patient_id, appointment_id)
        .await
        .ok()?;
    let mut summary = serde_json::from_str::<Vec<ClinicalSummary>>(&body)
        .ok()?
        .into_iter()
        .next()?;

    // The record's doctor_name column is rarely filled in directly — the
    // record form only ever captures doctor_id — so resolve the name from
    // the doctor profile whenever it's missing.
    if summary.doctor_name.is_none() {
        if let Some(doctor_id) = summary.doctor_id.as_deref() {
            if let Ok(doctor) = doctor_service.find_doctor(doctor_id).await {
                summary.doctor_name = Some(doctor.name);
            }
        }
    }

    Some(summary)
}

fn render_template(templates: &Tera, template_name: &str, context: &Context) -> HttpResponse {
    match templates.render(template_name, context) {
        Ok(body) => HttpResponse::Ok().content_type("text/html").body(body),
        Err(error) => HttpResponse::InternalServerError()
            .content_type("text/plain")
            .body(format!("Template error: {error}")),
    }
}

fn render_error(templates: &Tera, error: BillingError) -> HttpResponse {
    let mut context = Context::new();
    context.insert("message", &billing_error_message(error));
    context.insert("back_url", "/billing");
    context.insert("back_label", "Back to Billing");
    render_template(templates, "error.html", &context)
}

fn billing_error_message(error: BillingError) -> String {
    match error {
        BillingError::InvalidInput(message) => message,
        BillingError::InvoiceNotFound => "Invoice was not found.".to_string(),
        BillingError::StorageUnavailable => "Billing storage is unavailable.".to_string(),
    }
}

fn parse_create_invoice_form(body: &[u8]) -> Result<CreateInvoiceForm, BillingError> {
    let body = std::str::from_utf8(body).map_err(|_| {
        BillingError::InvalidInput("Invoice form data must be valid UTF-8.".to_string())
    })?;
    let mut fields: HashMap<String, Vec<String>> = HashMap::new();

    for pair in body.split('&').filter(|pair| !pair.is_empty()) {
        let mut parts = pair.splitn(2, '=');
        let key = decode_form_component(parts.next().unwrap_or(""))?;
        let value = decode_form_component(parts.next().unwrap_or(""))?;
        fields.entry(key).or_default().push(value);
    }

    Ok(CreateInvoiceForm {
        patient_id: take_required_field(&mut fields, "patient_id"),
        appointment_id: take_optional_field(&mut fields, "appointment_id"),
        treatment_name: take_required_field(&mut fields, "treatment_name"),
        treatment_cost: take_required_field(&mut fields, "treatment_cost"),
        prescription_names: take_repeated_field(&mut fields, "prescription_names"),
        custom_prescription_names: take_repeated_field(&mut fields, "custom_prescription_names"),
        custom_prescription_costs: take_repeated_field(&mut fields, "custom_prescription_costs"),
    })
}

fn decode_form_component(value: &str) -> Result<String, BillingError> {
    urlencoding::decode(&value.replace('+', " "))
        .map(|decoded| decoded.into_owned())
        .map_err(|_| BillingError::InvalidInput("Invoice form data is not valid.".to_string()))
}

fn take_required_field(fields: &mut HashMap<String, Vec<String>>, name: &str) -> String {
    take_repeated_field(fields, name)
        .into_iter()
        .next()
        .unwrap_or_default()
}

fn take_optional_field(fields: &mut HashMap<String, Vec<String>>, name: &str) -> Option<String> {
    let value = take_required_field(fields, name);
    if value.trim().is_empty() {
        None
    } else {
        Some(value)
    }
}

fn take_repeated_field(fields: &mut HashMap<String, Vec<String>>, name: &str) -> Vec<String> {
    fields.remove(name).unwrap_or_default()
}

fn redirect_to(location: &str) -> HttpResponse {
    HttpResponse::SeeOther()
        .insert_header((header::LOCATION, location))
        .finish()
}
