use super::models::{
    CreateInvoiceForm, Invoice, InvoiceItem, InvoiceItemType, MedicalReport, MedicineOption,
    Payment, PaymentStatus, RecordPaymentForm,
};
use super::repository::{InvoiceRepository, RepositoryError};
use chrono::Utc;
use std::collections::HashSet;
use std::sync::Arc;
use uuid::Uuid;

const CUSTOM_PRESCRIPTION_VALUE: &str = "__custom__";

#[derive(Debug, PartialEq)]
pub enum BillingError {
    InvalidInput(String),
    InvoiceNotFound,
    StorageUnavailable,
}

pub struct BillingService {
    invoice_repository: Arc<dyn InvoiceRepository>,
}

#[derive(Clone, Debug)]
struct PrescriptionCharge {
    name: String,
    cost: f64,
}

impl BillingService {
    pub fn new(invoice_repository: Arc<dyn InvoiceRepository>) -> Self {
        Self { invoice_repository }
    }

    pub fn medicine_catalog(&self) -> Vec<MedicineOption> {
        vec![
            MedicineOption {
                name: "Paracetamol",
                dosage: "500mg",
                unit_cost: 6.50,
            },
            MedicineOption {
                name: "Amoxicillin",
                dosage: "500mg",
                unit_cost: 18.00,
            },
            MedicineOption {
                name: "Ibuprofen",
                dosage: "400mg",
                unit_cost: 9.80,
            },
            MedicineOption {
                name: "Cetirizine",
                dosage: "10mg",
                unit_cost: 7.20,
            },
            MedicineOption {
                name: "Omeprazole",
                dosage: "20mg",
                unit_cost: 14.50,
            },
        ]
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

        let mut items = vec![InvoiceItem {
            item_type: InvoiceItemType::Treatment,
            name: form.treatment_name.trim().to_string(),
            description: None,
            cost: treatment_cost,
        }];

        for medicine in self.prescription_charges_from_form(&form)? {
            items.push(InvoiceItem {
                item_type: InvoiceItemType::Prescription,
                name: medicine.name,
                description: None,
                cost: medicine.cost,
            });
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

        let prescription_items = self.prescription_charges_from_form(form)?;
        let prescription_total: f64 = prescription_items
            .iter()
            .map(|medicine| medicine.cost)
            .sum();

        if self.round_money(treatment_cost + prescription_total) <= 0.0 {
            return Err(BillingError::InvalidInput(
                "Invoice total must be greater than zero.".to_string(),
            ));
        }

        Ok(())
    }

    fn prescription_charges_from_form(
        &self,
        form: &CreateInvoiceForm,
    ) -> Result<Vec<PrescriptionCharge>, BillingError> {
        let mut medicines = Vec::new();
        let catalog = self.medicine_catalog();
        let mut seen = HashSet::new();

        for name in form
            .prescription_names
            .iter()
            .map(|name| name.trim())
            .filter(|name| !name.is_empty() && *name != CUSTOM_PRESCRIPTION_VALUE)
        {
            let duplicate_key = name.to_lowercase();
            if !seen.insert(duplicate_key) {
                return Err(BillingError::InvalidInput(
                    "Duplicate prescription medicines are not allowed.".to_string(),
                ));
            }

            let medicine = catalog
                .iter()
                .find(|medicine| medicine.name == name)
                .cloned()
                .ok_or_else(|| {
                    BillingError::InvalidInput(
                        "Prescriptions must be selected from the medicine list.".to_string(),
                    )
                })?;
            medicines.push(PrescriptionCharge {
                name: self.prescription_label(medicine.name),
                cost: medicine.unit_cost,
            });
        }

        let custom_count = form
            .custom_prescription_names
            .len()
            .max(form.custom_prescription_costs.len());

        for index in 0..custom_count {
            let name = form
                .custom_prescription_names
                .get(index)
                .map(|name| name.trim())
                .unwrap_or("");
            let cost_raw = form
                .custom_prescription_costs
                .get(index)
                .map(|cost| cost.trim())
                .unwrap_or("");

            if name.is_empty() && cost_raw.is_empty() {
                continue;
            }

            let cost = self.parse_cost(cost_raw, "Custom prescription cost")?;
            if name.is_empty() {
                return Err(BillingError::InvalidInput(
                    "Custom medicine name is required when a custom prescription cost is entered."
                        .to_string(),
                ));
            }
            if name.len() > 200 {
                return Err(BillingError::InvalidInput(
                    "Custom medicine name cannot exceed 200 characters.".to_string(),
                ));
            }
            if cost <= 0.0 {
                return Err(BillingError::InvalidInput(
                    "Custom prescription cost must be greater than zero.".to_string(),
                ));
            }

            let duplicate_key = name.to_lowercase();
            if !seen.insert(duplicate_key) {
                return Err(BillingError::InvalidInput(
                    "Duplicate prescription medicines are not allowed.".to_string(),
                ));
            }

            medicines.push(PrescriptionCharge {
                name: name.to_string(),
                cost,
            });
        }

        Ok(medicines)
    }

    fn prescription_label(&self, name: &str) -> String {
        self.medicine_catalog()
            .into_iter()
            .find(|medicine| medicine.name == name)
            .map(|medicine| format!("{} {}", medicine.name, medicine.dosage))
            .unwrap_or_else(|| name.to_string())
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
            prescription_names: vec!["Paracetamol".to_string()],
            custom_prescription_names: Vec::new(),
            custom_prescription_costs: Vec::new(),
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

        assert_eq!(invoice.total, 86.5);
        assert_eq!(invoice.amount_paid, 0.0);
        assert_eq!(invoice.balance_due, 86.5);
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
        assert_eq!(updated.balance_due, 46.5);
        assert_eq!(updated.status, PaymentStatus::Pending);
        assert_eq!(updated.payments.len(), 1);
    }

    #[actix_web::test]
    async fn full_payment_marks_invoice_paid() {
        let service = service();
        let invoice = service.create_invoice(invoice_form(None)).await.unwrap();

        let updated = service
            .record_payment(&invoice.id, payment_form("86.50"))
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
            .record_payment(&invoice.id, payment_form("86.51"))
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
    async fn creates_one_invoice_item_per_prescribed_medicine() {
        let service = service();
        let mut form = invoice_form(None);
        form.prescription_names = vec!["Paracetamol".to_string(), "Amoxicillin".to_string()];

        let invoice = service.create_invoice(form).await.unwrap();

        assert_eq!(invoice.items.len(), 3);
        assert_eq!(invoice.total, 104.5);
        assert!(invoice
            .items
            .iter()
            .any(|item| item.name == "Paracetamol 500mg" && item.cost == 6.5));
        assert!(invoice
            .items
            .iter()
            .any(|item| item.name == "Amoxicillin 500mg" && item.cost == 18.0));
    }

    #[actix_web::test]
    async fn rejects_duplicate_prescription_medicine() {
        let service = service();
        let mut form = invoice_form(None);
        form.prescription_names = vec!["Paracetamol".to_string(), "Paracetamol".to_string()];

        let error = service.create_invoice(form).await.unwrap_err();

        assert_eq!(
            error,
            BillingError::InvalidInput(
                "Duplicate prescription medicines are not allowed.".to_string()
            )
        );
    }

    #[actix_web::test]
    async fn creates_invoice_with_custom_prescription_medicine() {
        let service = service();
        let mut form = invoice_form(None);
        form.prescription_names = vec![
            "Paracetamol".to_string(),
            CUSTOM_PRESCRIPTION_VALUE.to_string(),
        ];
        form.custom_prescription_names = vec!["Sterile eye drops".to_string()];
        form.custom_prescription_costs = vec!["12.35".to_string()];

        let invoice = service.create_invoice(form).await.unwrap();

        assert_eq!(invoice.items.len(), 3);
        assert_eq!(invoice.total, 98.85);
        assert!(invoice
            .items
            .iter()
            .any(|item| item.name == "Sterile eye drops" && item.cost == 12.35));
    }

    #[actix_web::test]
    async fn rejects_custom_prescription_cost_without_name() {
        let service = service();
        let mut form = invoice_form(None);
        form.prescription_names = vec![CUSTOM_PRESCRIPTION_VALUE.to_string()];
        form.custom_prescription_names = vec!["".to_string()];
        form.custom_prescription_costs = vec!["12.35".to_string()];

        let error = service.create_invoice(form).await.unwrap_err();

        assert_eq!(
            error,
            BillingError::InvalidInput(
                "Custom medicine name is required when a custom prescription cost is entered."
                    .to_string()
            )
        );
    }

    #[actix_web::test]
    async fn rejects_custom_prescription_name_without_positive_cost() {
        let service = service();
        let mut form = invoice_form(None);
        form.prescription_names = vec![CUSTOM_PRESCRIPTION_VALUE.to_string()];
        form.custom_prescription_names = vec!["Sterile eye drops".to_string()];
        form.custom_prescription_costs = vec!["0".to_string()];

        let error = service.create_invoice(form).await.unwrap_err();

        assert_eq!(
            error,
            BillingError::InvalidInput(
                "Custom prescription cost must be greater than zero.".to_string()
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
