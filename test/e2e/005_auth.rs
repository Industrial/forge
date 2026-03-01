//! E2E tests for Forge auth: real scenarios only.
//!
//! **Pattern**: Every test uses the real `forge` and `cargo` CLIs. We generate a project
//! with `forge new`, then run `cargo check` / `cargo run` on the generated tree and assert
//! on real outcomes (exit codes, file layout, HTTP responses). No in-process mocks.
//!
//! **Structure**:
//! 1. Create a temp dir (project `.tmp/` via `forge_e2e_lib::tmpdir`).
//! 2. Run `forge new <name>` in that dir; assert success.
//! 3. Assert generated auth-related files and content.
//! 4. Run `cargo check` or `cargo run` in the project dir (target is `project_dir/target`, under `.tmp/`).
//! 5. For “run” scenarios: start the app, register/login, hit protected endpoint, then cleanup.

use std::fs;
use std::process::Command;
use std::time::Duration;

use forge_e2e_lib::cli;

// --- Tests (real CLI scenarios) ---

/// Real scenario: `forge new` → assert auth workspace (auth.rs, org, membership, user, handlers) → `cargo check` succeeds.
#[test]
fn forge_new_generates_auth_workspace_and_builds() {
  let workspace = forge_e2e_lib::tmpdir::tmpdir().unwrap();
  let project_name = "auth_test_app";

  let out = cli::run_forge_new(workspace.path(), project_name);
  assert!(
    out.status.success(),
    "forge new failed: stderr={}",
    String::from_utf8_lossy(&out.stderr)
  );

  let project_root = workspace.path().join(project_name);
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

  let check_out = cli::run_cargo_check(&project_root);
  if !check_out.status.success() {
    eprintln!(
      "cargo check STDERR: {}",
      String::from_utf8_lossy(&check_out.stderr)
    );
  }
  assert!(
    check_out.status.success(),
    "generated auth project must pass cargo check"
  );
}

/// Real scenario: `forge new` → patch port → `cargo run` → register → login → GET /api/auth/admin with cookie → 200.
#[tokio::test]
async fn forge_new_project_auth_flow_register_login_protected_route() {
  let workspace = forge_e2e_lib::tmpdir::tmpdir().unwrap();
  let project_name = "auth_serve_test";

  let out = cli::run_forge_new(workspace.path(), project_name);
  assert!(
    out.status.success(),
    "forge new failed: stderr={}",
    String::from_utf8_lossy(&out.stderr)
  );

  let project_root = workspace.path().join(project_name);
  cli::assert_project_layout(&project_root);

  let port = 30_000u16 + (std::process::id() % 1000) as u16;
  fs::write(
    project_root.join("config/app.toml"),
    format!(
      r#"[app]
name = "auth_serve_test"
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

  for _ in 0..300 {
    tokio::time::sleep(Duration::from_millis(200)).await;
    if client.get(format!("{}/", base)).send().await.is_ok() {
      break;
    }
  }

  let reg = client
    .post(format!("{}/api/auth/register", base))
    .json(&serde_json::json!({ "email": "auth-e2e@test.com", "password": "password123" }))
    .send()
    .await
    .expect("register");
  assert!(
    reg.status().is_success(),
    "register should succeed: {}",
    reg.status()
  );

  let login = client
    .post(format!("{}/api/auth/login", base))
    .json(&serde_json::json!({ "email": "auth-e2e@test.com", "password": "password123" }))
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

  let _ = child.kill();
  let _ = child.wait();
}
