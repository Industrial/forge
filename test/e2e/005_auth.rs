//! E2E tests for Forge auth using the prebuilt project from `bin/test-e2e`.
//! Run `bin/test-e2e` first. Tests use seed users (admin@admin.com, viewer@default.org, etc.);
//! harness enables auto_seed. Browser tests skip when E2E_WEBDRIVER_URL is unset.

use std::fs;
use std::time::Duration;

use forge_e2e_lib::cli;

const SEED_PASSWORD: &str = "password123";

/// Asserts prebuilt project has auth layout: user model with AuthzContext, auth handlers with admin gate, main with auth routes.
fn assert_auth_layout(project_root: &std::path::Path) {
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
    auth_handlers.contains("admin")
      && (auth_handlers.contains("is_admin") || auth_handlers.contains("record_authz_denied")),
    "auth handlers should gate admin on is_admin or record_authz_denied"
  );

  let main_rs = fs::read_to_string(project_root.join("crates/app/src/main.rs")).unwrap();
  assert!(main_rs.contains("post_route") && main_rs.contains("/api/auth/admin"));
  assert!(!main_rs.contains("forge::prelude"));
}

/// 1. Layout: project has auth, org, membership, user with AuthzContext, admin route.
#[tokio::test]
async fn e2e_auth_layout() {
  let project_root = cli::prebuilt_project_root();
  assert!(
    project_root.exists(),
    "prebuilt project not found at {} — run bin/test-e2e first",
    project_root.display()
  );
  cli::assert_project_layout(&project_root);
  assert_auth_layout(&project_root);
}

/// 2. Unauthed: visiting /dashboard redirects to login.
#[tokio::test]
async fn e2e_auth_unauthed_dashboard_redirects_to_login() {
  let project_root = cli::prebuilt_project_root();
  assert!(project_root.exists(), "run bin/test-e2e first");
  assert_auth_layout(&project_root);

  let base = cli::e2e_base_url().expect("run e2e via bin/test-e2e (E2E_BASE_URL not set)");
  if std::env::var("E2E_WEBDRIVER_URL").is_err() {
    return;
  }

  let timeout = Duration::from_secs(30);
  let result = tokio::time::timeout(timeout, async {
    let c = forge_e2e_lib::browser::connect().await?;
    forge_e2e_lib::browser::assert_dashboard_redirects_to_login(&c, &base).await?;
    c.close().await?;
    Ok::<(), Box<dyn std::error::Error + Send + Sync>>(())
  })
  .await;

  match result {
    Ok(Ok(())) => {}
    Ok(Err(e)) => panic!("unauthed dashboard redirect: {}", e),
    Err(_) => panic!("unauthed dashboard redirect timed out after {:?}", timeout),
  }
}

/// 3. Failed login: wrong password leaves user on login (no redirect to dashboard).
#[tokio::test]
async fn e2e_auth_login_fails_wrong_password() {
  let project_root = cli::prebuilt_project_root();
  assert!(project_root.exists(), "run bin/test-e2e first");
  assert_auth_layout(&project_root);

  let base = cli::e2e_base_url().expect("run e2e via bin/test-e2e (E2E_BASE_URL not set)");
  if std::env::var("E2E_WEBDRIVER_URL").is_err() {
    return;
  }

  let timeout = Duration::from_secs(30);
  let result = tokio::time::timeout(timeout, async {
    let c = forge_e2e_lib::browser::connect().await?;
    forge_e2e_lib::browser::login_fails(&c, &base, "admin@admin.com", "wrongpassword").await?;
    c.close().await?;
    Ok::<(), Box<dyn std::error::Error + Send + Sync>>(())
  })
  .await;

  match result {
    Ok(Ok(())) => {}
    Ok(Err(e)) => panic!("login_fails: {}", e),
    Err(_) => panic!("login_fails timed out after {:?}", timeout),
  }
}

/// 4. Login with seed user: admin@admin.com can log in and see dashboard.
#[tokio::test]
async fn e2e_auth_login_with_seed_user_then_dashboard() {
  let project_root = cli::prebuilt_project_root();
  assert!(project_root.exists(), "run bin/test-e2e first");
  assert_auth_layout(&project_root);

  let base = cli::e2e_base_url().expect("run e2e via bin/test-e2e (E2E_BASE_URL not set)");
  if std::env::var("E2E_WEBDRIVER_URL").is_err() {
    return;
  }

  let timeout = Duration::from_secs(30);
  let result = tokio::time::timeout(timeout, async {
    let c = forge_e2e_lib::browser::connect().await?;
    forge_e2e_lib::browser::login(&c, &base, "admin@admin.com", SEED_PASSWORD).await?;
    forge_e2e_lib::browser::assert_dashboard_visible(&c, &base).await?;
    c.close().await?;
    Ok::<(), Box<dyn std::error::Error + Send + Sync>>(())
  })
  .await;

  match result {
    Ok(Ok(())) => {}
    Ok(Err(e)) => panic!("login with seed user: {}", e),
    Err(_) => panic!("login with seed user timed out after {:?}", timeout),
  }
}

/// 5. Logout: after login, logout then /dashboard redirects to login.
#[tokio::test]
async fn e2e_auth_logout_then_dashboard_redirects_to_login() {
  let project_root = cli::prebuilt_project_root();
  assert!(project_root.exists(), "run bin/test-e2e first");
  assert_auth_layout(&project_root);

  let base = cli::e2e_base_url().expect("run e2e via bin/test-e2e (E2E_BASE_URL not set)");
  if std::env::var("E2E_WEBDRIVER_URL").is_err() {
    return;
  }

  let timeout = Duration::from_secs(45);
  let result = tokio::time::timeout(timeout, async {
    let c = forge_e2e_lib::browser::connect().await?;
    forge_e2e_lib::browser::login(&c, &base, "admin@admin.com", SEED_PASSWORD).await?;
    forge_e2e_lib::browser::logout(&c, &base).await?;
    forge_e2e_lib::browser::assert_dashboard_redirects_to_login(&c, &base).await?;
    c.close().await?;
    Ok::<(), Box<dyn std::error::Error + Send + Sync>>(())
  })
  .await;

  match result {
    Ok(Ok(())) => {}
    Ok(Err(e)) => panic!("logout then dashboard redirect: {}", e),
    Err(_) => panic!("logout flow timed out after {:?}", timeout),
  }
}

/// 6. Global admin: admin@admin.com can access /api/auth/admin.
#[tokio::test]
async fn e2e_auth_global_admin_can_access_admin_endpoint() {
  let project_root = cli::prebuilt_project_root();
  assert!(project_root.exists(), "run bin/test-e2e first");
  assert_auth_layout(&project_root);

  let base = cli::e2e_base_url().expect("run e2e via bin/test-e2e (E2E_BASE_URL not set)");
  if std::env::var("E2E_WEBDRIVER_URL").is_err() {
    return;
  }

  let timeout = Duration::from_secs(30);
  let result = tokio::time::timeout(timeout, async {
    let c = forge_e2e_lib::browser::connect().await?;
    forge_e2e_lib::browser::login(&c, &base, "admin@admin.com", SEED_PASSWORD).await?;
    forge_e2e_lib::browser::assert_admin_endpoint_granted(&c, &base).await?;
    c.close().await?;
    Ok::<(), Box<dyn std::error::Error + Send + Sync>>(())
  })
  .await;

  match result {
    Ok(Ok(())) => {}
    Ok(Err(e)) => panic!("global admin access: {}", e),
    Err(_) => panic!("global admin access timed out after {:?}", timeout),
  }
}

/// 7. Non-admin: viewer@default.org cannot access /api/auth/admin (Forbidden).
#[tokio::test]
async fn e2e_auth_non_admin_cannot_access_admin_endpoint() {
  let project_root = cli::prebuilt_project_root();
  assert!(project_root.exists(), "run bin/test-e2e first");
  assert_auth_layout(&project_root);

  let base = cli::e2e_base_url().expect("run e2e via bin/test-e2e (E2E_BASE_URL not set)");
  if std::env::var("E2E_WEBDRIVER_URL").is_err() {
    return;
  }

  let timeout = Duration::from_secs(30);
  let result = tokio::time::timeout(timeout, async {
    let c = forge_e2e_lib::browser::connect().await?;
    forge_e2e_lib::browser::login(&c, &base, "viewer@default.org", SEED_PASSWORD).await?;
    forge_e2e_lib::browser::assert_admin_endpoint_denied(&c, &base).await?;
    c.close().await?;
    Ok::<(), Box<dyn std::error::Error + Send + Sync>>(())
  })
  .await;

  match result {
    Ok(Ok(())) => {}
    Ok(Err(e)) => panic!("non-admin denied: {}", e),
    Err(_) => panic!("non-admin denied timed out after {:?}", timeout),
  }
}

/// 8. Full flow: register → login → dashboard (new user).
#[tokio::test]
async fn e2e_auth_register_login_dashboard_full_flow() {
  let project_root = cli::prebuilt_project_root();
  assert!(project_root.exists(), "run bin/test-e2e first");
  assert_auth_layout(&project_root);

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

  let timeout = Duration::from_secs(90);
  let result = tokio::time::timeout(timeout, async {
    let c = forge_e2e_lib::browser::connect().await?;
    forge_e2e_lib::browser::register(&c, &base, &email, password).await?;
    forge_e2e_lib::browser::login(&c, &base, &email, password).await?;
    forge_e2e_lib::browser::assert_dashboard_visible(&c, &base).await?;
    c.close().await?;
    Ok::<(), Box<dyn std::error::Error + Send + Sync>>(())
  })
  .await;

  match result {
    Ok(Ok(())) => {}
    Ok(Err(e)) => panic!("register/login/dashboard: {}", e),
    Err(_) => panic!(
      "register/login/dashboard timed out after {:?} — check server and chromedriver",
      timeout
    ),
  }
}
