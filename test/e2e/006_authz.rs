//! E2E tests for Forge authz using the prebuilt project from `bin/test-e2e`.
//! Run `bin/test-e2e` first. 100% fantoccini: unauthed /dashboard redirect, then login and verify dashboard.

use std::fs;

use forge_e2e_lib::cli;

/// Single E2E test: prebuilt authz layout; then browser: /dashboard unauthed → redirect to login; register+login → dashboard loads.
#[tokio::test]
async fn e2e_prebuilt_authz_layout_and_protected_route() {
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
  assert!(
    auth_handlers.contains("admin")
      && (auth_handlers.contains("is_admin") || auth_handlers.contains("record_authz_denied")),
    "auth handlers should gate admin or use record_authz_denied"
  );

  let base = cli::e2e_base_url().expect("run e2e via bin/test-e2e (E2E_BASE_URL not set)");
  if std::env::var("E2E_WEBDRIVER_URL").is_err() {
    return;
  }
  let email = format!(
    "authz-e2e-{}@test.com",
    std::time::SystemTime::now()
      .duration_since(std::time::UNIX_EPOCH)
      .unwrap()
      .as_millis()
  );
  let password = "password";

  let c = forge_e2e_lib::browser::connect()
    .await
    .expect("browser connect (run chromedriver or set E2E_WEBDRIVER_URL)");
  forge_e2e_lib::browser::assert_dashboard_redirects_to_login(&c, &base)
    .await
    .expect("unauthed /dashboard must redirect to login");
  forge_e2e_lib::browser::register(&c, &base, &email, password)
    .await
    .expect("register via browser");
  forge_e2e_lib::browser::login(&c, &base, &email, password)
    .await
    .expect("login via browser");
  forge_e2e_lib::browser::assert_dashboard_visible(&c, &base)
    .await
    .expect("dashboard visible after login");
  c.close().await.expect("close browser");
}
