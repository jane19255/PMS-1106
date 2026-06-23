mod billing;
mod db;
mod handlers;
mod models;
mod prescriptions;
mod routes;
mod doctors;

use actix_files::Files;
use actix_web::{web, App, HttpResponse, HttpServer};
use billing::repository::{
    InMemoryInvoiceRepository, InvoiceRepository, SupabaseInvoiceRepository,
};
use billing::service::BillingService;
use doctors::repository::{DoctorRepository, InMemoryDoctorRepository, SupabaseDoctorRepository};
use doctors::service::DoctorService;
use dotenv::dotenv;
use firebase_auth::FirebaseAuth;
use std::sync::Arc;
use tera::Tera;

#[actix_web::main]
async fn main() -> std::io::Result<()> {
    dotenv().ok();

    let templates = Tera::new("templates/**/*.html")
        .expect("templates directory should contain valid Tera templates");

    let firebase_project_id =
        std::env::var("FIREBASE_PROJECT_ID").expect("FIREBASE_PROJECT_ID must be set in .env");

    let firebase_auth = FirebaseAuth::new(&firebase_project_id).await;
    let firestore_db = db::FirebaseRestDb::new(firebase_project_id.clone());
    let supabase_db = db::SupabaseRestDb::from_env();

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
    let template_data = web::Data::new(templates);

    println!("Server running at http://127.0.0.1:8080");

    HttpServer::new(move || {
        App::new()
            .app_data(billing_service.clone())
            .app_data(doctor_service.clone())
            .app_data(template_data.clone())
            .app_data(web::Data::new(firebase_auth.clone()))
            .app_data(web::Data::new(firestore_db.clone()))
            .app_data(web::Data::new(supabase_db.clone()))
            .app_data(web::Data::new(firebase_project_id.clone()))
            .app_data(web::FormConfig::default().limit(32_768))
            .service(Files::new("/static", "static"))
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
                println!(
                    ">>> ÃƒÂ°Ã…Â¸Ã…Â¡Ã‚Â¨ ACTIX 404 REJECTION: The browser asked for '{}', but no route matched!",
                    req.path()
                );
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


