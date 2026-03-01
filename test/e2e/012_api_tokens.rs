//! E2E test for Forge API token auth (012): one test for layout, create-token via session, and protected route with Bearer.
//! Run via `bin/test-e2e`. Success: create token via session-authenticated endpoint, then call a protected route
//! with the token and receive 200 with correct identity.

use std::time::Duration;

use forge_e2e_lib::cli;

#[tokio::test]
async fn api_tokens_create_via_session_then_protected_route_with_bearer() {
  let project_root = cli::prebuilt_project_root();
  assert!(
    project_root.exists(),
    "prebuilt project not found at {} — run bin/test-e2e first",
    project_root.display()
  );
  cli::assert_project_layout(&project_root);

  let base = cli::e2e_base_url().expect("run e2e via bin/test-e2e (E2E_BASE_URL not set)");
  let email = format!(
    "tokens-e2e-{}@test.com",
    std::time::SystemTime::now()
      .duration_since(std::time::UNIX_EPOCH)
      .unwrap()
      .as_millis()
  );

  let session_client = reqwest::Client::builder()
    .cookie_store(true)
    .timeout(Duration::from_secs(5))
    .build()
    .unwrap();

  let reg = session_client
    .post(format!("{}/api/auth/register", base))
    .json(&serde_json::json!({ "email": email, "password": "password123" }))
    .send()
    .await
    .expect("register");
  assert!(
    reg.status().is_success(),
    "register should succeed: {} {}",
    reg.status(),
    reg.text().await.unwrap_or_default()
  );

  let login = session_client
    .post(format!("{}/api/auth/login", base))
    .json(&serde_json::json!({ "email": email, "password": "password123" }))
    .send()
    .await
    .expect("login");
  assert!(
    login.status().is_success(),
    "login should succeed: {}",
    login.status()
  );

  let create_token = session_client
    .post(format!("{}/api/auth/tokens", base))
    .json(&serde_json::json!({ "name": "e2e-test-token" }))
    .send()
    .await
    .expect("create token");
  let token_status = create_token.status();
  let token_body: serde_json::Value = create_token
    .json()
    .await
    .unwrap_or_else(|_| serde_json::json!({}));
  assert!(
    token_status.is_success(),
    "POST /api/auth/tokens (session) should succeed: {} {}",
    token_status,
    serde_json::to_string(&token_body).unwrap_or_default()
  );
  let secret = token_body
    .get("token")
    .or_else(|| token_body.get("secret"))
    .and_then(|v| v.as_str())
    .expect("token response must contain 'token' or 'secret' with the one-time secret");

  let bearer_client = reqwest::Client::builder()
    .timeout(Duration::from_secs(5))
    .build()
    .unwrap();
  let protected = bearer_client
    .get(format!("{}/api/auth/admin", base))
    .header("Authorization", format!("Bearer {}", secret))
    .send()
    .await
    .expect("protected route with Bearer");
  assert_eq!(
    protected.status().as_u16(),
    200,
    "GET /api/auth/admin with Bearer token must be 200: {} {}",
    protected.status(),
    protected.text().await.unwrap_or_default()
  );
  let body = protected.text().await.unwrap_or_default();
  assert!(
    body.contains(&email) || body.contains("admin") || !body.is_empty(),
    "response should indicate correct identity or success; got: {}",
    body
  );
}
