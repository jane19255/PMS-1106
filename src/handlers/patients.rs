use actix_web::{web, HttpRequest, HttpResponse, Responder};
use serde_json::json;
use tera::{Context, Tera};
use firebase_auth::FirebaseAuth;

use crate::models::Patient;
use crate::db::FirebaseRestDb;
use crate::handlers::auth::{require_auth, require_permission, AppAction};

pub fn routes(cfg: &mut web::ServiceConfig) {
    // FLAT ROUTING: Absolutely zero chance of prefix collisions
    cfg.route("/patients", web::get().to(patients_page));
    cfg.route("/api/patients/new", web::post().to(create_patient));
}

// ── GET /patients ─────────────────────────────────────────────────────────────

pub async fn patients_page(
    req: HttpRequest,
    tera: web::Data<Tera>,
    firebase_auth: web::Data<FirebaseAuth>,
) -> impl Responder {
    println!(">>> INNER TRIPWIRE: Actix successfully routed to the patients page!");

    let _uid = match require_auth(&req, &firebase_auth).await {
        Ok(uid) => uid,
        Err(redirect) => return redirect,
    };

    let mut ctx = Context::new();
    ctx.insert("firebase_api_key", &std::env::var("FIREBASE_API_KEY").unwrap_or_default());
    ctx.insert("firebase_project_id", &std::env::var("FIREBASE_PROJECT_ID").unwrap_or_default());

    match tera.render("Patients.html", &ctx) {
        Ok(html) => HttpResponse::Ok().content_type("text/html; charset=utf-8").body(html),
        Err(e) => {
            eprintln!("Template error: {e}");
            HttpResponse::InternalServerError().body("Template rendering failed")
        }
    }
}

// ── POST /api/patients/new ────────────────────────────────────────────────────

pub async fn create_patient(
    req: HttpRequest, // 👈 ADDED: Actix needs this to read the user's role cookie!
    patient_data: web::Json<Patient>,
    firestore_db: web::Data<FirebaseRestDb>,
) -> impl Responder {
    
    // 🚨 THE SECURITY GUARD
    // This instantly kicks out anyone who isn't an Admin or Receptionist
    if let Err(rejection) = require_permission(&req, AppAction::CreatePatient) {
        return rejection; 
    }

    let doc_id = &patient_data.nric;
    let mut fields = serde_json::Map::new();
    
    let mut insert_string = |key: &str, val: &str| {
        fields.insert(key.to_string(), json!({ "stringValue": val }));
    };

    insert_string("firstName", &patient_data.first_name);
    insert_string("lastName", &patient_data.last_name);
    insert_string("dob", &patient_data.dob);
    insert_string("gender", &patient_data.gender);
    insert_string("nric", &patient_data.nric);
    insert_string("nationality", &patient_data.nationality);
    insert_string("phone", &patient_data.phone);
    insert_string("email", &patient_data.email);
    
    if let Some(ref val) = patient_data.emergency_name { insert_string("emergencyName", val); }
    if let Some(ref val) = patient_data.emergency_phone { insert_string("emergencyPhone", val); }
    if let Some(ref val) = patient_data.address { insert_string("address", val); }
    if let Some(ref val) = patient_data.allergies { insert_string("allergies", val); }
    if let Some(ref val) = patient_data.medications { insert_string("medications", val); }
    if let Some(ref val) = patient_data.conditions { insert_string("conditions", val); }

    insert_string("status", "Active");
    let payload = json!({ "fields": fields });

    match firestore_db.create_document("patients", doc_id, &payload).await {
        Ok(_) => {
            println!("Successfully registered patient: {}", doc_id);
            HttpResponse::Ok().json(json!({ "status": "success" }))
        },
        Err(e) => {
            eprintln!("Failed to create patient: {}", e);
            HttpResponse::InternalServerError().body("Failed to save patient to database.")
        }
    }
}