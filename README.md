# Patient Management System Backend

## Overview

This project is a Patient Management System built with Rust, Actix Web, and server-side rendered HTML pages.

The system is designed as an enterprise-style web application for managing patient registration, appointments, medical records, billing, doctors, prescriptions, authentication, and reports.

The backend follows a layered architecture using Rust structs, traits, modules, and service logic. Pages are rendered on the server using Tera templates.

## Tech Stack

- Rust
- Actix Web
- Tera template engine
- Relational database schema using SQL
- HTML and CSS for SSR pages

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

The current backend includes an initial implementation of the Member 4 billing module.

Implemented so far:

- invoice creation
- invoice listing
- invoice detail page
- partial and full payment recording with payment history
- outstanding balance tracking
- invoice cancellation with guarded status transitions
- duplicate active-invoice prevention per appointment
- print-friendly medical report page
- billing input validation
- concurrent duplicate-invoice protection
- in-memory repository for temporary testing
- PostgreSQL/Supabase schema for invoices, invoice items, and payments

Billing supports both in-memory storage for local testing and persistent Supabase
PostgreSQL storage. Set `BILLING_STORAGE=supabase` after applying the billing
schema to enable persistence.

## Project Structure

```text
backend/
  Cargo.toml
  Cargo.lock
  README.md
  schema.sql
  src/
    main.rs
    routes.rs
    billing/
      handlers.rs
      models.rs
      repository.rs
      service.rs
  templates/
    layout.html
    error.html
    billing/
      index.html
      show.html
      report.html
  static/
    css/
      style.css
```

## How to Run

From the `backend` folder, run:

```bash
cargo run
```

Then open:

```text
http://127.0.0.1:8080/billing
```

## Billing Routes

```text
GET  /billing
POST /billing/invoices
GET  /billing/invoices/{invoice_id}
POST /billing/invoices/{invoice_id}/payments
POST /billing/invoices/{invoice_id}/cancel
GET  /billing/invoices/{invoice_id}/report
```

## Database Schema

The SQL schema is stored in:

```text
schema.sql
```

It currently includes tables for:

- invoices
- invoice items
- payments

More tables should be added as the other members implement their modules.

Run `supabase_schema.sql` before `schema.sql`, because billing invoices reference
the patient table. Then set the following value in `backend/.env`:

```text
BILLING_STORAGE=supabase
```

The Supabase repository uses transactional PostgreSQL functions for invoice
creation, payments, cancellation, and duplicate-appointment protection.

## Tests

Run the billing business-logic tests from the `backend` folder:

```bash
cargo test billing::service::tests
```

The tests cover totals, validation, partial and full payments, cancellation,
duplicate appointment billing, and concurrent invoice creation.

## Development Notes

- `backend/target/` is generated by Cargo and should not be committed.
- `Cargo.lock` should be committed because this is an application project.
- Server logs are ignored by Git.
- Environment files and local database files should not be committed.
