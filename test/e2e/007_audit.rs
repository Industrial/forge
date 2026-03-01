//! E2E tests for Forge audit using the prebuilt project from `bin/test-e2e`.
//! Run `bin/test-e2e` first.

use std::fs;
use std::time::Duration;

use forge_e2e_lib::cli;

/// Single E2E test: audit migration layout, authz denied recorded, and auth flow events in audit_log.
/// 1. Asserts project layout and audit_log migration file and mod/lib references.
/// 2. Unauthed GET /api/auth/admin → 4xx/5xx/404; assert audit_log has authz denied.
/// 3. Register, login, admin, logout, failed login; assert audit_log has auth login/logout/failed_login and authz allowed.
#[tokio::test]
async fn e2e_prebuilt_audit_migration_and_events() {
  let project_root = cli::prebuilt_project_root();
  assert!(
    project_root.exists(),
    "prebuilt project not found at {} — run bin/test-e2e first",
    project_root.display()
  );
  cli::assert_project_layout(&project_root);

  let migration_path =
    project_root.join("crates/db/src/migrations/m20220101_000005_create_audit_log_table.rs");
  assert!(
    migration_path.exists(),
    "audit_log migration must be generated"
  );
  let mod_rs = fs::read_to_string(project_root.join("crates/db/src/migrations/mod.rs")).unwrap();
  assert!(mod_rs.contains("m20220101_000005_create_audit_log_table"));
  let db_lib = fs::read_to_string(project_root.join("crates/db/src/lib.rs")).unwrap();
  assert!(db_lib.contains("m20220101_000005_create_audit_log_table"));

  let base = cli::e2e_base_url().expect("run e2e via bin/test-e2e (E2E_BASE_URL not set)");
  let client = reqwest::Client::builder()
    .cookie_store(true)
    .timeout(Duration::from_secs(5))
    .build()
    .unwrap();

  let admin_no_auth = client
    .get(format!("{}/api/auth/admin", base))
    .send()
    .await
    .unwrap();
  let status = admin_no_auth.status().as_u16();
  assert!(
    status == 403 || status == 401 || status == 500 || status == 404,
    "unauthenticated GET /api/auth/admin should be 4xx or 5xx (or 404): {}",
    admin_no_auth.status()
  );

  tokio::time::sleep(Duration::from_millis(100)).await;

  let db_path = project_root.join("db.sqlite");
  if db_path.exists() && status != 404 {
    let conn = rusqlite::Connection::open(&db_path).unwrap();
    let mut stmt = conn
      .prepare(
        "SELECT event_kind, outcome FROM audit_log WHERE event_kind = 'authz' AND outcome = 'denied'",
      )
      .unwrap();
    let denied: Vec<(String, String)> = stmt
      .query_map([], |r| Ok((r.get(0)?, r.get(1)?)))
      .unwrap()
      .map(|r| r.unwrap())
      .collect();
    assert!(
      !denied.is_empty(),
      "expected at least one authz denied event"
    );
  }

  let email = format!(
    "audit-{}@test.com",
    std::time::SystemTime::now()
      .duration_since(std::time::UNIX_EPOCH)
      .unwrap()
      .as_millis()
  );

  let reg = client
    .post(format!("{}/api/auth/register", base))
    .json(&serde_json::json!({ "email": email, "password": "secret123" }))
    .send()
    .await
    .expect("register");
  assert!(reg.status().is_success(), "register: {}", reg.status());

  let login_ok = client
    .post(format!("{}/api/auth/login", base))
    .json(&serde_json::json!({ "email": email, "password": "secret123" }))
    .send()
    .await
    .expect("login");
  assert!(
    login_ok.status().is_success(),
    "login: {}",
    login_ok.status()
  );

  let admin_ok = client
    .get(format!("{}/api/auth/admin", base))
    .send()
    .await
    .expect("admin");
  assert!(
    admin_ok.status().is_success(),
    "admin: {}",
    admin_ok.status()
  );

  let _ = client
    .get(format!("{}/api/auth/logout", base))
    .send()
    .await
    .expect("logout");

  let login_fail = client
    .post(format!("{}/api/auth/login", base))
    .json(&serde_json::json!({ "email": email, "password": "wrong" }))
    .send()
    .await
    .expect("login fail");
  assert!(
    login_fail.status().as_u16() == 401,
    "failed login should be 401: {}",
    login_fail.status()
  );

  tokio::time::sleep(Duration::from_millis(100)).await;

  assert!(db_path.exists(), "db.sqlite should exist (shared server)");

  let conn = rusqlite::Connection::open(&db_path).expect("open db");
  let mut stmt = conn
    .prepare("SELECT event_kind, outcome, reason FROM audit_log ORDER BY occurred_at ASC")
    .expect("prepare");
  let rows: Vec<(String, String, Option<String>)> = stmt
    .query_map([], |r| Ok((r.get(0)?, r.get(1)?, r.get(2)?)))
    .expect("query")
    .map(|r| r.expect("row"))
    .collect();

  assert!(
    rows
      .iter()
      .any(|(k, o, r)| k == "auth" && o == "success" && r.as_deref() == Some("login")),
    "expected auth login success event; rows: {:?}",
    rows
  );
  assert!(
    rows
      .iter()
      .any(|(k, o, r)| k == "auth" && o == "success" && r.as_deref() == Some("logout")),
    "expected auth logout event; rows: {:?}",
    rows
  );
  assert!(
    rows
      .iter()
      .any(|(k, o, r)| k == "auth" && o == "failure" && r.as_deref() == Some("failed_login")),
    "expected auth failed_login event; rows: {:?}",
    rows
  );
  assert!(
    rows.iter().any(|(k, o, _)| k == "authz" && o == "allowed"),
    "expected authz allowed event; rows: {:?}",
    rows
  );
}
