//! Application entry point for the patient management backend.
//!
//! This file wires shared services into Actix, registers all route modules, and starts
//! the background appointment reconciliation task before serving HTTP requests.
mod billing;
mod db;
mod doctors;
mod firebase_admin;
mod admindashboard;
mod doctor_dashboard;
mod auth;
mod patients;
mod medical_records;
mod prescriptions;
mod routes;
mod time;
mod staff;
mod appointments;

use actix_files::Files;
use actix_web::{web, App, HttpResponse, HttpServer};
use billing::repository::{
    InMemoryInvoiceRepository, InvoiceRepository, SupabaseInvoiceRepository,
};
use billing::service::BillingService;
use doctors::repository::{DoctorRepository, InMemoryDoctorRepository, SupabaseDoctorRepository};
use doctors::service::DoctorService;
use medical_records::repository::{
    InMemoryMedicalRecordRepository, MedicalRecordRepository, SupabaseMedicalRecordRepository,
};
use medical_records::service::MedicalRecordService;
use doctor_dashboard::repository::{
    DoctorDashboardRepository, InMemoryDoctorDashboardRepository, SupabaseDoctorDashboardRepository,
};
use doctor_dashboard::service::DoctorDashboardService;
use dotenv::dotenv;
use firebase_auth::FirebaseAuth;
use prescriptions::repository::{InMemoryPrescriptionRepository, PrescriptionRepository, SupabasePrescriptionRepository};
use prescriptions::service::PrescriptionService;
use staff::repository::{InMemoryStaffRepository, StaffRepository, SupabaseStaffRepository};
use staff::service::StaffService;
use std::sync::Arc;
use tera::Tera;

// Used for periodic background tasks, such as appointment reconciliation
use std::time::Duration;
use futures::FutureExt; // For .catch_unwind()
use tracing::{error, info}; // For structured logging

/// Starts an indefinite, async background task that checks and update the status of overdue 
/// appointments in the database every 5 minutes.
///
/// This function uses `actix_web::rt::spawn` to avoid blocking the main HTTP server thread
///
/// ### Error Handling
/// * **Database Failure:** If Supabase REST API is unreachable, log the errors and restart the 5 minute loop
/// * **Panics:** If unexpected panic occurs inside the async block, it is logged and safely recovered to
/// resume the next 5 minute loop
fn start_appointment_reconciliation(db: db::SupabaseRestDb) {
    
    // Spawn an async thread managed by Actix Web's runtime to prevent blocking the main thread
    actix_web::rt::spawn(async move {
        // Log a status message indicating that the background worker has started
        info!("Background appointment reconciliation worker started.");

        // Keep the background task running indefinitely, looping every 5 minutes
        loop {
            // Catch panics during async operations with assert
            let outcome = FutureExt::catch_unwind(std::panic::AssertUnwindSafe(async {
                // Call database method to set appointment status to "no-show".
                db.reconcile_overdue_appointments(db::singapore_today()).await
            })).await;

            // Handle success, failure, or panic outcomes
            match outcome {
                // No panic + No database errors
                Ok(Ok(_)) => {
                    info!("Appointment reconciliation completed successfully.");
                }
                // No panic + expected database errors (like database is offline)
                Ok(Err(error)) => {
                    error!("Appointment reconciliation failed: {error}");
                }
                // Panic in database call (like out-of-memory error) but loop will continue to run
                Err(panic) => {
                    error!("Appointment reconciliation panicked: {:?}. Retrying in 5 minutes.", panic);
                }
            }

            actix_web::rt::time::sleep(Duration::from_secs(300)).await;
        }
    });
}

#[actix_web::main]
async fn main() -> std::io::Result<()> {
    dotenv().ok(); // Load .env in cwd if exists

    let templates = Tera::new("templates/**/*.html")
        .expect("templates directory should contain valid Tera templates");

    // Load values from .env, required for Firebase and Supabase configuration
    let firebase_project_id =
        std::env::var("FIREBASE_PROJECT_ID").expect("FIREBASE_PROJECT_ID must be set in .env");

    let firebase_auth = FirebaseAuth::new(&firebase_project_id).await;
    let firebase_admin = web::Data::new(
        firebase_admin::FirebaseAdmin::from_env()
            .expect("Firebase Admin configuration is invalid"),
    );
    let firestore_db = db::FirebaseRestDb::new(firebase_project_id.clone());
    let supabase_db = db::SupabaseRestDb::from_env();

    // Repositories are decoupled via traits and wrapped in Arc<dyn Trait>
    // This allows runtime polymorphism and to be toggled between in-memory and Supabase PostgreSQL implementations
    // At the same time, Arc is used to ensure that the repository instances can be shared across multiple threads safely
    let invoice_repository: Arc<dyn InvoiceRepository> =
        match std::env::var("BILLING_STORAGE").as_deref() {
            Ok("supabase") => {
                println!("Billing storage: Supabase PostgreSQL");
                Arc::new(SupabaseInvoiceRepository::new(
                    supabase_db.url.clone(),
                    supabase_db.key.clone(),
                ))
            }
            _ => {
                println!("Billing storage: in-memory (set BILLING_STORAGE=supabase to persist)");
                Arc::new(InMemoryInvoiceRepository::default())
            }
        };

    // Wrap services in web::Data
    // Actix Web clones app state per worker thread
    // web::Data uses Arc again to ensure threads share the same service instances without duplicating resources or memory allocations
    let billing_service = web::Data::new(BillingService::new(invoice_repository));
    let doctor_repository: Arc<dyn DoctorRepository> =
        match std::env::var("DOCTOR_STORAGE").as_deref() {
            Ok("supabase") => {
                println!("Doctor storage: Supabase PostgreSQL");
                Arc::new(SupabaseDoctorRepository::new(
                    supabase_db.url.clone(),
                    supabase_db.key.clone(),
                ))
            }
            _ => {
                println!("Doctor storage: in-memory (set DOCTOR_STORAGE=supabase to persist)");
                Arc::new(InMemoryDoctorRepository::default())
            }
        };
    let doctor_service = web::Data::new(DoctorService::new(doctor_repository));
    let doctor_dashboard_repository: Arc<dyn DoctorDashboardRepository> =
        match std::env::var("DOCTOR_DASHBOARD_STORAGE").as_deref() {
            Ok("supabase") => {
                println!("Doctor dashboard storage: Supabase PostgreSQL");
                Arc::new(SupabaseDoctorDashboardRepository::new(
                    supabase_db.url.clone(),
                    supabase_db.key.clone(),
                ))
            }
            _ => {
                println!("Doctor dashboard storage: in-memory (set DOCTOR_DASHBOARD_STORAGE=supabase to persist)");
                Arc::new(InMemoryDoctorDashboardRepository)
            }
        };
    let doctor_dashboard_service = web::Data::new(DoctorDashboardService::new(doctor_dashboard_repository));
    let prescription_repository: Arc<dyn PrescriptionRepository> =
        match std::env::var("PRESCRIPTION_STORAGE").as_deref() {
            Ok("supabase") => {
                println!("Prescription storage: Supabase PostgreSQL");
                Arc::new(SupabasePrescriptionRepository::new(
                    supabase_db.url.clone(),
                    supabase_db.key.clone(),
                ))
            }
            _ => {
                println!("Prescription storage: in-memory (set PRESCRIPTION_STORAGE=supabase to persist)");
                Arc::new(InMemoryPrescriptionRepository::default())
            }
        };
    let prescription_service = web::Data::new(PrescriptionService::new(prescription_repository));
    let medical_record_repository: Arc<dyn MedicalRecordRepository> =
        match std::env::var("MEDICAL_RECORDS_STORAGE").as_deref() {
            Ok("supabase") => {
                println!("Medical records storage: Supabase PostgreSQL");
                Arc::new(SupabaseMedicalRecordRepository::new(
                    supabase_db.url.clone(),
                    supabase_db.key.clone(),
                ))
            }
            _ => {
                println!("Medical records storage: in-memory (set MEDICAL_RECORDS_STORAGE=supabase to persist)");
                Arc::new(InMemoryMedicalRecordRepository::default())
            }
        };
    let medical_record_service = web::Data::new(MedicalRecordService::new(medical_record_repository));
    let staff_repository: Arc<dyn StaffRepository> =
        match std::env::var("STAFF_STORAGE").as_deref() {
            Ok("supabase") => {
                println!("Staff storage: Supabase PostgreSQL");
                Arc::new(SupabaseStaffRepository::new(
                    supabase_db.url.clone(),
                    supabase_db.key.clone(),
                ))
            }
            _ => {
                println!("Staff storage: in-memory (set STAFF_STORAGE=supabase to persist)");
                Arc::new(InMemoryStaffRepository::default())
            }
        };
    let staff_service = web::Data::new(StaffService::new(staff_repository));
    let template_data = web::Data::new(templates);

    start_appointment_reconciliation(supabase_db.clone());

    println!("Server running at http://127.0.0.1:8080");

    // Use HttpServer::new to spawn multiple worker threads, each running an instance of the HTTP server
    // We use `move` and `.clone()` to ensure that each worker thread has its own copy of the shared server state
    HttpServer::new(move || {
        App::new()
            .app_data(billing_service.clone())
            .app_data(doctor_service.clone())
            .app_data(doctor_dashboard_service.clone())
            .app_data(medical_record_service.clone())
            .app_data(prescription_service.clone())
            .app_data(staff_service.clone())
            .app_data(template_data.clone())
            .app_data(web::Data::new(firebase_auth.clone()))
            .app_data(firebase_admin.clone())
            .app_data(web::Data::new(firestore_db.clone()))
            .app_data(web::Data::new(supabase_db.clone()))
            .app_data(web::FormConfig::default().limit(32_768)) // Set max form size to 32KB
            .service(Files::new("/assets", "../frontend/assets"))
            .route(
                "/",
                web::get().to(|| async {
                    HttpResponse::Found()
                        .append_header(("Location", "/login"))
                        .finish()
                }),
            )
            .configure(routes::configure)
            .default_service(web::route().to(|req: actix_web::HttpRequest| async move {
                println!("No route matched: {}", req.path());
                HttpResponse::NotFound().body(format!("404 Not Found: {}", req.path()))
            }))
    })
    .bind(("127.0.0.1", 8080))?
    .run()
    .await
}

#[cfg(test)]
mod template_tests {
    use tera::Tera;

    #[test]
    fn all_tera_templates_parse() {
        Tera::new("templates/**/*.html").expect("all Tera templates should parse");
    }
    #[test]
    fn doctor_detail_template_renders_without_schedule_form_context() {
        use crate::doctors::models::{Doctor, DoctorStatus};
        use tera::Context;

        let templates = Tera::new("templates/**/*.html").expect("all Tera templates should parse");
        let doctor = Doctor {
            id: "DOC-TEST".to_string(),
            staff_id: "STAFF-TEST".to_string(),
            license_number: "M-TEST".to_string(),
            name: "Dr. Test".to_string(),
            specialization: "General Medicine".to_string(),
            contact_number: "80000000".to_string(),
            email: "test@example.com".to_string(),
            status: DoctorStatus::Available,
        };
        let mut context = Context::new();
        context.insert("doctor", &doctor);
        context.insert("doctor_status", "Available");
        context.insert("schedules", &Vec::<crate::doctors::models::DoctorSchedule>::new());
        context.insert("schedule_day_of_week", "");
        context.insert("schedule_start_time", "");
        context.insert("schedule_end_time", "");
        context.insert("invalid_schedule_start_time", &false);
        context.insert("invalid_schedule_end_time", &false);

        let html = templates
            .render("doctors/show.html", &context)
            .expect("doctor detail template should render");

        assert!(html.contains("Dr. Test"));
    }
}

