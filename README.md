# Patient Management System

## Overview

This project is a Rust, Actix Web, Tera, and Supabase based Patient Management System for managing patient registration, appointments, medical records, billing, payments, prescriptions, authentication, role-based access control, and medical reports.

The implementation is organized as an enterprise-style SSR web application. Backend routes delegate business rules to service modules, persistence is isolated behind repository/database abstractions, and Tera templates render user-facing pages on the server.

## Tech Stack

- Rust 2021
- Actix Web
- Tera server-side templates
- Supabase PostgreSQL through REST/RPC endpoints
- Firebase authentication integration
- HTML, CSS, and JavaScript for SSR-enhanced pages
- `printpdf` for backend-generated PDF medical reports

## Main Features

- Patient registration and management
- User authentication and role-based access control
- Appointment scheduling and queue management
- Medical records and patient history timeline
- Billing, invoices, payments, and medical reports
- Doctor management and prescription tracking
- Server-side rendered pages
- Form validation and business workflow logic

## Member Scopes

### Member 1: Patient Registration, Authentication, and Database

- Implement patient registration.
- Implement user authentication, including login and logout.
- Manage user sessions.
- Implement role-based access control.
- Design and maintain the relational database schema.
- Provide reusable database access abstractions for other backend members.
- Define shared Rust data models with the team.
- Validate patient and user input data.
- Securely hash and store user passwords.

### Member 2: Appointment Scheduling and Queue Management

- Implement appointment CRUD operations.
- Validate appointment conflicts.
- Manage appointment queue tracking.
- Track patient queue status.
- Manage doctor availability and scheduling logic.
- Create SSR pages for appointment booking.
- Create SSR pages for queue display.

### Member 3: Medical Records and Patient History Timeline

- Implement medical record CRUD operations.
- Link appointments to medical records.
- Track treatments and doctor notes.
- Build patient history timeline data.
- Secure access to sensitive medical records.
- Create SSR pages for medical record display.
- Create SSR pages for patient history timeline display.

### Member 4: Billing System and Medical Report Generation

- Implement invoice creation.
- Implement payment tracking.
- Track billing status such as pending, paid, and cancelled.
- Include treatments and prescriptions in billing.
- Validate billing inputs.
- Handle billing errors.
- Generate print-friendly medical reports.
- Create SSR pages for billing, invoice details, and reports.

### Member 5: Doctor Management and Prescription Tracking

- Manage doctor profiles.
- Manage doctor schedules.
- Manage doctor availability.
- Implement prescription CRUD operations.
- Enforce role-based access control for doctor-related actions.
- Integrate prescriptions with medical records and billing.
- Create SSR pages for doctor and prescription management.

### Member 6: Frontend and SSR UI Integration

- Build shared SSR layouts and reusable templates.
- Create responsive frontend styling.
- Build dashboard and navigation pages.
- Integrate frontend pages with backend module routes.
- Create consistent forms, tables, and detail pages.
- Support dynamic rendering of data from all backend modules.
- Ensure the UI is usable across desktop and mobile screens.

## Current Implementation

The project currently provides an authenticated hospital management web portal with server-rendered pages, protected module routes, Supabase-backed patient data, Firebase-based login sessions, and a fully implemented billing/reporting workflow.

Implemented modules include:

- Authentication and session management through Firebase ID tokens and secure cookies
- Role-based access checks for patient, billing, and report actions
- Dashboard and shared navigation for core hospital modules
- Patient management page and JSON APIs for listing, creating, updating, and deleting patients
- Backend patient validation for required fields, NRIC/FIN, phone, email, date of birth, gender, duplicate NRIC, and status values
- Appointment and queue management UI pages
- Medical records UI pages and Supabase medical record schema support
- Billing dashboard with invoice creation, filtering, sorting, pagination, payment recording, cancellation, and invoice detail pages
- Prescription billing with catalog medicines, custom medicines, multiple medicine rows, server-side price validation, and invoice total calculation
- Medical report generation using patient details, clinical record context, billing items, payment records, and printable clinic-style formatting
- Backend-generated PDF report downloads using Rust `printpdf`
- Repository/service separation for billing business rules and persistence
- In-memory billing repository for tests and Supabase billing repository for persistent storage
- PostgreSQL/Supabase schema for patients, staff, doctors, appointments, queues, medical records, medicine inventory, prescriptions, invoices, invoice items, and payments

## Database Schema

Apply `backend/supabase_schema.sql` in the Supabase SQL Editor. It contains the
complete clinical, queue, prescription, and billing schema.

Core relationships:

- `patients` is the main patient registration table. `id` is the normalized
  NRIC/FIN and is protected by a unique constraint.
- `staff` stores backend staff profiles that map to Firebase users through
  `firebase_uid`. Its `role` values match the backend RBAC roles:
  `admin`, `doctor`, `receptionist`, and `pharmacist`.
- `doctors` extends `staff` for clinical users and is referenced by
  appointments, medical records, and prescriptions.
- `appointments` links a patient to a doctor at a scheduled time.
- `patient_queue` tracks appointment check-in and queue progress.
- `medical_records` links clinical notes to a patient, and optionally to an
  appointment and doctor.
- `medicine_inventory` stores medicine stock, cost, and active/inactive status.
- `prescriptions` links prescribed medicine work to a patient, doctor, and
  optional medical record.
- `prescription_items` stores the individual medicines, dosage, frequency,
  duration, and quantity for a prescription.
- `invoices` links billing to a patient and optional appointment.
- `invoice_items` stores treatment or prescription charges for an invoice.
- `payments` stores invoice payment transactions.

All core tables enable row-level security. The current Rust backend uses a
Supabase service/secret key and performs access control through Firebase RBAC
before calling Supabase.

## Project Structure

```text
backend/
  Cargo.toml
  Cargo.lock
  supabase_schema.sql
  src/
    main.rs
    routes.rs
    db.rs
    models.rs
    billing/
      handlers.rs
      models.rs
      pdf.rs
      repository.rs
      service.rs
    handlers/
      auth.rs
      patients.rs
  templates/
    layout.html
    header.html
    error.html
    dashboard.html
    Patients.html
    Appointments.html
    Medical-Records.html
    Doctor-Dashboard.html
    Staffs.html
    billing/
      index.html
      show.html
      report.html
frontend/
  assets/
    css/
    js/
```

## Setup

Create `backend/.env` from `backend/.env.example` and configure:

```text
FIREBASE_PROJECT_ID=...
SUPABASE_URL=...
SUPABASE_KEY=...
BILLING_STORAGE=supabase
```

Use a Supabase secret/service key for the backend. The backend is designed to keep Supabase writes server-side.

## Database Setup

Run the single schema script:

```text
backend/supabase_schema.sql
```

`supabase_schema.sql` creates all application tables, indexes, triggers, and
transactional RPC functions, including the billing workflow.

## Running the Application

From the backend folder:

```bash
cargo run
```

Open:

```text
http://127.0.0.1:8080
```

After login, use the dashboard navigation to open patients, appointments, records, staff, doctor dashboard, or billing.

## Main Routes

Page routes:

```text
GET /login
GET /dashboard
GET /doctor-dashboard
GET /staff
GET /patients
GET /appointments
GET /records
GET /billing
```

Session and user routes:

```text
POST /session
GET  /logout
POST /forgot-password
GET  /api/me
```

Patient API routes:

```text
GET    /api/patients
POST   /api/patients/new
PUT    /api/patients/{id}
DELETE /api/patients/{id}
```

Billing routes:

```text
GET  /billing
POST /billing/invoices
GET  /billing/invoices/{invoice_id}
POST /billing/invoices/{invoice_id}/payments
POST /billing/invoices/{invoice_id}/cancel
GET  /billing/invoices/{invoice_id}/report
GET  /billing/invoices/{invoice_id}/report.pdf
```

## Module Notes

### Authentication and Access Control

Users authenticate with Firebase. The backend verifies Firebase ID tokens, stores the authenticated session in an HTTP-only cookie, and checks module permissions before allowing protected actions. Role permissions are centralized in the auth handler so pages and APIs can reuse the same access rules.

### Patient Management

Patient data is stored in Supabase and exposed through protected JSON APIs. The patient pages use these APIs to load and manage patient profiles while keeping database writes on the backend.

### Appointments and Medical Records

Appointment and medical record pages are included as server-rendered module pages. Medical records are represented in the Supabase schema and can be linked to patients and appointments so reports can include diagnosis, doctor notes, treatment plans, and prescribed medicines.

### Billing and Reports

The system supports two report outputs:

- HTML clinic-style medical report rendered by Tera for browser viewing and Chrome print/save-as-PDF.
- Backend-generated PDF from `/billing/invoices/{invoice_id}/report.pdf` using Rust `printpdf`.

Billing includes invoice creation, treatment charges, catalog and custom prescription charges, payment tracking, cancellation rules, and report generation. Both report outputs include patient information, clinical record details, prescriptions, billing items, totals, and payment records.

## Testing

Run all backend tests:

```bash
cargo test
```

Run only billing service tests:

```bash
cargo test billing::service::tests
```

The tests cover:

- role-based billing permissions
- invoice totals and balances
- partial and full payments
- cancellation rules
- duplicate appointment billing
- duplicate prescription selection
- multiple prescription medicines
- custom prescription medicine validation
- concurrent invoice creation protection
- Tera template parsing

## Demonstration Walkthrough

1. Log in through Firebase authentication.
2. Open the dashboard and show the shared navigation across hospital modules.
3. Open the patient page and demonstrate protected patient data loading from Supabase.
4. Open appointment and medical record pages to show the wider patient workflow.
5. Open billing and select a patient from the backend patient selector.
6. Create an invoice with treatment cost, catalog medicines, and a custom medicine.
7. Open the invoice detail page and record a partial payment.
8. Show that balance due updates and invoice remains pending.
9. Record the remaining payment and show that the invoice becomes paid.
10. Open the medical report and show patient, clinical, billing, and payment sections.
11. Use browser print to show the clinic-style report.
12. Download `/report.pdf` to show backend-side Rust PDF generation.
13. Explain role checks, service/repository separation, Supabase schema/RPC functions, and automated tests.

## Key Technical Decisions

- Firebase handles identity, while the Rust backend owns session cookies and authorization decisions.
- Supabase is used for relational patient, medical record, and billing persistence.
- Server-side rendering keeps the app simple to deploy and avoids duplicating page rendering logic in a separate frontend framework.
- The `BillingService` contains business rules so handlers stay focused on HTTP and authorization.
- `InvoiceRepository` allows the same service logic to use in-memory storage for tests or Supabase storage for persistence.
- Prescription costs are calculated server-side from a medicine catalog rather than trusting browser-submitted prices.
- Payment and cancellation rules are enforced in both Rust service logic and Supabase RPC/database constraints.
- HTML print reports and backend PDF reports are both supported for usability and technical complexity.

## Notes

- `backend/target/` is generated by Cargo and should not be committed.
- `Cargo.lock` should be committed because this is an application project.
- `.env` files should not be committed.
