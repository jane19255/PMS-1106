# CSCare Patient Management System

A server-rendered hospital management application built with Rust, Actix Web, Tera, Supabase PostgreSQL, and Firebase Authentication.

The system supports patient registration, appointments, reception queues, doctor workflows, medical records, prescriptions, staff administration, billing, payments, and medical report generation. Application data is loaded from backend APIs and persisted in Supabase when the storage settings are configured as described below.

## Key Features

- Firebase Authentication with HTTP-only backend sessions
- Role-based access control for administrators, receptionists, doctors, and pharmacists
- Patient registration and profile management
- Appointment scheduling with database-enforced conflict prevention
- Patient check-in, vital-sign recording, priority queues, and room assignment
- Doctor profiles, schedules, availability, and consultation workflows
- Medical record CRUD operations and patient history timelines
- Prescription management and medicine inventory integration
- Invoice creation, payment tracking, cancellation rules, and balance calculation
- Printable HTML medical reports and backend-generated PDF reports
- Supabase RPC functions for multi-table operations that must complete atomically

## Technology Stack

| Area | Technology |
| --- | --- |
| Backend | Rust 2021, Actix Web |
| Templates | Tera server-side rendering |
| Database | Supabase PostgreSQL through REST and RPC APIs |
| Authentication | Firebase Authentication and Firebase Admin REST APIs |
| Frontend | HTML, CSS, and vanilla JavaScript |
| PDF generation | `printpdf` |
| Testing | Rust unit, template, and external integration tests |

## Architecture Explanation

The backend follows a handler-service-repository structure:

- **Handlers** process HTTP requests, enforce permissions, and return HTML or JSON responses.
- **Services** validate input and apply business rules.
- **Repositories** isolate in-memory and Supabase persistence logic.
- **Tera templates** render server-side pages, while JavaScript adds interactive behaviour and calls protected APIs.
- **Supabase functions** handle operations such as queue assignment, consultation transitions, and billing updates in a single database transaction.

Firebase manages user identity. Supabase stores the application’s relational data. A small Firestore staff profile maps each Firebase user to the role used by backend authorization checks.

A normal protected request follows this path:

```text
Browser request
    -> Actix route and handler
    -> Firebase session and role check
    -> Service validation and business rules
    -> Repository or Supabase RPC
    -> Supabase PostgreSQL
    -> HTML page or JSON response
```

This separation keeps HTTP handling, business rules, and database code independent. It also allows service logic to be tested with in-memory repositories without changing production Supabase code.

## Repository Structure

```text
PMS-1106/
|-- backend/
|   |-- Cargo.toml
|   |-- Cargo.lock
|   |-- supabase_schema.sql
|   |-- tests/
|   |   `-- external_services.rs
|   |-- src/
|   |   |-- main.rs
|   |   |-- routes.rs
|   |   |-- db.rs
|   |   |-- firebase_admin.rs
|   |   |-- admindashboard/
|   |   |-- appointments/
|   |   |-- billing/
|   |   |-- doctor_dashboard/
|   |   |-- doctors/
|   |   |-- handlers/
|   |   |-- medical_records/
|   |   |-- prescriptions/
|   |   |-- queue/
|   |   `-- staff/
|   `-- templates/
|       |-- billing/
|       |-- doctors/
|       |-- medical_records/
|       `-- prescriptions/
|-- frontend/
|   `-- assets/
|       |-- css/
|       `-- js/
|-- .gitignore
`-- README.md
```

## Setup Instructions

Follow these steps from the repository root.

### 1. Install the prerequisites

- Rust and Cargo
- A Supabase project
- A Firebase project with Email/Password Authentication enabled
- A Firebase service account with access to Firebase Authentication and Firestore

Confirm that Rust is installed:

```powershell
rustc --version
cargo --version
```

### 2. Configure Firebase

1. Create or select a Firebase project.
2. Enable Email/Password under **Authentication > Sign-in method**.
3. Create a web application and copy its project ID and web API key.
4. Create a service account with Firebase Authentication and Firestore access.
5. Store the service-account project ID, client email, and private key in the backend environment file described below.

### 3. Configure the backend environment

Copy the example environment file:

```powershell
Copy-Item backend/.env.example backend/.env
```

Configure these values in `backend/.env`:

```dotenv
FIREBASE_PROJECT_ID=your-firebase-project-id
FIREBASE_API_KEY=your-firebase-web-api-key
FIREBASE_ADMIN_PROJECT_ID=your-firebase-project-id
FIREBASE_ADMIN_CLIENT_EMAIL=your-service-account-email
FIREBASE_ADMIN_PRIVATE_KEY="-----BEGIN PRIVATE KEY-----\n...\n-----END PRIVATE KEY-----\n"

SUPABASE_URL=https://your-project.supabase.co
SUPABASE_KEY=your-supabase-service-role-or-secret-key

BILLING_STORAGE=supabase
DOCTOR_STORAGE=supabase
DOCTOR_DASHBOARD_STORAGE=supabase
PRESCRIPTION_STORAGE=supabase
STAFF_STORAGE=supabase
MEDICAL_RECORDS_STORAGE=supabase
```

The Supabase key and Firebase private key are server credentials. Never place them in frontend JavaScript or commit `backend/.env` to Git.

### 4. Create the Supabase database

Open the Supabase SQL Editor and run the complete script:

```text
backend/supabase_schema.sql
```

The script creates:

- patients, staff, doctors, and doctor schedules
- appointments, patient queues, vitals, and room status
- medical records, prescriptions, prescription items, and medicine inventory
- invoices, invoice items, and payments
- indexes, validation constraints, timestamp triggers, and transactional RPC functions

The schema enables row-level security on application tables. The Rust backend uses a server-side Supabase secret/service key and performs role checks before accessing protected data.

### 5. Build and test the backend

```powershell
cd backend
cargo test
```

Return to the repository root if you want to continue following the commands exactly:

```powershell
cd ..
```

### 6. Run the application

From the backend directory:

```powershell
cd backend
cargo run
```

Open [http://127.0.0.1:8080](http://127.0.0.1:8080). The root route redirects to the login page.

### 7. Confirm the setup

1. Sign in with an existing Firebase user whose UID has a matching Firestore staff profile.
2. Confirm that the displayed name and role are correct.
3. Open a permitted module from the navigation bar.
4. If the user is redirected to login, check the Firebase credentials, staff profile, and backend console output.

## User Roles

| Role | Main responsibilities |
| --- | --- |
| Administrator | Staff accounts, doctor profiles, system-wide patient access, and administration |
| Receptionist | Patient registration, appointments, check-in, vital signs, queues, and billing |
| Doctor | Assigned consultations, patient records, and prescriptions |
| Pharmacist | Prescription viewing, dispensing, and medicine-related workflows |

Permissions are enforced by the Rust backend. Hiding a frontend button is only a user-interface convenience and is not treated as authorization.

## Main Routes

### Pages

| Method | Route | Purpose |
| --- | --- | --- |
| `GET` | `/login` | Login page |
| `GET` | `/dashboard` | Reception and administration dashboard |
| `GET` | `/patients` | Patient management |
| `GET` | `/appointments` | Appointment management |
| `GET` | `/staff` | Staff and doctor management |
| `GET` | `/doctor-dashboard` | Doctor consultation dashboard |
| `GET` | `/medical-records` | Medical record management |
| `GET` | `/prescriptions` | Prescription management |
| `GET` | `/billing` | Billing dashboard |

### Core APIs

| Area | Routes |
| --- | --- |
| Session | `POST /session`, `GET /logout`, `POST /forgot-password`, `GET /api/me` |
| Patients | `GET /api/patients`, `POST /api/patients/new`, `GET/PUT/DELETE /api/patients/{id}` |
| Appointments | `GET/POST /api/appointments`, `GET/PUT/DELETE /api/appointments/{id}` |
| Staff | `GET/POST /api/staff`, `PUT/DELETE /api/staff/{staff_id}` |
| Doctors | `GET/POST /api/doctors`, `GET/PUT/DELETE /api/doctors/{doctor_id}` |
| Medical records | `GET/POST /api/medical-records`, `GET/PUT/DELETE /api/medical-records/{record_id}` |
| Prescriptions | `GET/POST /api/prescriptions` and protected prescription action routes |
| Dashboard workflow | `/api/dashboard/appointments`, `/mark-arrived`, `/save-vitals`, `/send-to-room`, `/rooms` |
| Doctor workflow | `/api/doctor-dashboard/appointments`, `/start-consultation`, `/complete-consultation` |

### Billing and Reports

```text
GET  /billing
POST /billing/invoices
GET  /billing/invoices/{invoice_id}
POST /billing/invoices/{invoice_id}/payments
POST /billing/invoices/{invoice_id}/cancel
GET  /billing/invoices/{invoice_id}/report
GET  /billing/invoices/{invoice_id}/report.pdf
```

## Testing and Code Quality

Run the backend tests from `backend/`:

```powershell
cargo test
```

Run strict Rust linting:

```powershell
cargo clippy --all-targets -- -D warnings
```

Run the read-only checks against the configured Supabase and Firebase projects:

```powershell
cargo test --test external_services -- --ignored --test-threads=1
```

The normal test suite does not contact external services. The ignored integration tests require valid credentials and internet access.

Current automated coverage includes:

- role and permission rules
- patient input validation
- appointment overlap detection
- doctor and staff service behaviour
- queue priority validation
- medical record and prescription service rules
- invoice totals, payments, cancellation, and duplicate prevention
- concurrent invoice creation protection
- Tera template parsing
- live Supabase schema and Firebase project configuration checks

## Project Walkthrough

The following walkthrough demonstrates the full flow and shows how the modules share real database records:

1. **Sign in:** Log in with a role-specific Firebase account and confirm that restricted navigation items follow the user’s permissions.
2. **Register a patient:** Open **Patients**, create a patient profile, and confirm that it appears in the Supabase-backed patient table.
3. **Book an appointment:** Open **Appointments**, select the patient and doctor, then choose a date and time. Attempting an overlapping booking should be rejected.
4. **Check in the patient:** On the reception dashboard, mark the scheduled appointment as arrived. The system creates a queue entry and assigns the next daily queue number.
5. **Record triage information:** Enter vital signs and select a queue priority. Urgent and emergency priorities require a reason.
6. **Assign a room:** Send the patient to the assigned doctor’s room. The appointment, queue, and room status are updated together.
7. **Complete the consultation:** On the doctor dashboard, start the consultation, add a medical record and prescription, and then complete the consultation to release the room.
8. **Create an invoice:** Open **Billing**, select the patient, and add treatment and prescription charges. Totals are calculated and validated by the backend.
9. **Record payment:** Add a partial or full payment and confirm that the outstanding balance and invoice status update correctly.
10. **Generate reports:** Open the printable medical report and download the backend-generated PDF version.

This workflow demonstrates authentication, authorization, cross-module data relationships, validation, transactional database functions, and report generation.

## Team Responsibilities

- **Patient and authentication:** patient CRUD, Firebase sessions, validation, RBAC, and shared database setup.
- **Appointments and queues:** scheduling, overlap detection, check-in, triage, and queue progression.
- **Medical records:** record CRUD, consultation data, patient timelines, and protected clinical access.
- **Billing and reports:** invoices, payments, billing rules, printable reports, and PDF generation.
- **Doctors and prescriptions:** doctor profiles, schedules, availability, prescriptions, and medicine workflows.
- **Frontend integration:** shared layouts, responsive styling, dashboards, forms, tables, and backend API integration.

## Development Notes

- `backend/target/` is generated by Cargo and must not be committed.
- `Cargo.lock` should be committed because this repository contains an application.
- `.env` files and service-account credentials must never be committed.
- Apply database changes through the consolidated `backend/supabase_schema.sql` script.
