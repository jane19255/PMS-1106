use crate::admindashboard::models::{
    MarkArrivedPayload,
    SaveVitalPayload,
    SendToRoomPayload,
};
use crate::admindashboard::repository;
use crate::admindashboard::service;
use crate::db::{singapore_today, FirebaseRestDb, SupabaseRestDb};
use crate::auth::{require_auth_and_permission, AppAction};
use actix_web::{web, HttpRequest, HttpResponse, Responder};
use chrono::NaiveDate;
use firebase_auth::FirebaseAuth;
use tera::Tera;

pub fn routes(cfg: &mut web::ServiceConfig) {
    cfg.route("/dashboard", web::get().to(dashboard_html));
    cfg.route("/api/dashboard/appointments", web::get().to(api_calendar_appointments));
    cfg.route("/api/dashboard/mark-arrived", web::post().to(api_mark_arrived));
    cfg.route("/api/dashboard/save-vitals", web::post().to(api_save_vitals));
    cfg.route("/api/dashboard/send-to-room", web::post().to(api_send_to_room));
    cfg.route("/api/dashboard/rooms", web::get().to(api_room_statuses));
}

pub async fn dashboard_html(
    req: HttpRequest,
    tera: web::Data<Tera>,
    firebase_auth: web::Data<FirebaseAuth>,
    firestore_db: web::Data<FirebaseRestDb>,
) -> impl Responder {
    if let Err(redirect) = require_auth_and_permission(
        &req,
        &firebase_auth,
        &firestore_db,
        AppAction::ManageAppointments
    ).await {
        return redirect;
    }

    let ctx = tera::Context::new();

    match tera.render("dashboard.html", &ctx) {
        Ok(html) => HttpResponse::Ok()
            .content_type("text/html")
            .body(html),

        Err(e) => {
            println!("Tera error = {:?}", e);
            HttpResponse::InternalServerError()
                .body(format!("Template error: {}", e))
        }
    }
}

pub async fn api_calendar_appointments(
    req: HttpRequest,
    firebase_auth: web::Data<FirebaseAuth>,
    firestore_db: web::Data<FirebaseRestDb>,
    supabase_db: web::Data<SupabaseRestDb>,
    date_query: web::Query<std::collections::HashMap<String, String>>,
) -> impl Responder {
    if let Err(res) = require_auth_and_permission(
        &req,
        &firebase_auth,
        &firestore_db,
        AppAction::ManageAppointments,
    )
    .await
    {
        return res;
    }

    let today = singapore_today();
    let default_date = today.format("%Y-%m-%d").to_string();

    let date_str = date_query
        .get("date")
        .unwrap_or(&default_date);

    let target_date = match NaiveDate::parse_from_str(
        date_str,
        "%Y-%m-%d",
    ) {
        Ok(d) => d,
        Err(_) => today,
    };

    if target_date == today {
        // Reconcile first so yesterday's scheduled bookings do not stay open forever.
        if let Err(message) = supabase_db.reconcile_overdue_appointments(today).await {
            return HttpResponse::InternalServerError().body(message);
        }
    }

    match repository::get_appointments_by_date(&supabase_db, target_date, today).await {
        Ok(data) => HttpResponse::Ok()
            .content_type("application/json")
            .body(data),
        Err(message) => HttpResponse::InternalServerError().body(message),
    }
}

pub async fn api_mark_arrived(
    req: HttpRequest,
    firebase_auth: web::Data<FirebaseAuth>,
    firestore_db: web::Data<FirebaseRestDb>,
    supabase_db: web::Data<SupabaseRestDb>,
    payload: web::Json<MarkArrivedPayload>,
) -> impl Responder {
    if let Err(res) = require_auth_and_permission(&req, &firebase_auth, &firestore_db, AppAction::ManageAppointments).await {
        return res;
    }
    let today = singapore_today();
    match service::mark_patient_arrived(&supabase_db, payload.into_inner(), today).await {
        Ok(json_data) => HttpResponse::Ok().json(json_data),
        Err(msg) => HttpResponse::BadRequest().body(msg),
    }
}

pub async fn api_save_vitals(
    req: HttpRequest,
    firebase_auth: web::Data<FirebaseAuth>,
    firestore_db: web::Data<FirebaseRestDb>,
    supabase_db: web::Data<SupabaseRestDb>,
    payload: web::Json<SaveVitalPayload>,
) -> impl Responder {
    if let Err(res) = require_auth_and_permission(&req, &firebase_auth, &firestore_db, AppAction::ManageAppointments).await {
        return res;
    }
    match service::save_patient_vitals(&supabase_db, payload.into_inner()).await {
        Ok(json_data) => HttpResponse::Ok().json(json_data),
        Err(msg) => HttpResponse::BadRequest().body(msg),
    }
}

pub async fn api_send_to_room(
    req: HttpRequest,
    firebase_auth: web::Data<FirebaseAuth>,
    firestore_db: web::Data<FirebaseRestDb>,
    supabase_db: web::Data<SupabaseRestDb>,
    payload: web::Json<SendToRoomPayload>,
) -> impl Responder {
    if let Err(res) = require_auth_and_permission(
        &req,
        &firebase_auth,
        &firestore_db,
        AppAction::ManageAppointments,
    )
    .await
    {
        return res;
    }

    let payload = payload.into_inner();

    match repository::send_to_room(
        &supabase_db,
        &payload.appointment_id,
        &payload.doctor_id,
    )
    .await
    {
        Ok(json_data) => HttpResponse::Ok()
            .content_type("application/json")
            .body(json_data),
        Err(msg) => HttpResponse::BadRequest().body(msg),
    }
}

pub async fn api_room_statuses(
    req: HttpRequest,
    firebase_auth: web::Data<FirebaseAuth>,
    firestore_db: web::Data<FirebaseRestDb>,
    supabase_db: web::Data<SupabaseRestDb>,
) -> impl Responder {
    if let Err(res) = require_auth_and_permission(
        &req,
        &firebase_auth,
        &firestore_db,
        AppAction::ManageAppointments,
    )
    .await
    {
        return res;
    }

    match repository::get_room_statuses(&supabase_db).await {
        Ok(data) => HttpResponse::Ok()
            .content_type("application/json")
            .body(data),
        Err(message) => HttpResponse::InternalServerError().body(message),
    }
}

