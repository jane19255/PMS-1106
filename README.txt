# Full-Stack Team Integration Guide (Rust + Actix + Firebase)

To ensure our application is highly secure and meets our strict project requirements, **the frontend will never talk directly to Google Firebase.** Here is our official data flow:
`Frontend (HTML/JS)  <--->  Rust Actix Server  <--->  Firebase Database`
---

## Backend
*Assigned Modules: Appointments, Medical Records, Billing, Staff Management*

Your job is to build the Actix routes that securely handle data for your specific feature. You do not need to write raw HTTP requests to Google; you will use the `FirebaseRestDb` client I built.

### Step 1: Define Your Data Model
Every piece of data that goes into our database needs a strict definition. Add your struct to `src/models.rs`.

```rust
// Inside src/models.rs
use serde::{Deserialize, Serialize};

// Example for Member 2 (Appointments)
#[derive(Serialize, Deserialize)]
pub struct Appointment {
    pub patient_id: String,
    pub date: String,
    pub purpose: String,
    pub status: String,
}


Step 2: Register Your Security Permissions
Before you build a route, you need to decide who is allowed to use it. Open src/handlers/auth.rs and update the Rules Engine:

1. Add your action to the AppAction enum.

2. Define which roles can perform that action in the has_permission function.

// Inside src/handlers/auth.rs
pub enum AppAction {
    CreatePatient,
    // Add your new actions here!
    CreateAppointment,
    DeleteAppointment, 
}

pub fn has_permission(role: &str, action: AppAction) -> bool {
    let normalized_role = role.to_lowercase();
    if normalized_role == "admin" { return true; } // Admins can do everything

    match action {
        // ... existing rules ...
        
        // Define your new rules here!
        AppAction::CreateAppointment => matches!(normalized_role.as_str(), "receptionist" | "doctor"),
        AppAction::DeleteAppointment => false, // Only admin can delete!
    }
}

Step 3: Build Your Route Handlers
Create a new file for your module (e.g., src/handlers/appointments.rs). Here is a complete template for a secure POST route that saves data to the database:

use actix_web::{web, HttpRequest, HttpResponse, Responder};
use serde_json::json;
use crate::models::Appointment;
use crate::db::FirebaseRestDb;
use crate::handlers::auth::{require_permission, AppAction};

pub async fn create_appointment(
    req: HttpRequest, // Required to check user cookies
    data: web::Json<Appointment>, // The data sent from the frontend
    db: web::Data<FirebaseRestDb>, // The shared database client
) -> impl Responder {
    
    // 1. SECURITY CHECK: Kick out unauthorized users immediately
    if let Err(rejection) = require_permission(&req, AppAction::CreateAppointment) {
        return rejection; 
    }

    // 2. FORMAT THE DATA FOR FIREBASE
    // Firestore requires data to be wrapped in a specific "fields" object
    let doc_id = format!("{}_{}", data.patient_id, data.date); // Create a unique ID
    let payload = json!({
        "fields": {
            "patientId": { "stringValue": data.patient_id },
            "date": { "stringValue": data.date },
            "purpose": { "stringValue": data.purpose },
            "status": { "stringValue": data.status }
        }
    });

    // 3. SEND TO DATABASE
    // Use the `create_document` abstraction (you can also use get_document, update_document, delete_document)
    match db.create_document("appointments", &doc_id, &payload).await {
        Ok(_) => HttpResponse::Ok().json(json!({ "status": "success", "id": doc_id })),
        Err(e) => {
            eprintln!("Database error: {}", e);
            HttpResponse::InternalServerError().body("Failed to save to database")
        }
    }
}