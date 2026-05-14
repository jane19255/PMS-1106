use actix_web::{web, App, HttpServer};
use dotenv::dotenv;
use tera::Tera;
use firebase_auth::FirebaseAuth; // Added this

mod db;
mod handlers;
mod models;

#[actix_web::main]
async fn main() -> Result<(), std::io::Error> {
    dotenv().ok();

    // Note: Since your HTML files are in the 'pages' folder, 
    // "pages/**/*.html" ensures Tera finds them all.
    let tera = Tera::new("pages/**/*.html")
        .expect("Failed to parse pages");

    let firebase_project_id = std::env::var("FIREBASE_PROJECT_ID")
        .expect("FIREBASE_PROJECT_ID must be set in .env");

    // 1. Initialize the new Firebase Auth client
    let firebase_auth = FirebaseAuth::new(&firebase_project_id).await;

    // 2. Initialize your new Firebase REST Database client
    let firestore_db = db::FirebaseRestDb::new(firebase_project_id.clone());
    let supabase_db = db::SupabaseRestDb::from_env();

    println!("Server running at http://127.0.0.1:8080");

  HttpServer::new(move || {
        App::new()
            .app_data(web::Data::new(tera.clone()))
            .app_data(web::Data::new(firebase_auth.clone()))
            .app_data(web::Data::new(firestore_db.clone()))
            .app_data(web::Data::new(supabase_db.clone()))
            .app_data(web::Data::new(firebase_project_id.clone()))
            .app_data(web::FormConfig::default().limit(32_768))
            .service(actix_files::Files::new("/assets", "assets").prefer_utf8(true))
            .route("/", web::get().to(|| async {
                actix_web::HttpResponse::Found()
                    .append_header(("Location", "/login"))
                    .finish()
            }))
            .configure(handlers::auth::routes)
            .configure(handlers::patients::routes)
            
            // 🚨 THE ULTIMATE TRIPWIRE: Catch any 404 and print it to the terminal!
            .default_service(web::route().to(|req: actix_web::HttpRequest| async move {
                println!(">>> 🚨 ACTIX 404 REJECTION: The browser asked for '{}', but no route matched!", req.path());
                actix_web::HttpResponse::NotFound().body(format!("404 Not Found: {}", req.path()))
            }))
    })
    .bind("127.0.0.1:8080")?
    .run()
    .await
}