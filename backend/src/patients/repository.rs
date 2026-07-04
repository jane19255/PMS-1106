//! This module contains the repository functions for interacting with the Supabase database for patient-related operations.
//! It provides functions to list, get, create, update, and delete patients in the database.
//!
//! It is different from the handlers module, which handles HTTP requests and responses.

use serde_json::Value;

use super::models::{PatientView, SupabasePatientRow};
use crate::db::SupabaseRestDb;

pub async fn list_patients(db: &SupabaseRestDb) -> Result<Vec<PatientView>, String> {
    let url = db.rest_url("patients?select=*&order=created_at.desc.nullslast");
    let response = db
        .authed(db.http_client.get(url))
        .send()
        .await
        .map_err(|error| error.to_string())?;
    let body = SupabaseRestDb::read_response(response).await?;
    let rows: Vec<SupabasePatientRow> = serde_json::from_str(&body)
        .map_err(|error| format!("Failed to parse patients from database: {error}"))?;

    Ok(rows.into_iter().map(PatientView::from).collect())
}

pub async fn get_patient(
    db: &SupabaseRestDb,
    patient_id: &str,
) -> Result<Option<PatientView>, String> {
    let encoded_id = urlencoding::encode(patient_id);
    let url = db.rest_url(&format!("patients?id=eq.{}&select=*&limit=1", encoded_id));
    let response = db
        .authed(db.http_client.get(url))
        .send()
        .await
        .map_err(|error| error.to_string())?;
    let body = SupabaseRestDb::read_response(response).await?;
    let rows: Vec<SupabasePatientRow> = serde_json::from_str(&body)
        .map_err(|error| format!("Failed to parse patient from database: {error}"))?;

    Ok(rows.into_iter().next().map(PatientView::from))
}

pub async fn create_patient(db: &SupabaseRestDb, payload: &Value) -> Result<(), String> {
    let url = db.rest_url("patients");
    let response = db
        .authed(db.http_client.post(url))
        .header("Prefer", "return=representation")
        .json(payload)
        .send()
        .await
        .map_err(|error| error.to_string())?;
    SupabaseRestDb::read_response(response).await.map(|_| ())
}

pub async fn update_patient(
    db: &SupabaseRestDb,
    patient_id: &str,
    payload: &Value,
) -> Result<(), String> {
    let encoded_id = urlencoding::encode(patient_id);
    let url = db.rest_url(&format!("patients?id=eq.{}", encoded_id));
    let response = db
        .authed(db.http_client.patch(url))
        .header("Prefer", "return=representation")
        .json(payload)
        .send()
        .await
        .map_err(|error| error.to_string())?;
    SupabaseRestDb::read_response(response).await.map(|_| ())
}

pub async fn delete_patient(db: &SupabaseRestDb, patient_id: &str) -> Result<(), String> {
    let encoded_id = urlencoding::encode(patient_id);
    let url = db.rest_url(&format!("patients?id=eq.{}", encoded_id));
    let response = db
        .authed(db.http_client.delete(url))
        .send()
        .await
        .map_err(|error| error.to_string())?;
    SupabaseRestDb::read_response(response).await.map(|_| ())
}
