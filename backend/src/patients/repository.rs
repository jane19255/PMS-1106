//! This module contains the repository functions for interacting with the Supabase database for patient-related operations.
//! It provides functions to list, get, create, update, and delete patients in the database.
//!
//! It is different from the handlers module, which handles HTTP requests and responses.

use serde_json::Value;

use crate::db::SupabaseRestDb;
use crate::models::{PatientView, SupabasePatientRow};

pub async fn list_patients(db: &SupabaseRestDb) -> Result<Vec<PatientView>, String> {
    let body = db.list_patients().await?;
    let rows: Vec<SupabasePatientRow> = serde_json::from_str(&body)
        .map_err(|error| format!("Failed to parse patients from database: {error}"))?;

    Ok(rows.into_iter().map(PatientView::from).collect())
}

pub async fn get_patient(
    db: &SupabaseRestDb,
    patient_id: &str,
) -> Result<Option<PatientView>, String> {
    let body = db.get_patient(patient_id).await?;
    let rows: Vec<SupabasePatientRow> = serde_json::from_str(&body)
        .map_err(|error| format!("Failed to parse patient from database: {error}"))?;

    Ok(rows.into_iter().next().map(PatientView::from))
}

pub async fn create_patient(db: &SupabaseRestDb, payload: &Value) -> Result<(), String> {
    db.create_patient(payload).await.map(|_| ())
}

pub async fn update_patient(
    db: &SupabaseRestDb,
    patient_id: &str,
    payload: &Value,
) -> Result<(), String> {
    db.update_patient(patient_id, payload).await.map(|_| ())
}

pub async fn delete_patient(db: &SupabaseRestDb, patient_id: &str) -> Result<(), String> {
    db.delete_patient(patient_id).await.map(|_| ())
}