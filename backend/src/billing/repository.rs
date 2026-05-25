use super::models::Invoice;
use std::collections::HashMap;
use std::sync::Mutex;

#[derive(Debug)]
pub enum RepositoryError {
    NotFound,
    StorageUnavailable,
}

pub trait InvoiceRepository: Send + Sync {
    fn create(&self, invoice: Invoice) -> Result<Invoice, RepositoryError>;
    fn find_by_id(&self, invoice_id: &str) -> Result<Invoice, RepositoryError>;
    fn list(&self) -> Result<Vec<Invoice>, RepositoryError>;
    fn update(&self, invoice: Invoice) -> Result<Invoice, RepositoryError>;
}

#[derive(Default)]
pub struct InMemoryInvoiceRepository {
    invoices: Mutex<HashMap<String, Invoice>>,
}

impl InvoiceRepository for InMemoryInvoiceRepository {
    fn create(&self, invoice: Invoice) -> Result<Invoice, RepositoryError> {
        let mut invoices = self
            .invoices
            .lock()
            .map_err(|_| RepositoryError::StorageUnavailable)?;

        invoices.insert(invoice.id.clone(), invoice.clone());
        Ok(invoice)
    }

    fn find_by_id(&self, invoice_id: &str) -> Result<Invoice, RepositoryError> {
        let invoices = self
            .invoices
            .lock()
            .map_err(|_| RepositoryError::StorageUnavailable)?;

        invoices
            .get(invoice_id)
            .cloned()
            .ok_or(RepositoryError::NotFound)
    }

    fn list(&self) -> Result<Vec<Invoice>, RepositoryError> {
        let invoices = self
            .invoices
            .lock()
            .map_err(|_| RepositoryError::StorageUnavailable)?;

        let mut invoice_list: Vec<Invoice> = invoices.values().cloned().collect();
        invoice_list.sort_by(|left, right| right.created_at.cmp(&left.created_at));
        Ok(invoice_list)
    }

    fn update(&self, invoice: Invoice) -> Result<Invoice, RepositoryError> {
        let mut invoices = self
            .invoices
            .lock()
            .map_err(|_| RepositoryError::StorageUnavailable)?;

        if !invoices.contains_key(&invoice.id) {
            return Err(RepositoryError::NotFound);
        }

        invoices.insert(invoice.id.clone(), invoice.clone());
        Ok(invoice)
    }
}
