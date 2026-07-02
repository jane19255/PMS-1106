fn load_env() {
    let _ = dotenv::from_filename(".env");
}

fn required_env(name: &str) -> String {
    std::env::var(name).unwrap_or_else(|_| panic!("{name} must be set in backend/.env"))
}

fn supabase_get(
    client: &reqwest::Client,
    url: String,
    key: &str,
) -> reqwest::RequestBuilder {
    let request = client.get(url).header("apikey", key);
    if key.starts_with("eyJ") {
        request.header("Authorization", format!("Bearer {key}"))
    } else {
        request
    }
}

#[actix_web::test]
#[ignore = "requires live Supabase credentials"]
async fn live_supabase_exposes_required_tables_and_functions() {
    load_env();
    let base_url = required_env("SUPABASE_URL");
    let key = required_env("SUPABASE_KEY");
    let client = reqwest::Client::new();

    // PostgREST exposes its current schema as an OpenAPI document.
    let response = supabase_get(
        &client,
        format!("{}/rest/v1/", base_url.trim_end_matches('/')),
        &key,
    )
    .send()
    .await
    .expect("Supabase should be reachable");

    assert!(
        response.status().is_success(),
        "Supabase schema request returned {}",
        response.status()
    );
    let schema = response.text().await.expect("schema response should be text");

    for item in [
        "patients",
        "appointments",
        "patient_queue",
        "medical_records",
        "invoices",
        "/rpc/enqueue_patient",
        "/rpc/save_patient_vitals",
        "/rpc/send_patient_to_room",
        "/rpc/start_consultation",
        "/rpc/complete_consultation",
        "/rpc/billing_create_invoice",
    ] {
        assert!(schema.contains(item), "Supabase schema is missing {item}");
    }
}

#[actix_web::test]
#[ignore = "requires live Firebase configuration"]
async fn live_firebase_auth_configuration_is_reachable() {
    load_env();
    let api_key = required_env("FIREBASE_API_KEY");
    let expected_project_id = required_env("FIREBASE_PROJECT_ID");

    // This reads public Auth project configuration and does not create a user.
    let response = reqwest::Client::new()
        .get("https://www.googleapis.com/identitytoolkit/v3/relyingparty/getProjectConfig")
        .query(&[("key", api_key)])
        .send()
        .await
        .expect("Firebase should be reachable");

    assert!(
        response.status().is_success(),
        "Firebase configuration request returned {}",
        response.status()
    );
    let config: serde_json::Value = response
        .json()
        .await
        .expect("Firebase configuration should be valid JSON");

    assert!(config.is_object(), "Firebase configuration should be an object");
    let authorized_domains = config
        .get("authorizedDomains")
        .and_then(|value| value.as_array())
        .expect("Firebase configuration should list authorized domains");
    let firebase_domain = format!("{expected_project_id}.firebaseapp.com");
    assert!(
        authorized_domains
            .iter()
            .filter_map(|value| value.as_str())
            .any(|domain| domain == firebase_domain),
        "FIREBASE_API_KEY does not belong to FIREBASE_PROJECT_ID"
    );
}
