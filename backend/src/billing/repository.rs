use super::models::{Invoice, InvoiceItem, Payment, PaymentStatus};
use chrono::{DateTime, Utc};
use reqwest::{Client, Response};
use serde::Deserialize;
use serde_json::json;
use std::collections::HashMap;
use std::future::Future;
use std::pin::Pin;
use std::sync::Mutex;

// Repository methods return boxed futures so both memory and Supabase storage use one interface.
pub type RepositoryFuture<'a, T> =
    Pin<Box<dyn Future<Output = Result<T, RepositoryError>> + Send + 'a>>;

#[derive(Debug)]
pub enum RepositoryError {
    NotFound,
    DuplicateAppointment,
    StorageUnavailable,
}

pub trait InvoiceRepository: Send + Sync {
    fn create(&self, invoice: Invoice) -> RepositoryFuture<'_, Invoice>;
    fn find_by_id(&self, invoice_id: &str) -> RepositoryFuture<'_, Invoice>;
    fn list(&self) -> RepositoryFuture<'_, Vec<Invoice>>;
    fn update(&self, invoice: Invoice) -> RepositoryFuture<'_, Invoice>;
}

#[derive(Default)]
pub struct InMemoryInvoiceRepository {
    invoices: Mutex<HashMap<String, Invoice>>,
}

impl InvoiceRepository for InMemoryInvoiceRepository {
    fn create(&self, invoice: Invoice) -> RepositoryFuture<'_, Invoice> {
        Box::pin(async move {
            let mut invoices = self
                .invoices
                .lock()
                .map_err(|_| RepositoryError::StorageUnavailable)?;

            if let Some(appointment_id) = invoice.appointment_id.as_deref() {
                let duplicate_exists = invoices.values().any(|existing| {
                    existing.appointment_id.as_deref() == Some(appointment_id)
                        && existing.status != PaymentStatus::Cancelled
                });
                if duplicate_exists {
                    return Err(RepositoryError::DuplicateAppointment);
                }
            }

            invoices.insert(invoice.id.clone(), invoice.clone());
            Ok(invoice)
        })
    }

    fn find_by_id(&self, invoice_id: &str) -> RepositoryFuture<'_, Invoice> {
        let invoice_id = invoice_id.to_string();
        Box::pin(async move {
            let invoices = self
                .invoices
                .lock()
                .map_err(|_| RepositoryError::StorageUnavailable)?;

            invoices
                .get(&invoice_id)
                .cloned()
                .ok_or(RepositoryError::NotFound)
        })
    }

    fn list(&self) -> RepositoryFuture<'_, Vec<Invoice>> {
        Box::pin(async move {
            let invoices = self
                .invoices
                .lock()
                .map_err(|_| RepositoryError::StorageUnavailable)?;

            let mut invoice_list: Vec<Invoice> = invoices.values().cloned().collect();
            invoice_list.sort_by_key(|invoice| std::cmp::Reverse(invoice.created_at));
            Ok(invoice_list)
        })
    }

    fn update(&self, invoice: Invoice) -> RepositoryFuture<'_, Invoice> {
        Box::pin(async move {
            let mut invoices = self
                .invoices
                .lock()
                .map_err(|_| RepositoryError::StorageUnavailable)?;

            if !invoices.contains_key(&invoice.id) {
                return Err(RepositoryError::NotFound);
            }

            invoices.insert(invoice.id.clone(), invoice.clone());
            Ok(invoice)
        })
    }
}

pub struct SupabaseInvoiceRepository {
    url: String,
    key: String,
    client: Client,
}

impl SupabaseInvoiceRepository {
    pub fn new(url: String, key: String) -> Self {
        Self {
            url: url.trim_end_matches('/').to_string(),
            key,
            client: Client::new(),
        }
    }

    fn request(&self, builder: reqwest::RequestBuilder) -> reqwest::RequestBuilder {
        let builder = builder
            .header("apikey", &self.key)
            .header("Content-Type", "application/json");

        if self.key.starts_with("eyJ") {
            builder.header("Authorization", format!("Bearer {}", self.key))
        } else {
            builder
        }
    }

    fn invoices_url(&self, query: &str) -> String {
        format!("{}/rest/v1/invoices?{}", self.url, query)
    }

    async fn decode_rows(response: Response) -> Result<Vec<DatabaseInvoice>, RepositoryError> {
        if !response.status().is_success() {
            return Err(Self::response_error(response).await);
        }
        response
            .json::<Vec<DatabaseInvoice>>()
            .await
            .map_err(|_| RepositoryError::StorageUnavailable)
    }

    async fn response_error(response: Response) -> RepositoryError {
        let status = response.status();
        let body = response.text().await.unwrap_or_default();
        if status.as_u16() == 409 || body.contains("active invoice already exists") {
            RepositoryError::DuplicateAppointment
        } else {
            eprintln!("Supabase billing error {status}: {body}");
            RepositoryError::StorageUnavailable
        }
    }
}

impl InvoiceRepository for SupabaseInvoiceRepository {
    fn create(&self, invoice: Invoice) -> RepositoryFuture<'_, Invoice> {
        Box::pin(async move {
            // The RPC saves the invoice and its items together to avoid half-saved invoices.
            let url = format!("{}/rest/v1/rpc/billing_create_invoice", self.url);
            let response = self
                .request(self.client.post(url))
                .json(&json!({ "p_invoice": invoice }))
                .send()
                .await
                .map_err(|_| RepositoryError::StorageUnavailable)?;

            if !response.status().is_success() {
                return Err(Self::response_error(response).await);
            }
            self.find_by_id(&invoice.id).await
        })
    }

    fn find_by_id(&self, invoice_id: &str) -> RepositoryFuture<'_, Invoice> {
        let invoice_id = invoice_id.to_string();
        Box::pin(async move {
            let encoded_id = urlencoding::encode(&invoice_id);
            let query = format!(
                "select=*,invoice_items(*),payments(*)&id=eq.{}&limit=1",
                encoded_id
            );
            let response = self
                .request(self.client.get(self.invoices_url(&query)))
                .send()
                .await
                .map_err(|_| RepositoryError::StorageUnavailable)?;
            let mut rows = Self::decode_rows(response).await?;
            rows.pop()
                .map(Invoice::from)
                .ok_or(RepositoryError::NotFound)
        })
    }

    fn list(&self) -> RepositoryFuture<'_, Vec<Invoice>> {
        Box::pin(async move {
            let response = self
                .request(self.client.get(
                    self.invoices_url(
                        "select=*,invoice_items(*),payments(*)&order=created_at.desc",
                    ),
                ))
                .send()
                .await
                .map_err(|_| RepositoryError::StorageUnavailable)?;
            Ok(Self::decode_rows(response)
                .await?
                .into_iter()
                .map(Invoice::from)
                .collect())
        })
    }

    fn update(&self, invoice: Invoice) -> RepositoryFuture<'_, Invoice> {
        Box::pin(async move {
            // Payments and invoice totals must be updated in the same database transaction.
            let url = format!("{}/rest/v1/rpc/billing_update_invoice", self.url);
            let response = self
                .request(self.client.post(url))
                .json(&json!({ "p_invoice": invoice }))
                .send()
                .await
                .map_err(|_| RepositoryError::StorageUnavailable)?;

            if !response.status().is_success() {
                return Err(Self::response_error(response).await);
            }
            self.find_by_id(&invoice.id).await
        })
    }
}

#[derive(Deserialize)]
struct DatabaseInvoice {
    id: String,
    patient_id: String,
    appointment_id: Option<String>,
    subtotal: f64,
    total: f64,
    status: PaymentStatus,
    created_at: DateTime<Utc>,
    paid_at: Option<DateTime<Utc>>,
    cancelled_at: Option<DateTime<Utc>>,
    #[serde(default)]
    invoice_items: Vec<InvoiceItem>,
    #[serde(default)]
    payments: Vec<Payment>,
}

impl From<DatabaseInvoice> for Invoice {
    fn from(row: DatabaseInvoice) -> Self {
        let amount_paid = round_money(row.payments.iter().map(|payment| payment.amount).sum());
        let balance_due = round_money((row.total - amount_paid).max(0.0));
        Self {
            id: row.id,
            patient_id: row.patient_id,
            appointment_id: row.appointment_id,
            items: row.invoice_items,
            subtotal: row.subtotal,
            total: row.total,
            status: row.status,
            payments: row.payments,
            amount_paid,
            balance_due,
            created_at: row.created_at,
            paid_at: row.paid_at,
            cancelled_at: row.cancelled_at,
        }
    }
}

fn round_money(value: f64) -> f64 {
    (value * 100.0).round() / 100.0
}
