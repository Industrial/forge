//! E2E tests for Forge auth using the prebuilt project from `bin/test-e2e`.
//! Run `bin/test-e2e` first.

use std::fs;
use std::time::Duration;

use forge_e2e_lib::cli;

#[test]
fn prebuilt_project_has_auth_layout() {
  let project_root = cli::prebuilt_project_root();
  assert!(
    project_root.exists(),
    "prebuilt project not found at {} — run bin/test-e2e first",
    project_root.display()
  );
  cli::assert_project_layout(&project_root);

  assert!(
    project_root.join("crates/db/src/auth.rs").exists(),
    "crates/db/src/auth.rs missing"
  );
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
  assert!(user_model.contains("current_org_id") && user_model.contains("current_role"));
  assert!(user_model.contains("impl AuthzContext"));

  let auth_handlers =
    fs::read_to_string(project_root.join("crates/app/src/handlers/auth.rs")).unwrap();
  assert!(
    auth_handlers.contains("guard_and_audit")
      || auth_handlers.contains("guard(Action::Manage, Role::Owner)")
  );

  let main_rs = fs::read_to_string(project_root.join("crates/app/src/main.rs")).unwrap();
  assert!(main_rs.contains("post_route") && main_rs.contains("/api/auth/admin"));
  assert!(!main_rs.contains("forge::prelude"));
}

#[tokio::test]
#[ignore = "SQLite readonly in e2e env (code 1032); see bd issue"]
async fn prebuilt_server_auth_flow_register_login_protected_route() {
  let base = cli::e2e_base_url().expect("run e2e via bin/test-e2e (E2E_BASE_URL not set)");
  let email = format!(
    "auth-e2e-{}@test.com",
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

  let reg = client
    .post(format!("{}/api/auth/register", base))
    .json(&serde_json::json!({ "email": email, "password": "password123" }))
    .send()
    .await
    .expect("register");
  let reg_status = reg.status();
  let reg_body = reg.text().await.unwrap_or_default();
  assert!(
    reg_status.is_success(),
    "register should succeed: {} {}",
    reg_status,
    reg_body
  );

  let login = client
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

  let admin = client
    .get(format!("{}/api/auth/admin", base))
    .send()
    .await
    .expect("admin");
  assert!(
    admin.status().as_u16() == 200,
    "GET /api/auth/admin with session should be 200: {}",
    admin.status()
  );
}
