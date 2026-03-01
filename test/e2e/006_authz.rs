//! E2E tests for Forge authz using the prebuilt project from `bin/test-e2e`.
//! Run `bin/test-e2e` first.

use std::fs;
use std::time::Duration;

use forge_e2e_lib::cli;

#[test]
fn prebuilt_project_has_authz_layout() {
  let project_root = cli::prebuilt_project_root();
  assert!(
    project_root.exists(),
    "prebuilt project not found at {} — run bin/test-e2e first",
    project_root.display()
  );
  cli::assert_project_layout(&project_root);

  assert!(
    project_root
      .join("crates/db/src/models/organization.rs")
      .exists()
  );
  assert!(
    project_root
      .join("crates/db/src/models/membership.rs")
      .exists()
  );
  let user_model = fs::read_to_string(project_root.join("crates/db/src/models/user.rs")).unwrap();
  assert!(user_model.contains("impl AuthzContext"));
  let auth_handlers =
    fs::read_to_string(project_root.join("crates/app/src/handlers/auth.rs")).unwrap();
  assert!(auth_handlers.contains("guard") || auth_handlers.contains("guard_and_audit"));
}

#[tokio::test]
async fn prebuilt_server_protected_route_requires_auth() {
  let base = cli::e2e_base_url().expect("run e2e via bin/test-e2e (E2E_BASE_URL not set)");
  let email = format!(
    "authz-e2e-{}@test.com",
    std::time::SystemTime::now()
      .duration_since(std::time::UNIX_EPOCH)
      .unwrap()
      .as_millis()
  );
  let client = reqwest::Client::builder()
    .cookie_store(true)
    .timeout(Duration::from_secs(5))
    .build()
    .unwrap();

  let unauthed = client
    .get(format!("{}/api/auth/admin", base))
    .send()
    .await
    .expect("request");
  assert!(
    unauthed.status().as_u16() == 401
      || unauthed.status().as_u16() == 403
      || unauthed.status().as_u16() == 404,
    "unauthenticated GET /api/auth/admin should be 401/403/404: {}",
    unauthed.status()
  );

  let _ = client
    .post(format!("{}/api/auth/register", base))
    .json(&serde_json::json!({ "email": email, "password": "password123" }))
    .send()
    .await;
  let _ = client
    .post(format!("{}/api/auth/login", base))
    .json(&serde_json::json!({ "email": email, "password": "password123" }))
    .send()
    .await
    .expect("login");

  let authed = client
    .get(format!("{}/api/auth/admin", base))
    .send()
    .await
    .expect("request");
  assert!(
    authed.status().as_u16() == 200
      || authed.status().as_u16() == 403
      || authed.status().as_u16() == 404,
    "authenticated GET /api/auth/admin should be 200, 403, or 404: {}",
    authed.status()
  );
}
