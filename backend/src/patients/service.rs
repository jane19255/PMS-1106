//! This module contains the service functions for patient-related operations in the web application.
//! It provides functions to validate and normalize patient data, as well as to construct payloads for database operations.
//! 
//! It is different from the handlers and the repository module, as it focuses on data validation and normalization.

use chrono::NaiveDate;
use serde_json::{json, Value};

use crate::db::singapore_today;
use crate::models::{Patient, UpdatePatient};

const VALID_GENDERS: [&str; 2] = ["Male", "Female"];
const VALID_PATIENT_STATUSES: [&str; 2] = ["Active", "Inactive"];

#[derive(Debug, Clone)]
pub(crate) struct ValidatedPatient {
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

pub(crate) fn patient_nric(patient: &ValidatedPatient) -> &str {
    &patient.nric
}

pub(crate) fn patient_status(patient: &ValidatedPatient) -> Option<&str> {
    patient.status.as_deref()
}

pub(crate) fn validate_new_patient(patient: &Patient) -> Result<ValidatedPatient, String> {
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

pub(crate) fn validate_updated_patient(patient: &UpdatePatient) -> Result<ValidatedPatient, String> {
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

pub(crate) fn patient_payload(
    patient: &ValidatedPatient,
    id: Option<&str>,
    status: Option<&str>,
) -> Value {
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

    if dob > singapore_today() {
        Err("Date of birth cannot be in the future.".to_string())
    } else {
        Ok(trimmed)
    }
}

pub(crate) fn normalize_identifier(value: &str) -> String {
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