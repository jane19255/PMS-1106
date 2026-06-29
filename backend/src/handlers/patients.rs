use actix_web::{web, HttpRequest, HttpResponse, Responder};
use chrono::{NaiveDate, Utc};
use firebase_auth::FirebaseAuth;
use serde_json::{json, Value};
use tera::{Context, Tera};
use uuid::Uuid;

use crate::db::{FirebaseRestDb, SupabaseRestDb};
use crate::handlers::auth::{require_auth_and_permission, AppAction};
use crate::models::{Patient, PatientView, SupabasePatientRow, UpdatePatient};

#[derive(serde::Deserialize)]
struct CreateMedicalRecordForm {
    patient_id: String,
    doctor_id: Option<String>,
    reason_of_visit: Option<String>,
    clinical_findings: Option<String>,
    diagnosis: Option<String>,
    doctor_notes: Option<String>,
    blood_pressure: Option<String>,
    temperature: Option<String>,
    pulse_rate: Option<String>,
    height_cm: Option<String>,
    weight_kg: Option<String>,
}

#[derive(serde::Deserialize)]
struct MedicalRecordRow {
    id: String,
    patient_id: String,
    doctor_id: Option<String>,
    diagnosis: Option<String>,
    doctor_notes: Option<String>,
    recorded_at: Option<String>,
    reason_of_visit: Option<String>,
    clinical_findings: Option<String>,
    blood_pressure: Option<String>,
    temperature: Option<String>,
    pulse_rate: Option<String>,
    height_cm: Option<String>,
    weight_kg: Option<String>,
}

#[derive(serde::Serialize)]
struct HistoryEntry {
    id: String,
    patient_id: String,
    doctor_id: Option<String>,
    doctor_name: Option<String>,
    diagnosis: Option<String>,
    doctor_notes: Option<String>,
    reason_of_visit: Option<String>,
    clinical_findings: Option<String>,
    blood_pressure: Option<String>,
    temperature: Option<String>,
    pulse_rate: Option<String>,
    height_cm: Option<String>,
    weight_kg: Option<String>,
    recorded_at: Option<String>,
}

const VALID_GENDERS: [&str; 2] = ["Male", "Female"];
const VALID_PATIENT_STATUSES: [&str; 2] = ["Active", "Inactive"];

#[derive(Debug, Clone)]
struct ValidatedPatient {
    first_name: String,
    last_name: String,
    dob: String,
    gender: String,
    nric: String,
    nationality: String,
    phone: String,
    email: String,
    emergency_name: Option<String>,
    emergency_phone: Option<String>,
    address: Option<String>,
    allergies: Option<String>,
    medications: Option<String>,
    conditions: Option<String>,
    status: Option<String>,
}

pub fn routes(cfg: &mut web::ServiceConfig) {
    cfg.route("/patients", web::get().to(patients_page));
    cfg.route("/api/patients", web::get().to(list_patients));
    cfg.route("/api/patients/new", web::post().to(create_patient));
    cfg.route("/api/patients/{id}", web::get().to(get_patient_by_id));
    cfg.route("/api/patients/{id}", web::put().to(update_patient));
    cfg.route("/api/patients/{id}", web::delete().to(delete_patient));
    cfg.route("/api/patients/{id}/history", web::get().to(get_patient_history));
    cfg.route("/api/medical-records", web::post().to(create_medical_record));
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
    match supabase_db.get_patient(&patient_id).await {
        Ok(body) => match serde_json::from_str::<Vec<SupabasePatientRow>>(&body) {
            Ok(rows) => match rows.into_iter().next().map(PatientView::from) {
                Some(patient) => HttpResponse::Ok().json(patient),
                None => HttpResponse::NotFound().body("Patient not found"),
            },
            Err(e) => {
                eprintln!("Failed to parse patient {patient_id}: {e}");
                HttpResponse::InternalServerError().body("Failed to parse patient")
            }
        },
        Err(e) => {
            eprintln!("Failed to fetch patient {patient_id}: {e}");
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

    let patient_id = validated.nric.clone();
    let payload = patient_payload(&validated, Some(&patient_id), Some("Active"));

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

    if validated.nric != patient_id {
        return HttpResponse::BadRequest()
            .body("Patient NRIC/FIN cannot be changed after registration.");
    }

    let payload = patient_payload(&validated, None, validated.status.as_deref());

    match supabase_db.update_patient(&patient_id, &payload).await {
        Ok(_) => HttpResponse::Ok().json(json!({ "status": "success" })),
        Err(e) => {
            eprintln!("Failed to update patient in Supabase: {e}");
            HttpResponse::InternalServerError().body("Failed to update patient.")
        }
    }
}

fn validate_new_patient(patient: &Patient) -> Result<ValidatedPatient, String> {
    validate_patient_fields(
        &patient.first_name,
        &patient.last_name,
        &patient.dob,
        &patient.gender,
        &patient.nric,
        &patient.nationality,
        &patient.phone,
        &patient.email,
        patient.emergency_name.as_deref(),
        patient.emergency_phone.as_deref(),
        patient.address.as_deref(),
        patient.allergies.as_deref(),
        patient.medications.as_deref(),
        patient.conditions.as_deref(),
        Some("Active"),
    )
}

fn validate_updated_patient(patient: &UpdatePatient) -> Result<ValidatedPatient, String> {
    validate_patient_fields(
        &patient.first_name,
        &patient.last_name,
        &patient.dob,
        &patient.gender,
        &patient.nric,
        &patient.nationality,
        &patient.phone,
        &patient.email,
        patient.emergency_name.as_deref(),
        patient.emergency_phone.as_deref(),
        patient.address.as_deref(),
        patient.allergies.as_deref(),
        patient.medications.as_deref(),
        patient.conditions.as_deref(),
        patient.status.as_deref(),
    )
}

#[allow(clippy::too_many_arguments)]
fn validate_patient_fields(
    first_name: &str,
    last_name: &str,
    dob: &str,
    gender: &str,
    nric: &str,
    nationality: &str,
    phone: &str,
    email: &str,
    emergency_name: Option<&str>,
    emergency_phone: Option<&str>,
    address: Option<&str>,
    allergies: Option<&str>,
    medications: Option<&str>,
    conditions: Option<&str>,
    status: Option<&str>,
) -> Result<ValidatedPatient, String> {
    let first_name = required_text(first_name, "First name")?;
    let last_name = required_text(last_name, "Last name")?;
    let dob = validate_dob(dob)?;
    let gender = validate_choice(gender, "Gender", &VALID_GENDERS)?;
    let nric = validate_nric(nric)?;
    let nationality = required_text(nationality, "Nationality")?;
    let phone = validate_phone(phone, "Phone number")?;
    let email = validate_email(email)?;
    let emergency_name = optional_text(emergency_name);
    let emergency_phone = match optional_text(emergency_phone) {
        Some(value) => Some(validate_phone(&value, "Emergency phone number")?),
        None => None,
    };
    let status = match status {
        Some(value) => Some(validate_choice(
            value,
            "Patient status",
            &VALID_PATIENT_STATUSES,
        )?),
        None => None,
    };

    Ok(ValidatedPatient {
        first_name,
        last_name,
        dob,
        gender,
        nric,
        nationality,
        phone,
        email,
        emergency_name,
        emergency_phone,
        address: optional_text(address),
        allergies: optional_text(allergies),
        medications: optional_text(medications),
        conditions: optional_text(conditions),
        status,
    })
}

fn patient_payload(patient: &ValidatedPatient, id: Option<&str>, status: Option<&str>) -> Value {
    let mut payload = json!({
        "first_name": patient.first_name,
        "last_name": patient.last_name,
        "dob": patient.dob,
        "gender": patient.gender,
        "nric": patient.nric,
        "nationality": patient.nationality,
        "phone": patient.phone,
        "email": patient.email,
        "emergency_name": patient.emergency_name,
        "emergency_phone": patient.emergency_phone,
        "address": patient.address,
        "allergies": patient.allergies,
        "medications": patient.medications,
        "conditions": patient.conditions
    });

    if let Some(id) = id {
        payload["id"] = json!(id);
    }

    if let Some(status) = status {
        payload["status"] = json!(status);
    }

    payload
}

fn required_text(value: &str, label: &str) -> Result<String, String> {
    let trimmed = value.trim();
    if trimmed.is_empty() {
        Err(format!("{label} is required."))
    } else {
        Ok(trimmed.to_string())
    }
}

fn optional_text(value: Option<&str>) -> Option<String> {
    value
        .map(str::trim)
        .filter(|value| !value.is_empty())
        .map(ToOwned::to_owned)
}

fn validate_choice(value: &str, label: &str, allowed: &[&str]) -> Result<String, String> {
    let trimmed = required_text(value, label)?;
    allowed
        .iter()
        .find(|candidate| candidate.eq_ignore_ascii_case(&trimmed))
        .map(|candidate| (*candidate).to_string())
        .ok_or_else(|| format!("{label} must be one of: {}.", allowed.join(", ")))
}

fn validate_dob(value: &str) -> Result<String, String> {
    let trimmed = required_text(value, "Date of birth")?;
    let dob = NaiveDate::parse_from_str(&trimmed, "%Y-%m-%d")
        .map_err(|_| "Date of birth must use YYYY-MM-DD format.".to_string())?;

    if dob > Utc::now().date_naive() {
        Err("Date of birth cannot be in the future.".to_string())
    } else {
        Ok(trimmed)
    }
}

fn normalize_identifier(value: &str) -> String {
    value
        .chars()
        .filter(|character| !character.is_whitespace() && *character != '-')
        .flat_map(char::to_uppercase)
        .collect()
}

fn validate_nric(value: &str) -> Result<String, String> {
    let normalized = normalize_identifier(value);
    let mut chars = normalized.chars();
    let first = chars.next();
    let last = normalized.chars().last();

    let valid = normalized.len() == 9
        && matches!(first, Some('S' | 'T' | 'F' | 'G' | 'M'))
        && normalized
            .chars()
            .skip(1)
            .take(7)
            .all(|character| character.is_ascii_digit())
        && matches!(last, Some('A'..='Z'));

    if valid {
        Ok(normalized)
    } else {
        Err("NRIC/FIN must follow the Singapore format, for example S1234567D.".to_string())
    }
}

fn validate_phone(value: &str, label: &str) -> Result<String, String> {
    let mut digits: String = value
        .trim()
        .chars()
        .filter(|character| character.is_ascii_digit())
        .collect();

    if digits.starts_with("65") && digits.len() == 10 {
        digits = digits[2..].to_string();
    }

    let valid = digits.len() == 8
        && digits
            .chars()
            .next()
            .is_some_and(|first| matches!(first, '6' | '8' | '9'));

    if valid {
        Ok(digits)
    } else {
        Err(format!("{label} must be an 8-digit Singapore number."))
    }
}

fn validate_email(value: &str) -> Result<String, String> {
    let email = required_text(value, "Email")?.to_lowercase();
    let (local, domain) = email
        .split_once('@')
        .ok_or_else(|| "Email must contain @.".to_string())?;

    let valid = !local.is_empty()
        && !domain.is_empty()
        && domain.contains('.')
        && !email.contains(char::is_whitespace)
        && !email.starts_with('@')
        && !email.ends_with('@');

    if valid {
        Ok(email)
    } else {
        Err("Email must be a valid email address.".to_string())
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    fn valid_patient() -> Patient {
        Patient {
            first_name: "Jane".to_string(),
            last_name: "Tan".to_string(),
            dob: "1990-03-14".to_string(),
            gender: "Female".to_string(),
            nric: "S1234567D".to_string(),
            nationality: "Singapore".to_string(),
            phone: "91234567".to_string(),
            email: "Jane.Tan@example.com".to_string(),
            emergency_name: Some("John Tan".to_string()),
            emergency_phone: Some("+65 8123 4567".to_string()),
            address: Some("123 Clinic Street".to_string()),
            allergies: None,
            medications: None,
            conditions: None,
        }
    }

    #[test]
    fn validates_and_normalizes_patient_registration() {
        let validated = validate_new_patient(&valid_patient()).expect("patient should validate");

        assert_eq!(validated.nric, "S1234567D");
        assert_eq!(validated.phone, "91234567");
        assert_eq!(validated.email, "jane.tan@example.com");
        assert_eq!(validated.emergency_phone.as_deref(), Some("81234567"));
        assert_eq!(validated.status.as_deref(), Some("Active"));
    }

    #[test]
    fn rejects_invalid_patient_registration_fields() {
        let mut patient = valid_patient();
        patient.nric = "1234567".to_string();
        assert!(validate_new_patient(&patient).is_err());

        patient = valid_patient();
        patient.phone = "12345678".to_string();
        assert!(validate_new_patient(&patient).is_err());

        patient = valid_patient();
        patient.email = "not-an-email".to_string();
        assert!(validate_new_patient(&patient).is_err());

        patient = valid_patient();
        patient.first_name = " ".to_string();
        assert!(validate_new_patient(&patient).is_err());
    }

    #[test]
    fn validates_patient_update_status() {
        let update = UpdatePatient {
            first_name: "Jane".to_string(),
            last_name: "Tan".to_string(),
            dob: "1990-03-14".to_string(),
            gender: "Female".to_string(),
            nric: "S1234567D".to_string(),
            nationality: "Singapore".to_string(),
            phone: "91234567".to_string(),
            email: "jane@example.com".to_string(),
            emergency_name: None,
            emergency_phone: None,
            address: None,
            allergies: None,
            medications: None,
            conditions: None,
            status: Some("Inactive".to_string()),
        };

        let validated = validate_updated_patient(&update).expect("update should validate");
        assert_eq!(validated.status.as_deref(), Some("Inactive"));
    }
}

pub async fn get_patient_history(
    req: HttpRequest,
    firebase_auth: web::Data<FirebaseAuth>,
    firestore_db: web::Data<FirebaseRestDb>,
    supabase_db: web::Data<SupabaseRestDb>,
    path: web::Path<String>,
) -> impl Responder {
    if let Err(rejection) = require_auth_and_permission(
        &req,
        &firebase_auth,
        &firestore_db,
        AppAction::ViewMedicalRecords,
    )
    .await
    {
        return rejection;
    }
    let patient_id = path.into_inner();
    let encoded_id = urlencoding::encode(&patient_id);
    let query = format!(
        "patient_id=eq.{}&select=*&order=recorded_at.desc.nullslast",
        encoded_id
    );

    match supabase_db.fetch_table("medical_records", &query).await {
        Ok(body) => match serde_json::from_str::<Vec<MedicalRecordRow>>(&body) {
            Ok(rows) => {
                let mut entries: Vec<HistoryEntry> = Vec::new();
                for row in rows {
                    let doctor_name = if let Some(ref did) = row.doctor_id {
                        fetch_doctor_name(&supabase_db, did).await.ok()
                    } else {
                        None
                    };
                    entries.push(HistoryEntry {
                        doctor_name,
                        id: row.id,
                        patient_id: row.patient_id,
                        doctor_id: row.doctor_id,
                        diagnosis: row.diagnosis,
                        doctor_notes: row.doctor_notes,
                        reason_of_visit: row.reason_of_visit,
                        clinical_findings: row.clinical_findings,
                        blood_pressure: row.blood_pressure,
                        temperature: row.temperature,
                        pulse_rate: row.pulse_rate,
                        height_cm: row.height_cm,
                        weight_kg: row.weight_kg,
                        recorded_at: row.recorded_at,
                    });
                }
                HttpResponse::Ok().json(entries)
            }
            Err(e) => {
                eprintln!("Failed to parse medical records: {e} | body: {body}");
                HttpResponse::InternalServerError().body("Failed to parse medical records")
            }
        },
        Err(e) => {
            eprintln!("Failed to fetch history for {patient_id}: {e}");
            HttpResponse::InternalServerError().body("Failed to load medical records")
        }
    }
}

pub async fn create_medical_record(
    req: HttpRequest,
    firebase_auth: web::Data<FirebaseAuth>,
    firestore_db: web::Data<FirebaseRestDb>,
    supabase_db: web::Data<SupabaseRestDb>,
    form: web::Json<CreateMedicalRecordForm>,
) -> impl Responder {
    if let Err(rejection) = require_auth_and_permission(
        &req,
        &firebase_auth,
        &firestore_db,
        AppAction::EditMedicalRecords,
    )
    .await
    {
        return rejection;
    }

    if form.patient_id.trim().is_empty() {
        return HttpResponse::BadRequest().body("patient_id is required");
    }

    let id = format!("MR-{}", Uuid::new_v4());
    let payload = json!({
        "id": id,
        "patient_id": form.patient_id.trim(),
        "doctor_id": form.doctor_id.as_deref().filter(|s| !s.trim().is_empty()),
        "diagnosis": form.diagnosis.as_deref().filter(|s| !s.trim().is_empty()),
        "doctor_notes": form.doctor_notes.as_deref().filter(|s| !s.trim().is_empty()),
        "reason_of_visit": form.reason_of_visit.as_deref().filter(|s| !s.trim().is_empty()),
        "clinical_findings": form.clinical_findings.as_deref().filter(|s| !s.trim().is_empty()),
        "blood_pressure": form.blood_pressure.as_deref().filter(|s| !s.trim().is_empty()),
        "temperature": form.temperature.as_deref().filter(|s| !s.trim().is_empty()),
        "pulse_rate": form.pulse_rate.as_deref().filter(|s| !s.trim().is_empty()),
        "height_cm": form.height_cm.as_deref().filter(|s| !s.trim().is_empty()),
        "weight_kg": form.weight_kg.as_deref().filter(|s| !s.trim().is_empty()),
    });

    match supabase_db.insert_row("medical_records", &payload).await {
        Ok(body) => {
            if let Ok(rows) = serde_json::from_str::<Vec<serde_json::Value>>(&body) {
                if let Some(row) = rows.into_iter().next() {
                    return HttpResponse::Created().json(row);
                }
            }
            HttpResponse::Created().json(json!({ "id": id }))
        }
        Err(e) => {
            eprintln!("Failed to create medical record: {e}");
            HttpResponse::InternalServerError().body(format!("Failed to save medical record: {e}"))
        }
    }
}

async fn fetch_doctor_name(supabase_db: &SupabaseRestDb, doctor_id: &str) -> Result<String, ()> {
    #[derive(serde::Deserialize)]
    struct StaffEmbed {
        full_name: String,
    }
    #[derive(serde::Deserialize)]
    struct DoctorRow {
        staff: StaffEmbed,
    }

    let encoded_id = urlencoding::encode(doctor_id);
    let url = supabase_db.rest_url(&format!(
        "doctors?id=eq.{}&select=staff:staff_id(full_name)",
        encoded_id
    ));
    let response = supabase_db
        .authed(supabase_db.http_client.get(&url))
        .send()
        .await
        .map_err(|_| ())?;
    let rows: Vec<DoctorRow> = response.json().await.map_err(|_| ())?;
    rows.into_iter().next().map(|r| r.staff.full_name).ok_or(())
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
