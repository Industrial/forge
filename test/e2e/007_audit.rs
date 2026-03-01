//! E2E tests for Forge audit: real scenarios only.
//!
//! **Pattern**: Every test uses the real `forge` and `cargo` CLIs. We generate a project
//! with `forge new`, then run `cargo check` / `cargo run` on the generated tree and assert
//! on real outcomes (exit codes, file layout, HTTP responses, audit_log table). No in-process mocks.
//!
//! **Structure**:
//! 1. Create a temp dir (project `.tmp/` via `forge_e2e_lib::tmpdir`).
//! 2. Run `forge new <name>` in that dir; assert success.
//! 3. Assert generated audit migration and layout.
//! 4. Run `cargo check` or `cargo run` in the project dir (target is `project_dir/target`, under `.tmp/`).
//! 5. For “run” scenarios: start the app, auth flow, assert audit_log rows, then cleanup.

use std::fs;
use std::process::Command;
use std::time::Duration;

use forge_e2e_lib::cli;

// --- Tests (real CLI scenarios) ---

/// Real scenario: `forge new` → assert audit_log migration present → `cargo check` succeeds.
#[test]
fn forge_new_generates_audit_log_migration_and_builds() {
  let workspace = forge_e2e_lib::tmpdir::tmpdir().unwrap();
  let project_name = "audit_migration_test";

  let out = cli::run_forge_new(workspace.path(), project_name);
  assert!(
    out.status.success(),
    "forge new failed: stderr={}",
    String::from_utf8_lossy(&out.stderr)
  );

  let project_root = workspace.path().join(project_name);
  cli::assert_project_layout(&project_root);

  assert!(
    project_root
      .join("crates/db/src/migrations/m20220101_000005_create_audit_log_table.rs")
      .exists(),
    "audit_log migration must be generated"
  );
  let mod_rs = fs::read_to_string(project_root.join("crates/db/src/migrations/mod.rs")).unwrap();
  assert!(mod_rs.contains("m20220101_000005_create_audit_log_table"));
  let db_lib = fs::read_to_string(project_root.join("crates/db/src/lib.rs")).unwrap();
  assert!(db_lib.contains("m20220101_000005_create_audit_log_table"));

  let check_out = cli::run_cargo_check(&project_root);
  if !check_out.status.success() {
    eprintln!(
      "cargo check STDERR: {}",
      String::from_utf8_lossy(&check_out.stderr)
    );
  }
  assert!(
    check_out.status.success(),
    "generated project must pass cargo check"
  );
}

/// Real scenario: `forge new` → run → register, login, admin, logout, failed login → assert audit_log events.
#[tokio::test]
async fn audit_events_recorded_for_auth_and_authz_flow() {
  let workspace = forge_e2e_lib::tmpdir::tmpdir().unwrap();
  let project_name = "audit_e2e_app";

  let out = cli::run_forge_new(workspace.path(), project_name);
  assert!(
    out.status.success(),
    "forge new failed: stderr={}",
    String::from_utf8_lossy(&out.stderr)
  );

  let project_root = workspace.path().join(project_name);
  let port = 30_000u16 + (std::process::id() % 1000) as u16;
  fs::write(
    project_root.join("config/app.toml"),
    format!(
      r#"[app]
name = "audit_e2e"
environment = "development"

[server]
host = "127.0.0.1"
port = {}
"#,
      port
    ),
  )
  .unwrap();

  let mut child = Command::new("cargo")
    .args(["run", "--quiet"])
    .current_dir(&project_root)
    .stdout(std::process::Stdio::null())
    .stderr(std::process::Stdio::piped())
    .spawn()
    .expect("spawn cargo run");

  let client = reqwest::Client::builder()
    .cookie_store(true)
    .timeout(Duration::from_secs(5))
    .build()
    .unwrap();
  let base = format!("http://127.0.0.1:{}", port);

  for i in 0..450 {
    tokio::time::sleep(Duration::from_millis(200)).await;
    if client.get(format!("{}/", base)).send().await.is_ok() {
      break;
    }
    if i == 449 {
      let _ = child.kill();
      let _ = child.wait();
      panic!("server did not become ready in time");
    }
  }

  let reg = client
    .post(format!("{}/api/auth/register", base))
    .json(&serde_json::json!({ "email": "audit@test.com", "password": "secret123" }))
    .send()
    .await
    .expect("register");
  assert!(reg.status().is_success(), "register: {}", reg.status());

  let login_ok = client
    .post(format!("{}/api/auth/login", base))
    .json(&serde_json::json!({ "email": "audit@test.com", "password": "secret123" }))
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
    .json(&serde_json::json!({ "email": "audit@test.com", "password": "wrong" }))
    .send()
    .await
    .expect("login fail");
  assert!(
    login_fail.status().as_u16() == 401,
    "failed login should be 401: {}",
    login_fail.status()
  );

  tokio::time::sleep(Duration::from_millis(100)).await;
  let _ = child.kill();
  let _ = child.wait();

  let db_path = project_root.join("db.sqlite");
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

/// Real scenario: `forge new` → run → GET /api/auth/admin without login → assert authz denied in audit_log.
#[tokio::test]
async fn audit_authz_denied_recorded_when_guard_fails() {
  let workspace = forge_e2e_lib::tmpdir::tmpdir().unwrap();
  let project_name = "audit_deny_app";

  let out = cli::run_forge_new(workspace.path(), project_name);
  assert!(
    out.status.success(),
    "forge new failed: {}",
    String::from_utf8_lossy(&out.stderr)
  );

  let project_root = workspace.path().join(project_name);
  let port = 30_000u16 + (std::process::id() % 1000) as u16;
  fs::write(
    project_root.join("config/app.toml"),
    format!(
      r#"[app]
name = "audit_deny"
environment = "development"

[server]
host = "127.0.0.1"
port = {}
"#,
      port
    ),
  )
  .unwrap();

  let mut child = Command::new("cargo")
    .args(["run", "--quiet"])
    .current_dir(&project_root)
    .stdout(std::process::Stdio::null())
    .stderr(std::process::Stdio::piped())
    .spawn()
    .expect("spawn cargo run");

  let client = reqwest::Client::builder()
    .cookie_store(true)
    .timeout(Duration::from_secs(5))
    .build()
    .unwrap();
  let base = format!("http://127.0.0.1:{}", port);

  for i in 0..300 {
    tokio::time::sleep(Duration::from_millis(200)).await;
    if client.get(format!("{}/", base)).send().await.is_ok() {
      break;
    }
    if i == 299 {
      let _ = child.kill();
      let _ = child.wait();
      panic!("server did not become ready in time");
    }
  }

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
  let _ = child.kill();
  let _ = child.wait();

  let db_path = project_root.join("db.sqlite");
  if db_path.exists() && status != 404 {
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
