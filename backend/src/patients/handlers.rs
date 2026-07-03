//! This module contains the HTTP handlers for patient-related operations in the web application.
//! It decides the HTTP responses based on the results of service and repository functions.
//! 
//! It is different from the repository module, which handles direct database interactions.

use actix_web::{web, HttpRequest, HttpResponse, Responder};
use firebase_auth::FirebaseAuth;
use serde_json::json;
use tera::{Context, Tera};

use super::repository;
use super::service::{
    normalize_identifier, patient_nric, patient_payload, patient_status, validate_new_patient,
    validate_updated_patient,
};

use crate::db::{FirebaseRestDb, SupabaseRestDb};
use crate::auth::{require_auth_and_permission, AppAction};
use crate::models::{Patient, UpdatePatient};

pub fn routes(cfg: &mut web::ServiceConfig) {
    cfg.route("/patients", web::get().to(patients_page));
    cfg.route("/api/patients", web::get().to(list_patients));
    cfg.route("/api/patients/new", web::post().to(create_patient));
    cfg.route("/api/patients/{id}", web::get().to(get_patient_by_id));
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

    let ctx = Context::new();

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

    match repository::list_patients(&supabase_db).await {
        Ok(patients) => HttpResponse::Ok().json(patients),
        Err(error) => {
            eprintln!("Failed to list patients from Supabase: {error}");
            HttpResponse::InternalServerError().body("Failed to load patients from database.")
        }
    }
}

pub async fn get_patient_by_id(
    req: HttpRequest,
    firebase_auth: web::Data<FirebaseAuth>,
    firestore_db: web::Data<FirebaseRestDb>,
    supabase_db: web::Data<SupabaseRestDb>,
    path: web::Path<String>,
) -> impl Responder {
    if let Err(rejection) =
        require_auth_and_permission(&req, &firebase_auth, &firestore_db, AppAction::ViewPatient)
            .await
    {
        return rejection;
    }

    let patient_id = path.into_inner();
    match repository::get_patient(&supabase_db, &patient_id).await {
        Ok(Some(patient)) => HttpResponse::Ok().json(patient),
        Ok(None) => HttpResponse::NotFound().body("Patient not found"),
        Err(error) => {
            eprintln!("Failed to fetch patient {patient_id}: {error}");
            HttpResponse::InternalServerError().body("Failed to load patient")
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

    let validated = match validate_new_patient(&patient_data) {
        Ok(patient) => patient,
        Err(message) => return HttpResponse::BadRequest().body(message),
    };

    let patient_id = patient_nric(&validated).to_string();
    let payload = patient_payload(&validated, Some(&patient_id), Some("Active"));

    match repository::create_patient(&supabase_db, &payload).await {
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
                HttpResponse::Conflict().body("A patient with this NRIC/FIN already exists.")
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

    let patient_id = normalize_identifier(&path.into_inner());
    let validated = match validate_updated_patient(&patient_data) {
        Ok(patient) => patient,
        Err(message) => return HttpResponse::BadRequest().body(message),
    };

    if patient_nric(&validated) != patient_id {
        return HttpResponse::BadRequest()
            .body("Patient NRIC/FIN cannot be changed after registration.");
    }

    let payload = patient_payload(&validated, None, patient_status(&validated));

    match repository::update_patient(&supabase_db, &patient_id, &payload).await {
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
    match repository::delete_patient(&supabase_db, &patient_id).await {
        Ok(_) => HttpResponse::Ok().json(json!({ "status": "success" })),
        Err(e) => {
            eprintln!("Failed to delete patient in Supabase: {e}");
            HttpResponse::InternalServerError().body("Failed to delete patient.")
        }
    }
}