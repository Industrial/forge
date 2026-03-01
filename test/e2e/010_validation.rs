//! E2E tests for Forge validation using the prebuilt project from `bin/test-e2e`.
//! Run `bin/test-e2e` first.

use std::fs;
use std::time::Duration;

use forge_e2e_lib::cli;

#[test]
fn prebuilt_project_has_auth_routes_in_main() {
  let project_root = cli::prebuilt_project_root();
  assert!(
    project_root.exists(),
    "prebuilt project not found at {} — run bin/test-e2e first",
    project_root.display()
  );
  cli::assert_project_layout(&project_root);

  let main_rs = fs::read_to_string(project_root.join("crates/app/src/main.rs")).unwrap();
  assert!(
    main_rs.contains("/api/auth/register"),
    "generated main.rs must contain /api/auth/register"
  );
}

#[tokio::test]
async fn prebuilt_server_register_and_login_validation_422_for_invalid() {
  let base = cli::e2e_base_url().expect("run e2e via bin/test-e2e (E2E_BASE_URL not set)");
  let client = reqwest::Client::builder()
    .timeout(Duration::from_secs(5))
    .build()
    .unwrap();

  let reg_invalid = client
    .post(format!("{}/api/auth/register", base))
    .json(&serde_json::json!({ "email": "not-an-email", "password": "short" }))
    .send()
    .await
    .expect("register");
  assert_eq!(
    reg_invalid.status().as_u16(),
    422,
    "invalid register should return 422: {}",
    reg_invalid.status()
  );
  let body = reg_invalid.text().await.unwrap();
  assert!(
    body.contains("errors") || body.contains("email") || body.contains("password"),
    "422 body should contain error details: {}",
    body
  );

  let unique = std::time::SystemTime::now()
    .duration_since(std::time::UNIX_EPOCH)
    .unwrap()
    .as_millis();
  let reg_ok = client
    .post(format!("{}/api/auth/register", base))
    .json(&serde_json::json!({ "email": format!("valid-{}@example.com", unique), "password": "password123" }))
    .send()
    .await
    .expect("register");
  assert!(
    reg_ok.status().is_success(),
    "valid register should succeed: {}",
    reg_ok.status()
  );

  let login_invalid = client
    .post(format!("{}/api/auth/login", base))
    .json(&serde_json::json!({ "email": "bad-email", "password": "any" }))
    .send()
    .await
    .expect("login");
  assert_eq!(
    login_invalid.status().as_u16(),
    422,
    "invalid login should return 422: {}",
    login_invalid.status()
  );
}
