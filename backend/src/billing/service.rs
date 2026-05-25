use super::models::{
    CreateInvoiceForm, Invoice, InvoiceItem, InvoiceItemType, MedicalReport, PaymentStatus,
};
use super::repository::{InvoiceRepository, RepositoryError};
use chrono::Utc;
use std::sync::Arc;
use uuid::Uuid;

#[derive(Debug)]
pub enum BillingError {
    InvalidInput(String),
    InvoiceNotFound,
    StorageUnavailable,
}

pub struct BillingService {
    invoice_repository: Arc<dyn InvoiceRepository>,
}

impl BillingService {
    pub fn new(invoice_repository: Arc<dyn InvoiceRepository>) -> Self {
        Self { invoice_repository }
    }

    pub fn create_invoice(&self, form: CreateInvoiceForm) -> Result<Invoice, BillingError> {
        self.validate_create_invoice(&form)?;

        let treatment_cost = self.parse_cost(&form.treatment_cost, "Treatment cost")?;
        let prescription_cost = self.parse_optional_cost(&form.prescription_cost)?;

        let mut items = vec![InvoiceItem {
            item_type: InvoiceItemType::Treatment,
            name: form.treatment_name.trim().to_string(),
            description: None,
            cost: treatment_cost,
        }];

        if let Some(prescription_name) = form.prescription_name {
            if !prescription_name.trim().is_empty() {
                items.push(InvoiceItem {
                    item_type: InvoiceItemType::Prescription,
                    name: prescription_name.trim().to_string(),
                    description: None,
                    cost: prescription_cost,
                });
            }
        }

        let subtotal = self.calculate_total(&items);
        let invoice = Invoice {
            id: format!("INV-{}", Uuid::new_v4()),
            patient_id: form.patient_id.trim().to_string(),
            appointment_id: form
                .appointment_id
                .filter(|appointment_id| !appointment_id.trim().is_empty()),
            items,
            subtotal,
            total: subtotal,
            status: PaymentStatus::Pending,
            created_at: Utc::now(),
            paid_at: None,
        };

        self.invoice_repository
            .create(invoice)
            .map_err(Self::map_repository_error)
    }

    pub fn list_invoices(&self) -> Result<Vec<Invoice>, BillingError> {
        self.invoice_repository
            .list()
            .map_err(Self::map_repository_error)
    }

    pub fn find_invoice(&self, invoice_id: &str) -> Result<Invoice, BillingError> {
        self.invoice_repository
            .find_by_id(invoice_id)
            .map_err(Self::map_repository_error)
    }

    pub fn mark_invoice_paid(&self, invoice_id: &str) -> Result<Invoice, BillingError> {
        let mut invoice = self.find_invoice(invoice_id)?;

        if invoice.status == PaymentStatus::Cancelled {
            return Err(BillingError::InvalidInput(
                "Cancelled invoices cannot be paid.".to_string(),
            ));
        }

        invoice.status = PaymentStatus::Paid;
        invoice.paid_at = Some(Utc::now());

        self.invoice_repository
            .update(invoice)
            .map_err(Self::map_repository_error)
    }

    pub fn generate_medical_report(&self, invoice_id: &str) -> Result<MedicalReport, BillingError> {
        let invoice = self.find_invoice(invoice_id)?;

        Ok(MedicalReport {
            invoice,
            generated_at: Utc::now(),
        })
    }

    fn calculate_total(&self, items: &[InvoiceItem]) -> f64 {
        items.iter().map(|item| item.cost).sum()
    }

    fn validate_create_invoice(&self, form: &CreateInvoiceForm) -> Result<(), BillingError> {
        if form.patient_id.trim().is_empty() {
            return Err(BillingError::InvalidInput(
                "Patient ID is required.".to_string(),
            ));
        }

        if form.treatment_name.trim().is_empty() {
            return Err(BillingError::InvalidInput(
                "Treatment name is required.".to_string(),
            ));
        }

        let treatment_cost = self.parse_cost(&form.treatment_cost, "Treatment cost")?;
        if treatment_cost < 0.0 {
            return Err(BillingError::InvalidInput(
                "Treatment cost cannot be negative.".to_string(),
            ));
        }

        let prescription_cost = self.parse_optional_cost(&form.prescription_cost)?;
        if prescription_cost < 0.0 {
            return Err(BillingError::InvalidInput(
                "Prescription cost cannot be negative.".to_string(),
            ));
        }

        Ok(())
    }

    fn parse_optional_cost(&self, value: &Option<String>) -> Result<f64, BillingError> {
        match value {
            Some(cost) if !cost.trim().is_empty() => self.parse_cost(cost, "Prescription cost"),
            _ => Ok(0.0),
        }
    }

    fn parse_cost(&self, value: &str, field_name: &str) -> Result<f64, BillingError> {
        if value.trim().is_empty() {
            return Ok(0.0);
        }

        value.trim().parse::<f64>().map_err(|_| {
            BillingError::InvalidInput(format!("{field_name} must be a valid number."))
        })
    }

    fn map_repository_error(error: RepositoryError) -> BillingError {
        match error {
            RepositoryError::NotFound => BillingError::InvoiceNotFound,
            RepositoryError::StorageUnavailable => BillingError::StorageUnavailable,
        }
    }
}
