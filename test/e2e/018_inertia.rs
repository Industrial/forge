//! E2E test for Forge Inertia (018): Vite+React frontend; browser verifies routes and ws-demo.
//! Run via `bin/test-e2e`. 100% fantoccini: navigate /, /login, /register, /dashboard, /ws-demo; assert Inertia shell and redirect for dashboard.

use forge_e2e_lib::cli;

/// Single E2E test: prebuilt frontend layout; then browser hits /, /login, /register, /dashboard, /ws-demo and asserts.
#[tokio::test]
async fn e2e_prebuilt_inertia_shell_i18n_auth_ws() {
  let project_root = cli::prebuilt_project_root();
  assert!(
    project_root.exists(),
    "prebuilt project not found at {} — run bin/test-e2e first",
    project_root.display()
  );
  cli::assert_project_layout(&project_root);

  let frontend_pkg = project_root.join("frontend/package.json");
  let frontend_main = project_root.join("frontend/src/main.tsx");
  assert!(
    frontend_pkg.exists(),
    "018 frontend/package.json missing at {}",
    frontend_pkg.display()
  );
  assert!(
    frontend_main.exists(),
    "018 frontend/src/main.tsx missing at {}",
    frontend_main.display()
  );

  let base = cli::e2e_base_url().expect("run e2e via bin/test-e2e (E2E_BASE_URL not set)");
  if std::env::var("E2E_WEBDRIVER_URL").is_err() {
    return;
  }
  let b = base.trim_end_matches('/');

  forge_e2e_lib::browser::assert_app_root_loads_and_contains(b, "Pages/Home")
    .await
    .expect("GET / must load Inertia with Pages/Home");

  forge_e2e_lib::browser::goto_path_and_assert_app(b, "/login")
    .await
    .expect("GET /login must load Inertia shell");
  forge_e2e_lib::browser::goto_path_and_assert_app(b, "/register")
    .await
    .expect("GET /register must load Inertia shell");

  let c = forge_e2e_lib::browser::connect()
    .await
    .expect("browser connect (run chromedriver or set E2E_WEBDRIVER_URL)");
  forge_e2e_lib::browser::assert_dashboard_redirects_to_login(&c, &base)
    .await
    .expect("/dashboard unauthed must redirect to login");
  c.close().await.expect("close browser");

  forge_e2e_lib::browser::assert_ws_demo_echo(&base, "ping")
    .await
    .expect("ws-demo must show echo");
}
