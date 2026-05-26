use actix_web::{web, HttpRequest, HttpResponse, Responder};
use actix_web::cookie::{Cookie, time::Duration};
use serde::Deserialize;
use tera::{Context, Tera};
use firebase_auth::FirebaseAuth;
use crate::db::FirebaseRestDb;

#[derive(Deserialize)]
pub struct SessionForm {
    pub id_token: String,       
    pub remember: Option<String>,
}

#[derive(Deserialize)]
pub struct ForgotPasswordForm {
    pub email: String,
}

#[derive(Deserialize)]
pub struct FirebaseClaims {
    pub sub: String,
}

// ── Route config ──────────────────────────────────────────────────────────────

pub fn routes(cfg: &mut web::ServiceConfig) {
    // FLAT ROUTING
    cfg.route("/login",           web::get().to(login_page));
    cfg.route("/dashboard",       web::get().to(dashboard_page));
    cfg.route("/session",         web::post().to(create_session));
    cfg.route("/logout",          web::get().to(logout));
    cfg.route("/forgot-password", web::post().to(forgot_password));
}

// ── GET /login ────────────────────────────────────────────────────────────────

pub async fn login_page(req: HttpRequest, tera: web::Data<Tera>) -> impl Responder {
    if is_authenticated(&req) {
        return HttpResponse::Found().append_header(("Location", "/dashboard")).finish();
    }
    let mut ctx = Context::new();
    ctx.insert("firebase_api_key", &std::env::var("FIREBASE_API_KEY").unwrap_or_default());
    ctx.insert("firebase_project_id", &std::env::var("FIREBASE_PROJECT_ID").unwrap_or_default());
    if let Some(query) = req.uri().query() {
        for pair in query.split('&') {
            let mut kv = pair.splitn(2, '=');
            if kv.next().unwrap_or("") == "error" {
                ctx.insert("flash_error", flash_message(kv.next().unwrap_or("")));
            }
        }
    }
    render_login(&tera, ctx)
}

// ── GET /dashboard ────────────────────────────────────────────────────────────

pub async fn dashboard_page(
    req: HttpRequest,
    tera: web::Data<Tera>,
    firebase_auth: web::Data<FirebaseAuth>,
) -> impl Responder {
    let _uid = match require_auth(&req, &firebase_auth).await {
        Ok(uid) => uid,
        Err(redirect) => return redirect, 
    };
    let mut ctx = Context::new();
    ctx.insert("firebase_api_key", &std::env::var("FIREBASE_API_KEY").unwrap_or_default());
    ctx.insert("firebase_project_id", &std::env::var("FIREBASE_PROJECT_ID").unwrap_or_default());

    match tera.render("dashboard.html", &ctx) {
        Ok(html) => HttpResponse::Ok().content_type("text/html; charset=utf-8").body(html),
        Err(e) => {
            eprintln!("Template error: {e}");
            HttpResponse::InternalServerError().body("Template rendering failed")
        }
    }
}

// ── POST /session ─────────────────────────────────────────────────────────────

pub async fn create_session(
    form: web::Form<SessionForm>,
    firebase_auth: web::Data<FirebaseAuth>, 
    firestore_db: web::Data<FirebaseRestDb>,
) -> impl Responder {
    match firebase_auth.verify::<FirebaseClaims>(&form.id_token) {
        Ok(claims) => {
            let uid = claims.sub;
            let mut user_role = String::from("Unauthorized");

            if let Ok(json_str) = firestore_db.get_document("staff", &uid).await {
                if let Ok(parsed) = serde_json::from_str::<serde_json::Value>(&json_str) {
                    if let Some(role) = parsed["fields"]["role"]["stringValue"].as_str() {
                        user_role = role.to_string();
                    }
                }
            }
            if user_role == "Unauthorized" {
                eprintln!("Rejected: UID {} has no valid staff profile!", uid);
                return HttpResponse::Forbidden().body("Access Denied: No staff profile found.");
            }
            println!("Login Success! UID: {} | Role: {}", uid, user_role);

            let remember = form.remember.as_deref() == Some("true");
            let max_age = if remember { Duration::days(30) } else { Duration::hours(1) };
            let auth_cookie = Cookie::build("firebase_token", &form.id_token)
                .path("/").http_only(true).same_site(actix_web::cookie::SameSite::Lax).max_age(max_age).finish();
            let role_cookie = Cookie::build("user_role", &user_role)
                .path("/").same_site(actix_web::cookie::SameSite::Lax).max_age(max_age).finish();

            HttpResponse::Ok().cookie(auth_cookie).cookie(role_cookie).finish()
        }
        Err(e) => HttpResponse::Unauthorized().body("Invalid login credentials")
    }
}

// ── GET /logout ───────────────────────────────────────────────────────────────

pub async fn logout() -> impl Responder {
    let cookie = Cookie::build("firebase_token", "").path("/").http_only(true).max_age(Duration::seconds(0)).finish();
    HttpResponse::Found().cookie(cookie).append_header(("Location", "/login")).finish()
}

// ── POST /forgot-password ─────────────────────────────────────────────────────

pub async fn forgot_password(form: web::Form<ForgotPasswordForm>) -> impl Responder {
    if form.email.is_empty() { return HttpResponse::BadRequest().finish(); }
    let api_key = std::env::var("FIREBASE_API_KEY").unwrap_or_default();
    let url = format!("https://identitytoolkit.googleapis.com/v1/accounts:sendOobCode?key={api_key}");
    let body = serde_json::json!({ "requestType": "PASSWORD_RESET", "email": form.email });
    let client = reqwest::Client::new();
    match client.post(&url).json(&body).send().await {
        Ok(res) if res.status().is_success() => HttpResponse::Ok().finish(),
        _ => HttpResponse::Ok().finish() 
    }
}

// ── Middleware helper ─────────────────────────────────────────────────────────

pub async fn require_auth(req: &HttpRequest, firebase_auth: &FirebaseAuth) -> Result<String, HttpResponse> {
    let token = req.cookie("firebase_token").map(|c| c.value().to_string()).ok_or_else(|| {
        HttpResponse::Found().append_header(("Location", "/login?error=unauthorized")).finish()
    })?;
    match firebase_auth.verify::<FirebaseClaims>(&token) {
        Ok(claims) => Ok(claims.sub),
        Err(_) => Err(HttpResponse::Found().append_header(("Location", "/login?error=session_expired")).finish())
    }
}

// ── Helpers ───────────────────────────────────────────────────────────────────

fn render_login(tera: &Tera, ctx: Context) -> HttpResponse {
    match tera.render("index.html", &ctx) {
        Ok(html) => HttpResponse::Ok().content_type("text/html; charset=utf-8").body(html),
        Err(e) => HttpResponse::InternalServerError().body(format!("Template error: {e}"))
    }
}
fn is_authenticated(req: &HttpRequest) -> bool {
    req.cookie("firebase_token").map(|c| !c.value().is_empty()).unwrap_or(false)
}
fn flash_message(code: &str) -> &'static str {
    match code {
        "session_expired" => "Your session has expired. Please sign in again.",
        "unauthorized"    => "You must be signed in to access that page.",
        _                 => "An error occurred. Please try again.",
    }
}

// ── RBAC Permissions Engine ───────────────────────────────────────────────────

// 1. List every restricted action in your app here
pub enum AppAction {
    CreatePatient,
    EditPatient,
    DeletePatient,
    ManageAppointments,
    ViewMedicalRecords,
    EditMedicalRecords,
}

// 2. The central source of truth for who can do what
pub fn has_permission(role: &str, action: AppAction) -> bool {
    let normalized_role = role.to_lowercase();

    // Admins have god-mode. They automatically pass every check.
    if normalized_role == "admin" {
        return true;
    }

    // Define the exact limits for everyone else here using pattern matching
    match action {
        // Receptionists handle intake
        AppAction::CreatePatient => matches!(normalized_role.as_str(), "receptionist"),
        AppAction::ManageAppointments => matches!(normalized_role.as_str(), "receptionist"),
        
        // Doctors handle medical data
        AppAction::ViewMedicalRecords => matches!(normalized_role.as_str(), "doctor" | "receptionist"),
        AppAction::EditMedicalRecords => matches!(normalized_role.as_str(), "doctor"),
        
        // Shared permissions
        AppAction::EditPatient => matches!(normalized_role.as_str(), "receptionist" | "doctor"),
        
        // Dangerous actions: 'false' means ONLY the admin can do this
        AppAction::DeletePatient => false, 
    }
}

// 3. The Actix Middleware Helper
pub fn require_permission(req: &HttpRequest, action: AppAction) -> Result<(), HttpResponse> {
    // Grab the role cookie we set during login
    let role = req
        .cookie("user_role")
        .map(|c| c.value().to_string())
        .unwrap_or_else(|| "Unauthorized".to_string());

    if has_permission(&role, action) {
        Ok(())
    } else {
        println!("🚨 SECURITY BLOCK: Role '{}' attempted unauthorized action!", role);
        Err(HttpResponse::Forbidden().body("Access Denied: You do not have permission to perform this action."))
    }
}