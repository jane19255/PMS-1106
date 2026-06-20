use super::models::{CreateInvoiceForm, PaymentStatus, RecordPaymentForm};
use super::service::{BillingError, BillingService};
use crate::db::{FirebaseRestDb, SupabaseRestDb};
use crate::handlers::auth::{require_auth_and_permission, AppAction};
use crate::models::{PatientView, SupabasePatientRow};
use actix_web::http::header;
use actix_web::{web, HttpRequest, HttpResponse, Responder};
use firebase_auth::FirebaseAuth;
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
            render_template(&templates, "billing/index.html", &context)
        }
        Err(error) => render_error(&templates, error),
    }
}

pub async fn create_invoice(
    req: HttpRequest,
    billing_service: web::Data<BillingService>,
    templates: web::Data<Tera>,
    form: web::Form<CreateInvoiceForm>,
    firebase_auth: web::Data<FirebaseAuth>,
    firestore_db: web::Data<FirebaseRestDb>,
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

    match billing_service.create_invoice(form.into_inner()).await {
        Ok(_) => redirect_to("/billing"),
        Err(error) => render_error(&templates, error),
    }
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
            context.insert("patient", &find_patient(&supabase_db, &invoice.patient_id).await);
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

pub async fn show_medical_report(
    req: HttpRequest,
    billing_service: web::Data<BillingService>,
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
            context.insert("report", &report);
            context.insert(
                "patient",
                &find_patient(&supabase_db, &report.invoice.patient_id).await,
            );
            render_template(&templates, "billing/report.html", &context)
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
    render_template(templates, "error.html", &context)
}

fn billing_error_message(error: BillingError) -> String {
    match error {
        BillingError::InvalidInput(message) => message,
        BillingError::InvoiceNotFound => "Invoice was not found.".to_string(),
        BillingError::StorageUnavailable => "Billing storage is unavailable.".to_string(),
    }
}

fn redirect_to(location: &str) -> HttpResponse {
    HttpResponse::SeeOther()
        .insert_header((header::LOCATION, location))
        .finish()
}
