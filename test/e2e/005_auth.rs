//! E2E tests for Forge auth using the prebuilt project from `bin/test-e2e`.
//! Run `bin/test-e2e` first. 100% fantoccini: register/login via forms, verify dashboard when authed.

use std::fs;

use forge_e2e_lib::cli;

/// Single E2E test: prebuilt auth layout; then browser register → login → dashboard (no reqwest).
#[tokio::test]
async fn e2e_prebuilt_auth_layout_and_flow() {
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

  let base = cli::e2e_base_url().expect("run e2e via bin/test-e2e (E2E_BASE_URL not set)");
  if std::env::var("E2E_WEBDRIVER_URL").is_err() {
    return;
  }
  let email = format!(
    "auth-e2e-{}@test.com",
    std::time::SystemTime::now()
      .duration_since(std::time::UNIX_EPOCH)
      .unwrap()
      .as_millis()
  );
  let password = "password123";

  let c = forge_e2e_lib::browser::connect()
    .await
    .expect("browser connect (run chromedriver or set E2E_WEBDRIVER_URL)");
  forge_e2e_lib::browser::register_via_browser(&c, &base, &email, password)
    .await
    .expect("register via browser");
  forge_e2e_lib::browser::login_via_browser(&c, &base, &email, password)
    .await
    .expect("login via browser");
  forge_e2e_lib::browser::assert_dashboard_visible(&c, &base)
    .await
    .expect("dashboard visible after login");
  c.close().await.expect("close browser");
}
