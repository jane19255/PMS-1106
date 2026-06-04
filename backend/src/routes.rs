use crate::billing::handlers as billing_handlers;
use crate::handlers::{auth, patients};
use actix_web::web;

pub fn configure(config: &mut web::ServiceConfig) {
    config.configure(auth::routes);
    config.configure(patients::routes);

    config.service(
        web::scope("/billing")
            .route("", web::get().to(billing_handlers::list_invoices))
            .route("/invoices", web::post().to(billing_handlers::create_invoice))
            .route(
                "/invoices/{invoice_id}",
                web::get().to(billing_handlers::show_invoice),
            )
            .route(
                "/invoices/{invoice_id}/pay",
                web::post().to(billing_handlers::mark_invoice_paid),
            )
            .route(
                "/invoices/{invoice_id}/report",
                web::get().to(billing_handlers::show_medical_report),
            ),
    );
}
