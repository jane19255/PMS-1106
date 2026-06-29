use crate::appointments::handlers as appointment_handlers;
use crate::billing::handlers as billing_handlers;
use crate::doctors::handlers as doctor_handlers;
use crate::doctor_dashboard::handlers as doctor_dashboard_handlers;
use crate::handlers::{auth, patients};
use crate::medical_records::handlers as medical_record_handlers;
use crate::prescriptions::handlers as prescription_handlers;
use crate::staff::handlers as staff_handlers;
use crate::queue::handlers as queue_handlers;
use actix_web::web;

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

    // Billing routes
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

    // Appointment scheduling routes
    config.service(
        web::scope("/appointments")
            .route("", web::get().to(appointment_handlers::list_appointments)) // List all appointments
            .route("", web::post().to(appointment_handlers::create_appointment)) // Create new appointment
            .route("/{appointment_id}", web::get().to(appointment_handlers::get_appointment)) // Get appointment by ID
            .route("/{appointment_id}", web::put().to(appointment_handlers::update_appointment)) // Update appointment by ID
            .route("/{appointment_id}", web::delete().to(appointment_handlers::delete_appointment)), // Delete appointment by ID
    );

    // Queue management routes
    config.service(
        web::scope("/queue")
            .route("/doctor/{doctor_id}", web::get().to(queue_handlers::list_queue_for_doctor)) // List queue for doctor
            .route("", web::post().to(queue_handlers::create_queue_entry)) // Create queue entry
            .route("/{queue_id}/status", web::put().to(queue_handlers::update_queue_status)), // Update queue status
    );
    // Doctor availability is exposed through the existing doctor schedule API:
    // GET/POST /api/doctors/{doctor_id}/schedules.
}
