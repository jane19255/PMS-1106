use crate::appointments::handlers as appointment_handlers;
use crate::billing::handlers as billing_handlers;
use crate::doctors::handlers as doctor_handlers;
use crate::admindashboard::handlers as admindashboard_handlers; // To rename to admin_dashboard_handlers when safe
use crate::doctor_dashboard::handlers as doctor_dashboard_handlers;
use crate::patients::handlers as patient_handlers;
use crate::medical_records::handlers as medical_record_handlers;
use crate::prescriptions::handlers as prescription_handlers;
use crate::staff::handlers as staff_handlers;

use crate::handlers::auth; // To delete when safe

/// Registers each feature module's route group with the Actix application.
pub fn configure(config: &mut actix_web::web::ServiceConfig) {
    // Authentication routes
    config.configure(auth::routes);

    // Patient management routes
    config.configure(patient_handlers::routes);
    config.configure(doctor_handlers::routes);
    config.configure(doctor_dashboard_handlers::routes);
    config.configure(medical_record_handlers::routes);
    config.configure(prescription_handlers::routes);
    config.configure(staff_handlers::routes);

    config.configure(admindashboard_handlers::routes);
    config.configure(appointment_handlers::routes);

    config.configure(billing_handlers::routes);
}
