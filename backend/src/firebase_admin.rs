//! Firebase Admin REST client helpers.
//!
//! The backend uses this module when staff management needs privileged Firebase
//! operations that are separate from normal request authentication.
use jsonwebtoken::{encode, Algorithm, EncodingKey, Header};
use reqwest::Client;
use serde::{Deserialize, Serialize};
use serde_json::json;
use std::time::{SystemTime, UNIX_EPOCH};

const TOKEN_URL: &str = "https://oauth2.googleapis.com/token";

#[derive(Clone)]
pub struct FirebaseAdmin {
    project_id: String,
    client_email: String,
    private_key: String,
    api_key: String,
    client: Client,
}

#[derive(Serialize)]
struct ServiceAccountClaims<'a> {
    iss: &'a str,
    scope: &'a str,
    aud: &'a str,
    iat: u64,
    exp: u64,
}

#[derive(Deserialize)]
struct AccessTokenResponse {
    access_token: String,
}

#[derive(Deserialize)]
struct CreateUserResponse {
    #[serde(rename = "localId")]
    local_id: String,
}

impl FirebaseAdmin {
    pub fn from_env() -> Result<Self, String> {
        Ok(Self {
            project_id: required_env("FIREBASE_ADMIN_PROJECT_ID")?,
            client_email: required_env("FIREBASE_ADMIN_CLIENT_EMAIL")?,
            private_key: required_env("FIREBASE_ADMIN_PRIVATE_KEY")?.replace("\\n", "\n"),
            api_key: required_env("FIREBASE_API_KEY")?,
            client: Client::new(),
        })
    }

    pub async fn create_user(&self, email: &str, display_name: &str) -> Result<String, String> {
        // Only the backend service account is allowed to create staff logins.
        let token = self.access_token().await?;
        let url = format!(
            "https://identitytoolkit.googleapis.com/v1/projects/{}/accounts?key={}",
            self.project_id, self.api_key
        );
        let response = self
            .client
            .post(url)
            .bearer_auth(token)
            .json(&json!({
                "email": email.trim(),
                "displayName": display_name.trim(),
                "emailVerified": false,
                "disabled": false
            }))
            .send()
            .await
            .map_err(|_| "Could not contact Firebase Authentication".to_string())?;
        if !response.status().is_success() {
            return Err(firebase_error(response).await);
        }
        response
            .json::<CreateUserResponse>()
            .await
            .map(|user| user.local_id)
            .map_err(|_| "Firebase returned an invalid user response".to_string())
    }

    pub async fn send_password_setup(&self, email: &str) -> Result<(), String> {
        let url = format!(
            "https://identitytoolkit.googleapis.com/v1/accounts:sendOobCode?key={}",
            self.api_key
        );
        let response = self
            .client
            .post(url)
            .json(&json!({ "requestType": "PASSWORD_RESET", "email": email.trim() }))
            .send()
            .await
            .map_err(|_| "Could not send the password setup email".to_string())?;
        if response.status().is_success() {
            Ok(())
        } else {
            Err(firebase_error(response).await)
        }
    }

    pub async fn delete_user(&self, uid: &str) -> Result<(), String> {
        let token = self.access_token().await?;
        let url = format!(
            "https://identitytoolkit.googleapis.com/v1/projects/{}/accounts:delete?key={}",
            self.project_id, self.api_key
        );
        let response = self
            .client
            .post(url)
            .bearer_auth(token)
            .json(&json!({ "localId": uid }))
            .send()
            .await
            .map_err(|_| "Could not contact Firebase Authentication".to_string())?;
        if response.status().is_success() {
            Ok(())
        } else {
            Err(firebase_error(response).await)
        }
    }

    // Keeps the Firebase Auth login email in sync with the staff record.
    // Without this, editing a staff member's email only changes Supabase —
    // the account still signs in with whatever email it was created under.
    pub async fn update_email(&self, uid: &str, new_email: &str) -> Result<(), String> {
        let token = self.access_token().await?;
        let url = format!(
            "https://identitytoolkit.googleapis.com/v1/accounts:update?key={}",
            self.api_key
        );
        let response = self
            .client
            .post(url)
            .bearer_auth(token)
            .json(&json!({ "localId": uid, "email": new_email.trim() }))
            .send()
            .await
            .map_err(|_| "Could not contact Firebase Authentication".to_string())?;
        if response.status().is_success() {
            Ok(())
        } else {
            Err(firebase_error(response).await)
        }
    }

    // Lets an admin set a user's password directly
    pub async fn set_password(&self, uid: &str, new_password: &str) -> Result<(), String> {
        let token = self.access_token().await?;
        let url = format!(
            "https://identitytoolkit.googleapis.com/v1/accounts:update?key={}",
            self.api_key
        );
        let response = self
            .client
            .post(url)
            .bearer_auth(token)
            .json(&json!({ "localId": uid, "password": new_password }))
            .send()
            .await
            .map_err(|_| "Could not contact Firebase Authentication".to_string())?;
        if response.status().is_success() {
            Ok(())
        } else {
            Err(firebase_error(response).await)
        }
    }

    // Used to offboard a doctor: the account is kept (for audit/history) but
    // can no longer sign in, unlike delete_user which removes it entirely.
    pub async fn disable_user(&self, uid: &str) -> Result<(), String> {
        let token = self.access_token().await?;
        let url = format!(
            "https://identitytoolkit.googleapis.com/v1/accounts:update?key={}",
            self.api_key
        );
        let response = self
            .client
            .post(url)
            .bearer_auth(token)
            .json(&json!({ "localId": uid, "disableUser": true }))
            .send()
            .await
            .map_err(|_| "Could not contact Firebase Authentication".to_string())?;
        if response.status().is_success() {
            Ok(())
        } else {
            Err(firebase_error(response).await)
        }
    }

    pub async fn save_staff_profile(
        &self,
        uid: &str,
        first_name: &str,
        last_name: &str,
        role: &str,
        status: &str,
    ) -> Result<(), String> {
        // Firestore stores the small profile used during login and permission checks.
        let token = self.access_token().await?;
        let url = format!(
            "https://firestore.googleapis.com/v1/projects/{}/databases/(default)/documents/staff/{}",
            self.project_id,
            urlencoding::encode(uid)
        );
        // Inactive staff keep their account but are blocked by the normal role check.
        let login_role = if status.eq_ignore_ascii_case("active") {
            role.trim().to_lowercase()
        } else {
            "unauthorized".to_string()
        };
        let response = self
            .client
            .patch(url)
            .bearer_auth(token)
            .json(&json!({
                "fields": {
                    "firstName": { "stringValue": first_name.trim() },
                    "lastName": { "stringValue": last_name.trim() },
                    "role": { "stringValue": login_role }
                }
            }))
            .send()
            .await
            .map_err(|_| "Could not update the Firebase staff profile".to_string())?;
        if response.status().is_success() {
            Ok(())
        } else {
            Err("Firebase staff profile could not be saved".to_string())
        }
    }

    pub async fn delete_staff_profile(&self, uid: &str) -> Result<(), String> {
        let token = self.access_token().await?;
        let url = format!(
            "https://firestore.googleapis.com/v1/projects/{}/databases/(default)/documents/staff/{}",
            self.project_id,
            urlencoding::encode(uid)
        );
        let response = self
            .client
            .delete(url)
            .bearer_auth(token)
            .send()
            .await
            .map_err(|_| "Could not delete the Firebase staff profile".to_string())?;
        if response.status().is_success() || response.status() == reqwest::StatusCode::NOT_FOUND {
            Ok(())
        } else {
            Err("Firebase staff profile could not be deleted".to_string())
        }
    }

    async fn access_token(&self) -> Result<String, String> {
        // The signed JWT is exchanged for a short-lived Google access token.
        let now = SystemTime::now()
            .duration_since(UNIX_EPOCH)
            .map_err(|_| "System time is invalid".to_string())?
            .as_secs();
        let claims = ServiceAccountClaims {
            iss: &self.client_email,
            scope: "https://www.googleapis.com/auth/identitytoolkit https://www.googleapis.com/auth/datastore",
            aud: TOKEN_URL,
            iat: now,
            exp: now + 3600,
        };
        let key = EncodingKey::from_rsa_pem(self.private_key.as_bytes())
            .map_err(|_| "Firebase Admin private key is invalid".to_string())?;
        let assertion = encode(&Header::new(Algorithm::RS256), &claims, &key)
            .map_err(|_| "Could not sign the Firebase Admin request".to_string())?;
        let response = self
            .client
            .post(TOKEN_URL)
            .form(&[
                ("grant_type", "urn:ietf:params:oauth:grant-type:jwt-bearer"),
                ("assertion", assertion.as_str()),
            ])
            .send()
            .await
            .map_err(|_| "Could not authenticate the Firebase service account".to_string())?;
        if !response.status().is_success() {
            return Err("Firebase service-account authentication failed".to_string());
        }
        response
            .json::<AccessTokenResponse>()
            .await
            .map(|token| token.access_token)
            .map_err(|_| "Firebase returned an invalid access token".to_string())
    }
}

fn required_env(name: &str) -> Result<String, String> {
    std::env::var(name)
        .ok()
        .filter(|value| !value.trim().is_empty())
        .ok_or_else(|| format!("{name} must be set"))
}

async fn firebase_error(response: reqwest::Response) -> String {
    let body = response.text().await.unwrap_or_default();
    if body.contains("EMAIL_EXISTS") {
        "A Firebase login already exists for this email".to_string()
    } else if body.contains("INVALID_EMAIL") {
        "Firebase rejected the email address".to_string()
    } else {
        "Firebase Authentication could not complete the request".to_string()
    }
}
