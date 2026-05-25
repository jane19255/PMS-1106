use super::models::CreateInvoiceForm;
use super::service::{BillingError, BillingService};
use actix_web::http::header;
use actix_web::{web, HttpResponse, Responder};
use tera::{Context, Tera};

pub async fn list_invoices(
    billing_service: web::Data<BillingService>,
    templates: web::Data<Tera>,
) -> impl Responder {
    match billing_service.list_invoices() {
        Ok(invoices) => {
            let mut context = Context::new();
            context.insert("invoices", &invoices);
            render_template(&templates, "billing/index.html", &context)
        }
        Err(error) => render_error(&templates, error),
    }
}

pub async fn create_invoice(
    billing_service: web::Data<BillingService>,
    templates: web::Data<Tera>,
    form: web::Form<CreateInvoiceForm>,
) -> impl Responder {
    match billing_service.create_invoice(form.into_inner()) {
        Ok(_) => redirect_to("/billing"),
        Err(error) => render_error(&templates, error),
    }
}

pub async fn show_invoice(
    billing_service: web::Data<BillingService>,
    templates: web::Data<Tera>,
    path: web::Path<String>,
) -> impl Responder {
    match billing_service.find_invoice(&path.into_inner()) {
        Ok(invoice) => {
            let mut context = Context::new();
            context.insert("invoice", &invoice);
            render_template(&templates, "billing/show.html", &context)
        }
        Err(error) => render_error(&templates, error),
    }
}

pub async fn mark_invoice_paid(
    billing_service: web::Data<BillingService>,
    templates: web::Data<Tera>,
    path: web::Path<String>,
) -> impl Responder {
    let invoice_id = path.into_inner();

    match billing_service.mark_invoice_paid(&invoice_id) {
        Ok(_) => redirect_to(&format!("/billing/invoices/{invoice_id}")),
        Err(error) => render_error(&templates, error),
    }
}

pub async fn show_medical_report(
    billing_service: web::Data<BillingService>,
    templates: web::Data<Tera>,
    path: web::Path<String>,
) -> impl Responder {
    match billing_service.generate_medical_report(&path.into_inner()) {
        Ok(report) => {
            let mut context = Context::new();
            context.insert("report", &report);
            render_template(&templates, "billing/report.html", &context)
        }
        Err(error) => render_error(&templates, error),
    }
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
