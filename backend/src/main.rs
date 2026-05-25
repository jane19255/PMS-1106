mod billing;
mod routes;

use actix_files::Files;
use actix_web::{web, App, HttpServer};
use billing::repository::InMemoryInvoiceRepository;
use billing::service::BillingService;
use std::sync::Arc;
use tera::Tera;

#[actix_web::main]
async fn main() -> std::io::Result<()> {
    let templates = Tera::new("templates/**/*.html")
        .expect("templates directory should contain valid Tera templates");

    let invoice_repository = Arc::new(InMemoryInvoiceRepository::default());
    let billing_service = web::Data::new(BillingService::new(invoice_repository));
    let template_data = web::Data::new(templates);

    HttpServer::new(move || {
        App::new()
            .app_data(billing_service.clone())
            .app_data(template_data.clone())
            .service(Files::new("/static", "static"))
            .configure(routes::configure)
    })
    .bind(("127.0.0.1", 8080))?
    .run()
    .await
}
