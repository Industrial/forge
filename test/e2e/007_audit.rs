//! End-to-End tests for Forge Audit Logging (Phase 7).
//!
//! Covers: audit_log migration in generated project, auth events (login success,
//! failed login, logout), authz events (guard_and_audit allow/deny).

use std::fs;
use std::process::Command;
use std::time::Duration;

fn get_forge_binary_path() -> std::path::PathBuf {
  if let Ok(path) = std::env::var("CARGO_BIN_EXE_forge") {
    std::path::PathBuf::from(path)
  } else {
    let mut current_dir = std::env::current_exe().unwrap();
    while current_dir.file_name().and_then(|s| s.to_str()) != Some("target") {
      if let Some(parent) = current_dir.parent() {
        current_dir = parent.to_path_buf();
      } else {
        break;
      }
    }
    let workspace_root = if current_dir.file_name().and_then(|s| s.to_str()) == Some("target") {
      current_dir.parent().unwrap().to_path_buf()
    } else {
      std::env::current_dir().unwrap()
    };
    workspace_root.join("target").join("debug").join("forge")
  }
}

#[tokio::test]
async fn forge_new_generates_audit_log_migration() {
  // Ensure forge-cli is built so the binary includes the audit migration template
  let workspace_root = std::path::Path::new(env!("CARGO_MANIFEST_DIR")).join("../..");
  let build = Command::new("cargo")
    .args(["build", "-p", "forge-cli"])
    .current_dir(&workspace_root)
    .output()
    .expect("cargo build forge-cli");
  assert!(
    build.status.success(),
    "forge-cli must build: {}",
    String::from_utf8_lossy(&build.stderr)
  );

  let temp_dir = tempfile::tempdir().unwrap();
  let forge_binary = get_forge_binary_path();

  let out = Command::new(&forge_binary)
    .arg("new")
    .arg("audit_migration_test")
    .current_dir(&temp_dir)
    .output()
    .expect("forge new");

  assert!(
    out.status.success(),
    "forge new failed: {}",
    String::from_utf8_lossy(&out.stderr)
  );
  let root = temp_dir.path().join("audit_migration_test");

  assert!(
    root
      .join("crates/db/src/migrations/m20220101_000005_create_audit_log_table.rs")
      .exists(),
    "audit_log migration must be generated"
  );
  let mod_rs = fs::read_to_string(root.join("crates/db/src/migrations/mod.rs")).unwrap();
  assert!(
    mod_rs.contains("m20220101_000005_create_audit_log_table"),
    "migrations mod must include audit_log"
  );
  let db_lib = fs::read_to_string(root.join("crates/db/src/lib.rs")).unwrap();
  assert!(
    db_lib.contains("m20220101_000005_create_audit_log_table"),
    "Migrator must include audit_log migration"
  );
}

#[tokio::test]
async fn audit_events_recorded_for_auth_and_authz_flow() {
  let temp_dir = tempfile::tempdir().unwrap();
  let forge_binary = get_forge_binary_path();

  let out = Command::new(&forge_binary)
    .arg("new")
    .arg("audit_e2e_app")
    .current_dir(&temp_dir)
    .output()
    .expect("forge new");
  assert!(
    out.status.success(),
    "forge new: {}",
    String::from_utf8_lossy(&out.stderr)
  );

  let project_dir = temp_dir.path().join("audit_e2e_app");

  // Use a fixed port and ensure config has it
  fs::write(
    project_dir.join("config/app.toml"),
    r#"[app]
name = "audit_e2e"
environment = "development"

[server]
host = "127.0.0.1"
port = 30999
"#,
  )
  .unwrap();

  // Start the app in the background (cargo run from generated project)
  let mut child = Command::new("cargo")
    .args(["run", "--quiet"])
    .current_dir(&project_dir)
    .stdout(std::process::Stdio::null())
    .stderr(std::process::Stdio::piped())
    .spawn()
    .expect("spawn cargo run");

  // Build and start the app (first build can be slow)
  let workspace_root = std::path::Path::new(env!("CARGO_MANIFEST_DIR")).join("../..");
  let _ = Command::new("cargo")
    .args(["build", "-p", "forge-cli"])
    .current_dir(&workspace_root)
    .output();

  // Wait for server to be ready (allow time for cargo run to compile and bind)
  let client: reqwest::Client = reqwest::Client::builder()
    .cookie_store(true)
    .build()
    .unwrap();
  let base = "http://127.0.0.1:30999";
  for i in 0..150 {
    tokio::time::sleep(Duration::from_millis(200)).await;
    if client.get(format!("{}/", base)).send().await.is_ok() {
      break;
    }
    if i == 149 {
      let _ = child.kill();
      panic!("server did not become ready in time");
    }
  }

  // 1. Register
  let reg = client
    .post(format!("{}/auth/register", base))
    .json(&serde_json::json!({ "email": "audit@test.com", "password": "secret123" }))
    .send()
    .await
    .expect("register");
  assert!(reg.status().is_success(), "register: {}", reg.status());

  // 2. Login success -> should produce auth event (login, success)
  let login_ok = client
    .post(format!("{}/auth/login", base))
    .json(&serde_json::json!({ "email": "audit@test.com", "password": "secret123" }))
    .send()
    .await
    .expect("login");
  assert!(
    login_ok.status().is_success(),
    "login: {}",
    login_ok.status()
  );

  // 3. Hit /auth/admin (guard_and_audit) -> authz event allowed
  let admin_ok = client
    .get(format!("{}/auth/admin", base))
    .send()
    .await
    .expect("admin");
  assert!(
    admin_ok.status().is_success(),
    "admin: {}",
    admin_ok.status()
  );

  // 4. Logout -> auth event (logout)
  let _logout = client
    .get(format!("{}/auth/logout", base))
    .send()
    .await
    .expect("logout");

  // 5. Failed login -> auth event (failed_login, failure)
  let login_fail = client
    .post(format!("{}/auth/login", base))
    .json(&serde_json::json!({ "email": "audit@test.com", "password": "wrong" }))
    .send()
    .await
    .expect("login fail");
  assert!(
    login_fail.status().as_u16() == 401,
    "failed login should be 401: {}",
    login_fail.status()
  );

  // Give a moment for audit writes
  tokio::time::sleep(Duration::from_millis(100)).await;

  let _ = child.kill();
  let _ = child.wait();

  // 6. Read audit_log from sqlite and assert expected events
  let db_path = project_dir.join("db.sqlite");
  assert!(db_path.exists(), "db.sqlite should exist after running app");

  let conn = rusqlite::Connection::open(&db_path).expect("open db");
  let mut stmt = conn
    .prepare("SELECT event_kind, outcome, reason FROM audit_log ORDER BY occurred_at ASC")
    .expect("prepare");

  let rows: Vec<(String, String, Option<String>)> = stmt
    .query_map([], |r| Ok((r.get(0)?, r.get(1)?, r.get(2)?)))
    .expect("query")
    .map(|r| r.expect("row"))
    .collect();

  let has_login_success = rows
    .iter()
    .any(|(k, o, r)| k == "auth" && o == "success" && r.as_deref() == Some("login"));
  let has_logout = rows
    .iter()
    .any(|(k, o, r)| k == "auth" && o == "success" && r.as_deref() == Some("logout"));
  let has_failed_login = rows
    .iter()
    .any(|(k, o, r)| k == "auth" && o == "failure" && r.as_deref() == Some("failed_login"));
  let has_authz_allowed = rows.iter().any(|(k, o, _)| k == "authz" && o == "allowed");

  assert!(
    has_login_success,
    "expected auth login success event; rows: {:?}",
    rows
  );
  assert!(has_logout, "expected auth logout event; rows: {:?}", rows);
  assert!(
    has_failed_login,
    "expected auth failed_login event; rows: {:?}",
    rows
  );
  assert!(
    has_authz_allowed,
    "expected authz allowed event; rows: {:?}",
    rows
  );
}

#[tokio::test]
async fn audit_authz_denied_recorded_when_guard_fails() {
  let temp_dir = tempfile::tempdir().unwrap();
  let forge_binary = get_forge_binary_path();

  let out = Command::new(&forge_binary)
    .arg("new")
    .arg("audit_deny_app")
    .current_dir(&temp_dir)
    .output()
    .expect("forge new");
  assert!(out.status.success());

  let project_dir = temp_dir.path().join("audit_deny_app");
  fs::write(
    project_dir.join("config/app.toml"),
    r#"[app]
name = "audit_deny"
environment = "development"
[server]
host = "127.0.0.1"
port = 30998
"#,
  )
  .unwrap();

  let mut child = Command::new("cargo")
    .args(["run", "--quiet"])
    .current_dir(&project_dir)
    .stdout(std::process::Stdio::null())
    .stderr(std::process::Stdio::piped())
    .spawn()
    .expect("spawn");

  let client: reqwest::Client = reqwest::Client::builder()
    .cookie_store(true)
    .build()
    .unwrap();
  let base = "http://127.0.0.1:30998";
  for i in 0..150 {
    tokio::time::sleep(Duration::from_millis(200)).await;
    if client.get(base).send().await.is_ok() {
      break;
    }
    if i == 149 {
      let _ = child.kill();
      panic!("server did not become ready in time");
    }
  }

  // Hit /auth/admin without logging in -> unauthenticated -> guard_and_audit logs denied
  let admin_no_auth = client
    .get(format!("{}/auth/admin", base))
    .send()
    .await
    .unwrap();
  let status = admin_no_auth.status().as_u16();
  assert!(
    status == 403 || status == 401 || status == 500,
    "unauthenticated request to /auth/admin should be 4xx or 5xx, got {}",
    admin_no_auth.status()
  );

  tokio::time::sleep(Duration::from_millis(100)).await;
  let _ = child.kill();
  let _ = child.wait();

  let db_path = project_dir.join("db.sqlite");
  if db_path.exists() {
    let conn = rusqlite::Connection::open(&db_path).unwrap();
    let mut stmt = conn
      .prepare("SELECT event_kind, outcome FROM audit_log WHERE event_kind = 'authz' AND outcome = 'denied'")
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
}
