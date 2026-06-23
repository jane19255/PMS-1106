use actix_web::{web, HttpRequest, HttpResponse, Responder};
use firebase_auth::FirebaseAuth;
use serde_json::json;
use tera::{Context, Tera};

use crate::db::{FirebaseRestDb, SupabaseRestDb};
use crate::handlers::auth::{require_auth_and_permission, AppAction};
use crate::models::{Patient, PatientView, SupabasePatientRow, UpdatePatient};

pub fn routes(cfg: &mut web::ServiceConfig) {
    cfg.route("/patients", web::get().to(patients_page));
    cfg.route("/api/patients", web::get().to(list_patients));
    cfg.route("/api/patients/new", web::post().to(create_patient));
    cfg.route("/api/patients/{id}", web::put().to(update_patient));
    cfg.route("/api/patients/{id}", web::delete().to(delete_patient));
}

pub async fn patients_page(
    req: HttpRequest,
    tera: web::Data<Tera>,
    firebase_auth: web::Data<FirebaseAuth>,
    firestore_db: web::Data<FirebaseRestDb>,
) -> impl Responder {
    if let Err(rejection) =
        require_auth_and_permission(&req, &firebase_auth, &firestore_db, AppAction::ViewPatient)
            .await
    {
        return rejection;
    }

    let mut ctx = Context::new();
    ctx.insert(
        "firebase_api_key",
        &std::env::var("FIREBASE_API_KEY").unwrap_or_default(),
    );
    ctx.insert(
        "firebase_project_id",
        &std::env::var("FIREBASE_PROJECT_ID").unwrap_or_default(),
    );

    match tera.render("Patients.html", &ctx) {
        Ok(html) => HttpResponse::Ok()
            .content_type("text/html; charset=utf-8")
            .body(html),
        Err(e) => {
            eprintln!("Template error: {e}");
            HttpResponse::InternalServerError().body("Template rendering failed")
        }
    }
}

pub async fn list_patients(
    req: HttpRequest,
    firebase_auth: web::Data<FirebaseAuth>,
    firestore_db: web::Data<FirebaseRestDb>,
    supabase_db: web::Data<SupabaseRestDb>,
) -> impl Responder {
    if let Err(rejection) =
        require_auth_and_permission(&req, &firebase_auth, &firestore_db, AppAction::ViewPatient)
            .await
    {
        return rejection;
    }

    match supabase_db.list_patients().await {
        Ok(body) => match serde_json::from_str::<Vec<SupabasePatientRow>>(&body) {
            Ok(rows) => {
                let patients: Vec<PatientView> = rows.into_iter().map(PatientView::from).collect();
                HttpResponse::Ok().json(patients)
            }
            Err(e) => {
                eprintln!("Failed to parse Supabase patient rows: {e} | Body: {body}");
                HttpResponse::InternalServerError().body("Failed to parse patients from database.")
            }
        },
        Err(e) => {
            eprintln!("Failed to list patients from Supabase: {e}");
            HttpResponse::InternalServerError().body("Failed to load patients from database.")
        }
    }
}

pub async fn create_patient(
    req: HttpRequest,
    patient_data: web::Json<Patient>,
    firebase_auth: web::Data<FirebaseAuth>,
    firestore_db: web::Data<FirebaseRestDb>,
    supabase_db: web::Data<SupabaseRestDb>,
) -> impl Responder {
    if let Err(rejection) = require_auth_and_permission(
        &req,
        &firebase_auth,
        &firestore_db,
        AppAction::CreatePatient,
    )
    .await
    {
        return rejection;
    }

    let patient_id = patient_data.nric.trim();
    if patient_id.is_empty() {
        return HttpResponse::BadRequest().body("Patient NRIC/FIN/Passport cannot be empty.");
    }

    let payload = json!({
        "id": patient_id,
        "first_name": patient_data.first_name,
        "last_name": patient_data.last_name,
        "dob": patient_data.dob,
        "gender": patient_data.gender,
        "nric": patient_data.nric,
        "nationality": patient_data.nationality,
        "phone": patient_data.phone,
        "email": patient_data.email,
        "emergency_name": patient_data.emergency_name,
        "emergency_phone": patient_data.emergency_phone,
        "address": patient_data.address,
        "allergies": patient_data.allergies,
        "medications": patient_data.medications,
        "conditions": patient_data.conditions,
        "status": "Active"
    });

    match supabase_db.create_patient(&payload).await {
        Ok(_) => {
            println!(
                "Successfully registered patient in Supabase: {}",
                patient_id
            );
            HttpResponse::Ok().json(json!({ "status": "success" }))
        }
        Err(e) => {
            eprintln!("Failed to create patient in Supabase: {e}");
            if e.contains("duplicate key") || e.contains("23505") {
                HttpResponse::Conflict()
                    .body("A patient with this NRIC/FIN/Passport already exists.")
            } else {
                HttpResponse::InternalServerError().body("Failed to save patient to database.")
            }
        }
    }
}

pub async fn update_patient(
    req: HttpRequest,
    path: web::Path<String>,
    patient_data: web::Json<UpdatePatient>,
    firebase_auth: web::Data<FirebaseAuth>,
    firestore_db: web::Data<FirebaseRestDb>,
    supabase_db: web::Data<SupabaseRestDb>,
) -> impl Responder {
    if let Err(rejection) =
        require_auth_and_permission(&req, &firebase_auth, &firestore_db, AppAction::EditPatient)
            .await
    {
        return rejection;
    }

    let patient_id = path.into_inner();
    let payload = json!({
        "first_name": patient_data.first_name,
        "last_name": patient_data.last_name,
        "dob": patient_data.dob,
        "gender": patient_data.gender,
        "nric": patient_data.nric,
        "nationality": patient_data.nationality,
        "phone": patient_data.phone,
        "email": patient_data.email,
        "emergency_name": patient_data.emergency_name,
        "emergency_phone": patient_data.emergency_phone,
        "address": patient_data.address,
        "allergies": patient_data.allergies,
        "medications": patient_data.medications,
        "conditions": patient_data.conditions
    });

    match supabase_db.update_patient(&patient_id, &payload).await {
        Ok(_) => HttpResponse::Ok().json(json!({ "status": "success" })),
        Err(e) => {
            eprintln!("Failed to update patient in Supabase: {e}");
            HttpResponse::InternalServerError().body("Failed to update patient.")
        }
    }
}

pub async fn delete_patient(
    req: HttpRequest,
    path: web::Path<String>,
    firebase_auth: web::Data<FirebaseAuth>,
    firestore_db: web::Data<FirebaseRestDb>,
    supabase_db: web::Data<SupabaseRestDb>,
) -> impl Responder {
    if let Err(rejection) = require_auth_and_permission(
        &req,
        &firebase_auth,
        &firestore_db,
        AppAction::DeletePatient,
    )
    .await
    {
        return rejection;
    }

    let patient_id = path.into_inner();
    match supabase_db.delete_patient(&patient_id).await {
        Ok(_) => HttpResponse::Ok().json(json!({ "status": "success" })),
        Err(e) => {
            eprintln!("Failed to delete patient in Supabase: {e}");
            HttpResponse::InternalServerError().body("Failed to delete patient.")
        }
    }
}
