use crate::appointments::handlers as appointment_handlers;
use crate::billing::handlers as billing_handlers;
use crate::doctors::handlers as doctor_handlers;
use crate::admindashboard::handlers as admindashboard_handlers;
use crate::doctor_dashboard::handlers as doctor_dashboard_handlers;
use crate::handlers::{auth, patients};
use crate::medical_records::handlers as medical_record_handlers;
use crate::prescriptions::handlers as prescription_handlers;
use crate::staff::handlers as staff_handlers;
use actix_web::web;

/// Registers the application's top-level route groups.
///
/// Most modules expose their own `routes` function because they own both page
/// routes and JSON API routes. Billing is registered here as a scoped group so
/// every billing endpoint shares the `/billing` prefix.
pub fn configure(config: &mut web::ServiceConfig) {
    // Authentication routes
    config.configure(auth::routes);

    // Patient management routes
    config.configure(patients::routes);
    config.configure(doctor_handlers::routes);
    config.configure(doctor_dashboard_handlers::routes);
    config.configure(medical_record_handlers::routes);
    config.configure(prescription_handlers::routes);
    config.configure(staff_handlers::routes);

    config.configure(admindashboard_handlers::routes);
    config.configure(appointment_handlers::routes);

    // Billing routes
    config.service(
        web::scope("/billing")
            .route("", web::get().to(billing_handlers::list_invoices))
            .route(
                "/billable-appointments",
                web::get().to(billing_handlers::list_billable_appointments),
            )
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

    // Doctor availability is exposed through the existing doctor schedule API:
    // GET/POST /api/doctors/{doctor_id}/schedules.
}
