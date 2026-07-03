use crate::appointments::handlers as appointment_handlers;
use crate::billing::handlers as billing_handlers;
use crate::doctors::handlers as doctor_handlers;
use crate::admindashboard::handlers as admindashboard_handlers;
use crate::doctor_dashboard::handlers as doctor_dashboard_handlers;
use crate::handlers::{auth, patients};
use crate::medical_records::handlers as medical_record_handlers;
use crate::prescriptions::handlers as prescription_handlers;
use crate::staff::handlers as staff_handlers;

/// Registers each feature module's route group with the Actix application.
pub fn configure(config: &mut actix_web::web::ServiceConfig) {
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

    config.configure(billing_handlers::routes);
}
