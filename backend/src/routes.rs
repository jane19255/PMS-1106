use crate::billing::handlers as billing_handlers;
use crate::handlers::{auth, patients};
use actix_web::web;

pub fn configure(config: &mut web::ServiceConfig) {
    config.configure(auth::routes);
    config.configure(patients::routes);

    config.service(
        web::scope("/billing")
            .route("", web::get().to(billing_handlers::list_invoices))
            .route(
                "/invoices",
                web::post().to(billing_handlers::create_invoice),
            )
            .route(
                "/invoices/{invoice_id}",
                web::get().to(billing_handlers::show_invoice),
            )
            .route(
                "/invoices/{invoice_id}/payments",
                web::post().to(billing_handlers::record_payment),
            )
            .route(
                "/invoices/{invoice_id}/cancel",
                web::post().to(billing_handlers::cancel_invoice),
            )
            .route(
                "/invoices/{invoice_id}/report",
                web::get().to(billing_handlers::show_medical_report),
            )
            .route(
                "/invoices/{invoice_id}/report.pdf",
                web::get().to(billing_handlers::download_medical_report_pdf),
            ),
    );
}
