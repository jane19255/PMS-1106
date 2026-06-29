use actix_web::cookie::{time::Duration, Cookie};
use actix_web::{web, HttpRequest, HttpResponse, Responder};
use firebase_auth::FirebaseAuth;
use serde::Deserialize;
use tera::{Context, Tera};

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

#[derive(Clone, Copy)]
pub enum AppAction {
    ViewPatient,
    CreatePatient,
    EditPatient,
    DeletePatient,

    ManageAppointments,

    ViewMedicalRecords,
    EditMedicalRecords,

    ViewPrescriptions,
    CreatePrescription,
    DispenseMedicine,
    ManageMedicines,

    ManageUsers,

    ManageDoctorAppt,

    ViewBilling,
    CreateInvoice,
    RecordPayment,
    CancelInvoice,
    GenerateBillingReport,
}

pub fn routes(cfg: &mut web::ServiceConfig) {
    cfg.route("/login", web::get().to(login_page));
    cfg.route("/doctor-dashboard", web::get().to(doctorDashboard_page));
    cfg.route("/staff", web::get().to(staffs_page));
    cfg.route("/appointments", web::get().to(appointments_page));
    cfg.route("/records", web::get().to(records_page));
    cfg.route("/session", web::post().to(create_session));
    cfg.route("/logout", web::get().to(logout));
    cfg.route("/forgot-password", web::post().to(forgot_password));
    cfg.route("/api/me", web::get().to(current_user));
}

pub async fn login_page(
    req: HttpRequest,
    tera: web::Data<Tera>,
    firebase_auth: web::Data<FirebaseAuth>,
    firestore_db: web::Data<FirebaseRestDb>,
) -> impl Responder {
    if let Ok(uid) = require_auth(&req, &firebase_auth).await {
        if let Some(role) = get_staff_role(&firestore_db, &uid).await {
            return HttpResponse::Found()
                .append_header(("Location", default_landing_path(&role)))
                .finish();
        }
    }

    let mut ctx = Context::new();
    insert_firebase_config(&mut ctx);

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

fn default_landing_path(role: &str) -> &'static str {
    match role.trim().to_lowercase().as_str() {
        "doctor" => "/doctor-dashboard",
        "pharmacist" => "/prescriptions",
        _ => "/dashboard",
    }
}

pub async fn doctorDashboard_page(
    req: HttpRequest,
    tera: web::Data<Tera>,
    firebase_auth: web::Data<FirebaseAuth>,
    firestore_db: web::Data<FirebaseRestDb>,
) -> impl Responder {
    render_protected_page(
        req,
        tera,
        firebase_auth,
        Some(firestore_db),
        "Doctor-Dashboard.html",
        Some(AppAction::ManageDoctorAppt),
    )
    .await
}

pub async fn appointments_page(
    req: HttpRequest,
    tera: web::Data<Tera>,
    firebase_auth: web::Data<FirebaseAuth>,
    firestore_db: web::Data<FirebaseRestDb>,
) -> impl Responder {
    render_protected_page(
        req,
        tera,
        firebase_auth,
        Some(firestore_db),
        "Appointments.html",
        Some(AppAction::ManageAppointments),
    )
    .await
}

pub async fn records_page(
    req: HttpRequest,
    tera: web::Data<Tera>,
    firebase_auth: web::Data<FirebaseAuth>,
    firestore_db: web::Data<FirebaseRestDb>,
) -> impl Responder {
    render_protected_page(
        req,
        tera,
        firebase_auth,
        Some(firestore_db),
        "Medical-Records.html",
        Some(AppAction::ViewMedicalRecords),
    )
    .await
}

pub async fn staffs_page(
    req: HttpRequest,
    tera: web::Data<Tera>,
    firebase_auth: web::Data<FirebaseAuth>,
    firestore_db: web::Data<FirebaseRestDb>,
) -> impl Responder {
    render_protected_page(
        req,
        tera,
        firebase_auth,
        Some(firestore_db),
        "Staffs.html",
        Some(AppAction::ManageUsers),
    )
    .await
}

async fn render_protected_page(
    req: HttpRequest,
    tera: web::Data<Tera>,
    firebase_auth: web::Data<FirebaseAuth>,
    firestore_db: Option<web::Data<FirebaseRestDb>>,
    template_name: &str,
    required_action: Option<AppAction>,
) -> HttpResponse {
    match required_action {
        Some(action) => {
            let Some(firestore_db) = firestore_db else {
                return HttpResponse::InternalServerError().body("Role checker is not configured.");
            };

            if let Err(rejection) =
                require_auth_and_permission(&req, &firebase_auth, &firestore_db, action).await
            {
                return rejection;
            }
        }
        None => {
            if let Err(redirect) = require_auth(&req, &firebase_auth).await {
                return redirect;
            }
        }
    }

    render_template(&tera, template_name)
}

pub async fn create_session(
    form: web::Form<SessionForm>,
    firebase_auth: web::Data<FirebaseAuth>,
    firestore_db: web::Data<FirebaseRestDb>,
) -> impl Responder {
    match firebase_auth.verify::<FirebaseClaims>(&form.id_token) {
        Ok(claims) => {
            let uid = claims.sub;
            let user_role = match get_staff_role(&firestore_db, &uid).await {
                Some(role) => role,
                None => {
                    eprintln!("Rejected: UID {} has no active staff profile.", uid);
                    return HttpResponse::Forbidden()
                        .body("Access Denied: No active staff profile found.");
                }
            };

            println!("Login Success! UID: {} | Role: {}", uid, user_role);

            let remember = form.remember.as_deref() == Some("true");
            let max_age = if remember {
                Duration::days(30)
            } else {
                Duration::hours(1)
            };

            let auth_cookie = Cookie::build("firebase_token", &form.id_token)
                .path("/")
                .http_only(true)
                .same_site(actix_web::cookie::SameSite::Lax)
                .max_age(max_age)
                .finish();

            let role_cookie = Cookie::build("user_role", &user_role)
                .path("/")
                .http_only(true)
                .same_site(actix_web::cookie::SameSite::Lax)
                .max_age(max_age)
                .finish();

            HttpResponse::Ok()
                .cookie(auth_cookie)
                .cookie(role_cookie)
                .json(serde_json::json!({
                    "redirect_to": default_landing_path(&user_role)
                }))
        }
        Err(_) => HttpResponse::Unauthorized().body("Invalid login credentials"),
    }
}

pub async fn current_user(
    req: HttpRequest,
    firebase_auth: web::Data<FirebaseAuth>,
    firestore_db: web::Data<FirebaseRestDb>,
) -> impl Responder {
    let uid = match require_auth(&req, &firebase_auth).await {
        Ok(uid) => uid,
        Err(rejection) => return rejection,
    };

    let (name, role) = match get_staff_profile(&firestore_db, &uid).await {
        Some(profile) => profile,
        None => {
            return HttpResponse::Forbidden().body("Access Denied: No active staff profile found.")
        }
    };

    HttpResponse::Ok().json(serde_json::json!({
        "uid": uid,
        "name": name,
        "role": role,
        "permissions": {
            "canViewPatients": has_permission(&role, AppAction::ViewPatient),
            "canCreatePatients": has_permission(&role, AppAction::CreatePatient),
            "canEditPatients": has_permission(&role, AppAction::EditPatient),
            "canDeletePatients": has_permission(&role, AppAction::DeletePatient),
            "canManageAppointments": has_permission(&role, AppAction::ManageAppointments),
            "canManageDoctorAppointments": has_permission(&role, AppAction::ManageDoctorAppt),
            "canViewMedicalRecords": has_permission(&role, AppAction::ViewMedicalRecords),
            "canEditMedicalRecords": has_permission(&role, AppAction::EditMedicalRecords),
            "canViewPrescriptions": has_permission(&role, AppAction::ViewPrescriptions),
            "canCreatePrescriptions": has_permission(&role, AppAction::CreatePrescription),
            "canDispenseMedicine": has_permission(&role, AppAction::DispenseMedicine),
            "canManageMedicines": has_permission(&role, AppAction::ManageMedicines),
            "canManageUsers": has_permission(&role, AppAction::ManageUsers),
            "canViewBilling": has_permission(&role, AppAction::ViewBilling),
            "canGenerateBillingReport": has_permission(&role, AppAction::GenerateBillingReport)
        }
    }))
}

pub async fn logout() -> impl Responder {
    let auth_cookie = Cookie::build("firebase_token", "")
        .path("/")
        .http_only(true)
        .max_age(Duration::seconds(0))
        .finish();

    let role_cookie = Cookie::build("user_role", "")
        .path("/")
        .http_only(true)
        .max_age(Duration::seconds(0))
        .finish();

    HttpResponse::Found()
        .cookie(auth_cookie)
        .cookie(role_cookie)
        .append_header(("Location", "/login"))
        .finish()
}

pub async fn forgot_password(form: web::Form<ForgotPasswordForm>) -> impl Responder {
    if form.email.trim().is_empty() {
        return HttpResponse::BadRequest().finish();
    }

    let api_key = std::env::var("FIREBASE_API_KEY").unwrap_or_default();
    let url =
        format!("https://identitytoolkit.googleapis.com/v1/accounts:sendOobCode?key={api_key}");
    let body = serde_json::json!({ "requestType": "PASSWORD_RESET", "email": form.email });
    let client = reqwest::Client::new();

    match client.post(&url).json(&body).send().await {
        Ok(res) if res.status().is_success() => HttpResponse::Ok().finish(),
        _ => HttpResponse::Ok().finish(),
    }
}

pub async fn require_auth(
    req: &HttpRequest,
    firebase_auth: &FirebaseAuth,
) -> Result<String, HttpResponse> {
    let token = req
        .cookie("firebase_token")
        .map(|c| c.value().to_string())
        .ok_or_else(|| {
            HttpResponse::Found()
                .append_header(("Location", "/login?error=unauthorized"))
                .finish()
        })?;

    match firebase_auth.verify::<FirebaseClaims>(&token) {
        Ok(claims) => Ok(claims.sub),
        Err(_) => Err(HttpResponse::Found()
            .append_header(("Location", "/login?error=session_expired"))
            .finish()),
    }
}

pub async fn get_staff_profile(
    firestore_db: &FirebaseRestDb,
    uid: &str,
) -> Option<(String, String)> {
    let json_str = firestore_db.get_document("staff", uid).await.ok()?;
    let parsed = serde_json::from_str::<serde_json::Value>(&json_str).ok()?;

    let role = parsed["fields"]["role"]["stringValue"]
        .as_str()?
        .trim()
        .to_lowercase();

    let first_name = parsed["fields"]["firstName"]["stringValue"]
        .as_str()
        .unwrap_or("")
        .trim();

    let last_name = parsed["fields"]["lastName"]["stringValue"]
        .as_str()
        .unwrap_or("")
        .trim();

    let name = format!("{} {}", first_name, last_name).trim().to_string();

    if role.is_empty() || role == "unauthorized" {
        None
    } else {
        Some((name, role))
    }
}

pub async fn get_staff_role(firestore_db: &FirebaseRestDb, uid: &str) -> Option<String> {
    get_staff_profile(firestore_db, uid)
        .await
        .map(|(_, role)| role)
}

pub async fn require_auth_and_permission(
    req: &HttpRequest,
    firebase_auth: &FirebaseAuth,
    firestore_db: &FirebaseRestDb,
    action: AppAction,
) -> Result<(String, String), HttpResponse> {
    // Check both login and role here so protected routes follow the same rule.
    let uid = require_auth(req, firebase_auth).await?;
    let role = get_staff_role(firestore_db, &uid).await.ok_or_else(|| {
        HttpResponse::Forbidden().body("Access Denied: No active staff profile found.")
    })?;

    if has_permission(&role, action) {
        Ok((uid, role))
    } else {
        println!(
            "SECURITY BLOCK: Role '{}' attempted unauthorized action.",
            role
        );
        Err(HttpResponse::Forbidden()
            .body("Access Denied: You do not have permission to perform this action."))
    }
}

pub fn has_permission(role: &str, action: AppAction) -> bool {
    // Keep the role rules in one place so the frontend and API do not disagree.
    let normalized_role = role.trim().to_lowercase();

    if normalized_role == "admin" {
        return !matches!(action, AppAction::ManageDoctorAppt);
    }

    match normalized_role.as_str() {
        "doctor" => matches!(
            action,
            AppAction::ViewPatient
                | AppAction::ManageDoctorAppt
                | AppAction::ViewMedicalRecords
                | AppAction::EditMedicalRecords
                | AppAction::ViewPrescriptions
                | AppAction::CreatePrescription
        ),

        "receptionist" => matches!(
            action,
            AppAction::ViewPatient
                | AppAction::CreatePatient
                | AppAction::EditPatient
                | AppAction::ManageAppointments
                | AppAction::ViewBilling
                | AppAction::CreateInvoice
                | AppAction::RecordPayment
                | AppAction::CancelInvoice
                | AppAction::GenerateBillingReport
        ),

        "pharmacist" => matches!(
            action,
            AppAction::ViewPatient
                | AppAction::ViewMedicalRecords
                | AppAction::ViewPrescriptions
                | AppAction::DispenseMedicine
                | AppAction::ManageMedicines
                | AppAction::ViewBilling
        ),

        _ => false,
    }
}

fn render_login(tera: &Tera, ctx: Context) -> HttpResponse {
    match tera.render("index.html", &ctx) {
        Ok(html) => HttpResponse::Ok()
            .content_type("text/html; charset=utf-8")
            .body(html),
        Err(e) => HttpResponse::InternalServerError().body(format!("Template error: {e}")),
    }
}

fn render_template(tera: &Tera, template_name: &str) -> HttpResponse {
    let mut ctx = Context::new();
    insert_firebase_config(&mut ctx);

    match tera.render(template_name, &ctx) {
        Ok(html) => HttpResponse::Ok()
            .content_type("text/html; charset=utf-8")
            .body(html),
        Err(e) => {
            eprintln!("Template error while rendering {template_name}: {e}");
            HttpResponse::InternalServerError().body("Template rendering failed")
        }
    }
}

pub fn insert_firebase_config(ctx: &mut Context) {
    ctx.insert(
        "firebase_api_key",
        &std::env::var("FIREBASE_API_KEY").unwrap_or_default(),
    );
    ctx.insert(
        "firebase_project_id",
        &std::env::var("FIREBASE_PROJECT_ID").unwrap_or_default(),
    );
}

fn flash_message(code: &str) -> &'static str {
    match code {
        "session_expired" => "Your session has expired. Please sign in again.",
        "unauthorized" => "You must be signed in to access that page.",
        _ => "An error occurred. Please try again.",
    }
}

#[cfg(test)]
mod tests {
    use super::{has_permission, AppAction};

    #[test]
    fn admin_can_create_update_and_delete_patients() {
        assert!(has_permission("admin", AppAction::CreatePatient));
        assert!(has_permission("admin", AppAction::EditPatient));
        assert!(has_permission("admin", AppAction::DeletePatient));
    }

    #[test]
    fn receptionist_can_create_and_update_but_not_delete_patients() {
        assert!(has_permission("receptionist", AppAction::CreatePatient));
        assert!(has_permission("receptionist", AppAction::EditPatient));
        assert!(!has_permission("receptionist", AppAction::DeletePatient));
    }

    #[test]
    fn doctor_and_pharmacist_have_read_only_patient_access() {
        for role in ["doctor", "pharmacist"] {
            assert!(has_permission(role, AppAction::ViewPatient));
            assert!(!has_permission(role, AppAction::CreatePatient));
            assert!(!has_permission(role, AppAction::EditPatient));
            assert!(!has_permission(role, AppAction::DeletePatient));
        }
    }

    #[test]
    fn receptionist_can_manage_billing_workflow() {
        assert!(has_permission("receptionist", AppAction::ViewBilling));
        assert!(has_permission("receptionist", AppAction::CreateInvoice));
        assert!(has_permission("receptionist", AppAction::RecordPayment));
        assert!(has_permission("receptionist", AppAction::CancelInvoice));
        assert!(has_permission(
            "receptionist",
            AppAction::GenerateBillingReport
        ));
    }

    #[test]
    fn doctor_can_use_doctor_dashboard_but_not_manage_doctors() {
        assert!(has_permission("doctor", AppAction::ManageDoctorAppt));
        assert!(!has_permission("doctor", AppAction::ManageUsers));
    }

    #[test]
    fn receptionist_manages_appointments_but_not_doctor_dashboard_or_doctors() {
        assert!(has_permission(
            "receptionist",
            AppAction::ManageAppointments
        ));
        assert!(!has_permission("receptionist", AppAction::ManageDoctorAppt));
        assert!(!has_permission("receptionist", AppAction::ManageUsers));
    }

    #[test]
    fn doctor_do_not_have_any_billing_access() {
        assert!(!has_permission("doctor", AppAction::ViewBilling));
        assert!(!has_permission("doctor", AppAction::GenerateBillingReport));
        assert!(!has_permission("doctor", AppAction::CreateInvoice));
        assert!(!has_permission("doctor", AppAction::RecordPayment));
        assert!(!has_permission("doctor", AppAction::CancelInvoice));
    }

    #[test]
    fn prescription_permissions_match_clinical_roles() {
        assert!(has_permission("doctor", AppAction::ViewPrescriptions));
        assert!(has_permission("doctor", AppAction::CreatePrescription));
        assert!(!has_permission("doctor", AppAction::DispenseMedicine));

        assert!(has_permission("pharmacist", AppAction::ViewPrescriptions));
        assert!(has_permission("pharmacist", AppAction::DispenseMedicine));
        assert!(!has_permission("pharmacist", AppAction::CreatePrescription));

        assert!(!has_permission(
            "receptionist",
            AppAction::ViewPrescriptions
        ));
        assert!(!has_permission(
            "receptionist",
            AppAction::CreatePrescription
        ));
        assert!(!has_permission("receptionist", AppAction::DispenseMedicine));
    }

    #[test]
    fn pharmacist_cannot_change_billing_records() {
        assert!(has_permission("pharmacist", AppAction::ViewBilling));
        assert!(!has_permission("pharmacist", AppAction::CreateInvoice));
        assert!(!has_permission("pharmacist", AppAction::RecordPayment));
        assert!(!has_permission("pharmacist", AppAction::CancelInvoice));
    }

    #[test]
    fn admin_has_all_billing_permissions() {
        for action in [
            AppAction::ViewBilling,
            AppAction::CreateInvoice,
            AppAction::RecordPayment,
            AppAction::CancelInvoice,
            AppAction::GenerateBillingReport,
        ] {
            assert!(has_permission("admin", action));
        }
    }

    #[test]
    fn admin_cannot_access_doctor_dashboard() {
        assert!(!has_permission("admin", AppAction::ManageDoctorAppt));
    }
}
