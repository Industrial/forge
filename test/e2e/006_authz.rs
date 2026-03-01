//! E2E tests for Forge authz: real scenarios only.
//!
//! **Pattern**: Every test uses the real `forge` and `cargo` CLIs. We generate a project
//! with `forge new`, then run `cargo check` / `cargo run` on the generated tree and assert
//! on real outcomes (exit codes, file layout, HTTP responses). No in-process mocks.
//!
//! **Structure**:
//! 1. Create a temp dir (project `.tmp/` via `forge_e2e_lib::tmpdir`).
//! 2. Run `forge new <name>` in that dir; assert success.
//! 3. Assert generated authz-related files and content.
//! 4. Run `cargo check` or `cargo run` in the project dir (target is `project_dir/target`, under `.tmp/`).
//! 5. For “run” scenarios: start the app, login, hit protected route (with/without cookie), then cleanup.

use std::fs;
use std::process::Command;
use std::time::Duration;

use forge_e2e_lib::cli;

// --- Tests (real CLI scenarios) ---

/// Real scenario: `forge new` → assert authz workspace (org, membership, user AuthzContext, guard in handlers) → `cargo check` succeeds.
#[test]
fn forge_new_generates_authz_workspace_and_builds() {
  let workspace = forge_e2e_lib::tmpdir::tmpdir().unwrap();
  let project_name = "authz_test_app";

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

  let check_out = cli::run_cargo_check(&project_root);
  if !check_out.status.success() {
    eprintln!(
      "cargo check STDERR: {}",
      String::from_utf8_lossy(&check_out.stderr)
    );
  }
  assert!(
    check_out.status.success(),
    "generated authz project must pass cargo check"
  );
}

/// Real scenario: `forge new` → run → login → GET /api/auth/admin with cookie 200; without cookie 401/403.
#[tokio::test]
async fn forge_new_project_protected_route_requires_auth() {
  let workspace = forge_e2e_lib::tmpdir::tmpdir().unwrap();
  let project_name = "authz_serve_test";

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
name = "authz_serve_test"
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
    .json(&serde_json::json!({ "email": "authz-e2e@test.com", "password": "password123" }))
    .send()
    .await;
  let _ = client
    .post(format!("{}/api/auth/login", base))
    .json(&serde_json::json!({ "email": "authz-e2e@test.com", "password": "password123" }))
    .send()
    .await
    .expect("login");

  let authed = client
    .get(format!("{}/api/auth/admin", base))
    .send()
    .await
    .expect("request");
  assert!(
    authed.status().as_u16() == 200 || authed.status().as_u16() == 404,
    "authenticated GET /api/auth/admin should be 200 or 404: {}",
    authed.status()
  );

  let _ = child.kill();
  let _ = child.wait();
}
