use chrono::{DateTime, Utc};
use serde::{Deserialize, Serialize};

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

#[derive(Clone, Debug, Deserialize)]
pub struct CreateInvoiceForm {
    pub patient_id: String,
    pub appointment_id: Option<String>,
    pub treatment_name: String,
    pub treatment_cost: String,
    pub prescription_name: Option<String>,
    pub prescription_cost: Option<String>,
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
