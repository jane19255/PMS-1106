use crate::billing::handlers;
use actix_web::web;

pub fn configure(config: &mut web::ServiceConfig) {
    config.service(
        web::scope("/billing")
            .route("", web::get().to(handlers::list_invoices))
            .route("/invoices", web::post().to(handlers::create_invoice))
            .route(
                "/invoices/{invoice_id}",
                web::get().to(handlers::show_invoice),
            )
            .route(
                "/invoices/{invoice_id}/pay",
                web::post().to(handlers::mark_invoice_paid),
            )
            .route(
                "/invoices/{invoice_id}/report",
                web::get().to(handlers::show_medical_report),
            ),
    );
}
