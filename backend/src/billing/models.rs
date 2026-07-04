//! Billing data models.
//!
//! Defines invoices, invoice items, payments, form payloads, and medical report shapes
//! shared by billing handlers and services.
use chrono::{DateTime, Utc};
use serde::{Deserialize, Deserializer, Serialize};

#[derive(Clone, Debug, Serialize, Deserialize)]
pub struct Invoice {
    pub id: String,
    pub patient_id: String,
    pub appointment_id: Option<String>,
    pub items: Vec<InvoiceItem>,
    pub subtotal: f64,
    pub total: f64,
    pub status: PaymentStatus,
    pub payments: Vec<Payment>,
    pub amount_paid: f64,
    pub balance_due: f64,
    pub created_at: DateTime<Utc>,
    pub paid_at: Option<DateTime<Utc>>,
    pub cancelled_at: Option<DateTime<Utc>>,
}

#[derive(Clone, Debug, Serialize, Deserialize)]
pub struct InvoiceItem {
    pub item_type: InvoiceItemType,
    pub name: String,
    pub description: Option<String>,
    pub cost: f64,
}

#[derive(Clone, Debug, Serialize, Deserialize)]
pub enum InvoiceItemType {
    Treatment,
    Prescription,
}

#[derive(Clone, Debug, PartialEq, Serialize, Deserialize)]
pub enum PaymentStatus {
    Pending,
    Paid,
    Cancelled,
}

#[derive(Clone, Debug, Serialize, Deserialize)]
pub struct Payment {
    pub id: String,
    pub invoice_id: String,
    pub amount: f64,
    pub payment_method: String,
    pub transaction_reference: Option<String>,
    pub paid_at: DateTime<Utc>,
}

#[derive(Clone, Debug, Serialize)]
pub struct MedicineOption {
    pub name: &'static str,
    pub dosage: &'static str,
    pub unit_cost: f64,
}

#[derive(Clone, Debug, Deserialize)]
pub struct CreateInvoiceForm {
    pub patient_id: String,
    pub appointment_id: Option<String>,
    pub treatment_name: String,
    pub treatment_cost: String,
    #[serde(default, deserialize_with = "deserialize_form_vec")]
    pub prescription_names: Vec<String>,
    #[serde(default, deserialize_with = "deserialize_form_vec")]
    pub custom_prescription_names: Vec<String>,
    #[serde(default, deserialize_with = "deserialize_form_vec")]
    pub custom_prescription_costs: Vec<String>,
}

#[derive(Deserialize)]
#[serde(untagged)]
enum FormVec {
    One(String),
    Many(Vec<String>),
}

fn deserialize_form_vec<'de, D>(deserializer: D) -> Result<Vec<String>, D::Error>
where
    D: Deserializer<'de>,
{
    Ok(match Option::<FormVec>::deserialize(deserializer)? {
        Some(FormVec::One(value)) => vec![value],
        Some(FormVec::Many(values)) => values,
        None => Vec::new(),
    })
}

#[derive(Clone, Debug, Deserialize)]
pub struct RecordPaymentForm {
    pub amount: String,
    pub payment_method: String,
    pub transaction_reference: Option<String>,
}

#[derive(Clone, Debug, Serialize)]
pub struct MedicalReport {
    pub invoice: Invoice,
    pub generated_at: DateTime<Utc>,
}

#[derive(Clone, Debug, Serialize, Deserialize)]
pub struct ClinicalSummary {
    // The medical_records primary key column is "id", not "record_id" — this
    // was mismatched before, which made every lookup silently fail to
    // deserialize (serde errors on a missing key, even for an Option field).
    pub id: Option<String>,
    pub appointment_id: Option<String>,
    pub recorded_at: Option<DateTime<Utc>>,
    pub doctor_id: Option<String>,
    pub doctor_name: Option<String>,
    pub reason_of_visit: Option<String>,
    pub clinical_findings: Option<String>,
    pub diagnosis: Option<String>,
    pub doctor_notes: Option<String>,
    pub treatment_plan: Option<String>,
    pub prescribed_medicines: Option<String>,
}
