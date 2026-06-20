use super::models::{
    CreateInvoiceForm, Invoice, InvoiceItem, InvoiceItemType, MedicalReport, Payment,
    PaymentStatus, RecordPaymentForm,
};
use super::repository::{InvoiceRepository, RepositoryError};
use chrono::Utc;
use std::sync::Arc;
use uuid::Uuid;

#[derive(Debug, PartialEq)]
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

    pub async fn create_invoice(&self, form: CreateInvoiceForm) -> Result<Invoice, BillingError> {
        self.validate_create_invoice(&form)?;

        if let Some(appointment_id) = form
            .appointment_id
            .as_deref()
            .map(str::trim)
            .filter(|id| !id.is_empty())
        {
            let duplicate_exists = self.list_invoices().await?.iter().any(|invoice| {
                invoice.appointment_id.as_deref() == Some(appointment_id)
                    && invoice.status != PaymentStatus::Cancelled
            });
            if duplicate_exists {
                return Err(BillingError::InvalidInput(
                    "An active invoice already exists for this appointment.".to_string(),
                ));
            }
        }

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
            payments: Vec::new(),
            amount_paid: 0.0,
            balance_due: subtotal,
            created_at: Utc::now(),
            paid_at: None,
            cancelled_at: None,
        };

        self.invoice_repository
            .create(invoice)
            .await
            .map_err(Self::map_repository_error)
    }

    pub async fn list_invoices(&self) -> Result<Vec<Invoice>, BillingError> {
        self.invoice_repository
            .list()
            .await
            .map_err(Self::map_repository_error)
    }

    pub async fn find_invoice(&self, invoice_id: &str) -> Result<Invoice, BillingError> {
        self.invoice_repository
            .find_by_id(invoice_id)
            .await
            .map_err(Self::map_repository_error)
    }

    pub async fn record_payment(
        &self,
        invoice_id: &str,
        form: RecordPaymentForm,
    ) -> Result<Invoice, BillingError> {
        let mut invoice = self.find_invoice(invoice_id).await?;

        match invoice.status {
            PaymentStatus::Cancelled => {
                return Err(BillingError::InvalidInput(
                    "Cancelled invoices cannot receive payments.".to_string(),
                ));
            }
            PaymentStatus::Paid => {
                return Err(BillingError::InvalidInput(
                    "This invoice is already fully paid.".to_string(),
                ));
            }
            PaymentStatus::Pending => {}
        }

        let amount = self.parse_cost(&form.amount, "Payment amount")?;
        if amount <= 0.0 {
            return Err(BillingError::InvalidInput(
                "Payment amount must be greater than zero.".to_string(),
            ));
        }
        if amount > invoice.balance_due + 0.001 {
            return Err(BillingError::InvalidInput(
                "Payment amount cannot exceed the outstanding balance.".to_string(),
            ));
        }

        let payment_method = form.payment_method.trim();
        if payment_method.is_empty() {
            return Err(BillingError::InvalidInput(
                "Payment method is required.".to_string(),
            ));
        }
        if payment_method.len() > 50 {
            return Err(BillingError::InvalidInput(
                "Payment method cannot exceed 50 characters.".to_string(),
            ));
        }

        let paid_at = Utc::now();
        invoice.payments.push(Payment {
            id: format!("PAY-{}", Uuid::new_v4()),
            invoice_id: invoice.id.clone(),
            amount,
            payment_method: payment_method.to_string(),
            transaction_reference: form
                .transaction_reference
                .map(|reference| reference.trim().to_string())
                .filter(|reference| !reference.is_empty()),
            paid_at,
        });
        invoice.amount_paid = self.round_money(invoice.amount_paid + amount);
        invoice.balance_due = self.round_money(invoice.total - invoice.amount_paid);

        if invoice.balance_due <= 0.0 {
            invoice.balance_due = 0.0;
            invoice.status = PaymentStatus::Paid;
            invoice.paid_at = Some(paid_at);
        }

        self.invoice_repository
            .update(invoice)
            .await
            .map_err(Self::map_repository_error)
    }

    pub async fn cancel_invoice(&self, invoice_id: &str) -> Result<Invoice, BillingError> {
        let mut invoice = self.find_invoice(invoice_id).await?;

        if invoice.status == PaymentStatus::Paid || invoice.amount_paid > 0.0 {
            return Err(BillingError::InvalidInput(
                "Invoices with recorded payments cannot be cancelled.".to_string(),
            ));
        }
        if invoice.status == PaymentStatus::Cancelled {
            return Err(BillingError::InvalidInput(
                "This invoice is already cancelled.".to_string(),
            ));
        }

        invoice.status = PaymentStatus::Cancelled;
        invoice.cancelled_at = Some(Utc::now());

        self.invoice_repository
            .update(invoice)
            .await
            .map_err(Self::map_repository_error)
    }

    pub async fn generate_medical_report(
        &self,
        invoice_id: &str,
    ) -> Result<MedicalReport, BillingError> {
        let invoice = self.find_invoice(invoice_id).await?;

        Ok(MedicalReport {
            invoice,
            generated_at: Utc::now(),
        })
    }

    fn calculate_total(&self, items: &[InvoiceItem]) -> f64 {
        self.round_money(items.iter().map(|item| item.cost).sum())
    }

    fn validate_create_invoice(&self, form: &CreateInvoiceForm) -> Result<(), BillingError> {
        if form.patient_id.trim().is_empty() {
            return Err(BillingError::InvalidInput(
                "Patient ID is required.".to_string(),
            ));
        }
        if form.patient_id.trim().len() > 100 {
            return Err(BillingError::InvalidInput(
                "Patient ID cannot exceed 100 characters.".to_string(),
            ));
        }

        if form.treatment_name.trim().is_empty() {
            return Err(BillingError::InvalidInput(
                "Treatment name is required.".to_string(),
            ));
        }
        if form.treatment_name.trim().len() > 200 {
            return Err(BillingError::InvalidInput(
                "Treatment name cannot exceed 200 characters.".to_string(),
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

        let has_prescription_name = form
            .prescription_name
            .as_deref()
            .map(str::trim)
            .is_some_and(|name| !name.is_empty());
        if prescription_cost > 0.0 && !has_prescription_name {
            return Err(BillingError::InvalidInput(
                "Prescription name is required when a prescription cost is entered.".to_string(),
            ));
        }

        if self.round_money(treatment_cost + prescription_cost) <= 0.0 {
            return Err(BillingError::InvalidInput(
                "Invoice total must be greater than zero.".to_string(),
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

        let amount = value.trim().parse::<f64>().map_err(|_| {
            BillingError::InvalidInput(format!("{field_name} must be a valid number."))
        })?;

        if !amount.is_finite() {
            return Err(BillingError::InvalidInput(format!(
                "{field_name} must be a finite number."
            )));
        }

        Ok(self.round_money(amount))
    }

    fn round_money(&self, value: f64) -> f64 {
        (value * 100.0).round() / 100.0
    }

    fn map_repository_error(error: RepositoryError) -> BillingError {
        match error {
            RepositoryError::NotFound => BillingError::InvoiceNotFound,
            RepositoryError::DuplicateAppointment => BillingError::InvalidInput(
                "An active invoice already exists for this appointment.".to_string(),
            ),
            RepositoryError::StorageUnavailable => BillingError::StorageUnavailable,
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::billing::repository::InMemoryInvoiceRepository;
    use std::sync::Barrier;
    use std::thread;

    fn service() -> BillingService {
        BillingService::new(Arc::new(InMemoryInvoiceRepository::default()))
    }

    fn invoice_form(appointment_id: Option<&str>) -> CreateInvoiceForm {
        CreateInvoiceForm {
            patient_id: "PAT-001".to_string(),
            appointment_id: appointment_id.map(str::to_string),
            treatment_name: "Consultation".to_string(),
            treatment_cost: "80.00".to_string(),
            prescription_name: Some("Medication".to_string()),
            prescription_cost: Some("20.00".to_string()),
        }
    }

    fn payment_form(amount: &str) -> RecordPaymentForm {
        RecordPaymentForm {
            amount: amount.to_string(),
            payment_method: "Card".to_string(),
            transaction_reference: Some("TXN-001".to_string()),
        }
    }

    #[actix_web::test]
    async fn creates_invoice_with_rounded_total_and_balance() {
        let service = service();
        let invoice = service
            .create_invoice(invoice_form(Some("APP-001")))
            .await
            .unwrap();

        assert_eq!(invoice.total, 100.0);
        assert_eq!(invoice.amount_paid, 0.0);
        assert_eq!(invoice.balance_due, 100.0);
        assert_eq!(invoice.status, PaymentStatus::Pending);
    }

    #[actix_web::test]
    async fn rejects_duplicate_active_invoice_for_appointment() {
        let service = service();
        service
            .create_invoice(invoice_form(Some("APP-001")))
            .await
            .unwrap();

        let error = service
            .create_invoice(invoice_form(Some("APP-001")))
            .await
            .unwrap_err();

        assert_eq!(
            error,
            BillingError::InvalidInput(
                "An active invoice already exists for this appointment.".to_string()
            )
        );
    }

    #[actix_web::test]
    async fn records_partial_payment_and_keeps_invoice_pending() {
        let service = service();
        let invoice = service.create_invoice(invoice_form(None)).await.unwrap();

        let updated = service
            .record_payment(&invoice.id, payment_form("40.00"))
            .await
            .unwrap();

        assert_eq!(updated.amount_paid, 40.0);
        assert_eq!(updated.balance_due, 60.0);
        assert_eq!(updated.status, PaymentStatus::Pending);
        assert_eq!(updated.payments.len(), 1);
    }

    #[actix_web::test]
    async fn full_payment_marks_invoice_paid() {
        let service = service();
        let invoice = service.create_invoice(invoice_form(None)).await.unwrap();

        let updated = service
            .record_payment(&invoice.id, payment_form("100.00"))
            .await
            .unwrap();

        assert_eq!(updated.balance_due, 0.0);
        assert_eq!(updated.status, PaymentStatus::Paid);
        assert!(updated.paid_at.is_some());
    }

    #[actix_web::test]
    async fn rejects_payment_above_outstanding_balance() {
        let service = service();
        let invoice = service.create_invoice(invoice_form(None)).await.unwrap();

        let error = service
            .record_payment(&invoice.id, payment_form("100.01"))
            .await
            .unwrap_err();

        assert_eq!(
            error,
            BillingError::InvalidInput(
                "Payment amount cannot exceed the outstanding balance.".to_string()
            )
        );
    }

    #[actix_web::test]
    async fn cancellation_releases_appointment_for_new_invoice() {
        let service = service();
        let invoice = service
            .create_invoice(invoice_form(Some("APP-001")))
            .await
            .unwrap();
        let cancelled = service.cancel_invoice(&invoice.id).await.unwrap();

        assert_eq!(cancelled.status, PaymentStatus::Cancelled);
        assert!(cancelled.cancelled_at.is_some());
        assert!(service
            .create_invoice(invoice_form(Some("APP-001")))
            .await
            .is_ok());
    }

    #[actix_web::test]
    async fn rejects_cancellation_after_partial_payment() {
        let service = service();
        let invoice = service.create_invoice(invoice_form(None)).await.unwrap();
        service
            .record_payment(&invoice.id, payment_form("10.00"))
            .await
            .unwrap();

        let error = service.cancel_invoice(&invoice.id).await.unwrap_err();

        assert_eq!(
            error,
            BillingError::InvalidInput(
                "Invoices with recorded payments cannot be cancelled.".to_string()
            )
        );
    }

    #[actix_web::test]
    async fn rejects_prescription_cost_without_name() {
        let service = service();
        let mut form = invoice_form(None);
        form.prescription_name = None;

        let error = service.create_invoice(form).await.unwrap_err();

        assert_eq!(
            error,
            BillingError::InvalidInput(
                "Prescription name is required when a prescription cost is entered.".to_string()
            )
        );
    }

    #[test]
    fn concurrent_requests_create_only_one_invoice_per_appointment() {
        let service = Arc::new(service());
        let barrier = Arc::new(Barrier::new(2));
        let mut workers = Vec::new();

        for _ in 0..2 {
            let service = service.clone();
            let barrier = barrier.clone();
            workers.push(thread::spawn(move || {
                barrier.wait();
                actix_web::rt::System::new().block_on(async move {
                    service
                        .create_invoice(invoice_form(Some("APP-CONCURRENT")))
                        .await
                })
            }));
        }

        let results: Vec<_> = workers
            .into_iter()
            .map(|worker| worker.join().unwrap())
            .collect();

        assert_eq!(results.iter().filter(|result| result.is_ok()).count(), 1);
        assert_eq!(results.iter().filter(|result| result.is_err()).count(), 1);
    }
}
