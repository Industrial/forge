//! E2E tests for Forge validation: real scenarios only.
//!
//! **Pattern**: Every test uses the real `forge` and `cargo` CLIs. We generate a project
//! with `forge new`, then run `cargo check` / `cargo run` on the generated tree and assert
//! on real outcomes (exit codes, file layout, HTTP responses). No in-process mocks.
//!
//! **Structure**:
//! 1. Create a temp dir (project `.tmp/` via `forge_e2e_lib::tmpdir`).
//! 2. Run `forge new <name>` in that dir; assert success.
//! 3. Assert generated app has auth routes (register/login with validation).
//! 4. Run `cargo run` in the project dir (target is `project_dir/target`, under `.tmp/`).
//! 5. Invalid register/login → 422 with error details; valid register → success.

use std::fs;
use std::process::Command;
use std::time::Duration;

use forge_e2e_lib::cli;

// --- Tests (real CLI scenarios) ---

/// Real scenario: `forge new` → assert main.rs has /api/auth/register → `cargo check` succeeds.
#[test]
fn forge_new_validation_project_builds() {
  let workspace = forge_e2e_lib::tmpdir::tmpdir().unwrap();
  let project_name = "validation_build_test";

  let out = cli::run_forge_new(workspace.path(), project_name);
  assert!(
    out.status.success(),
    "forge new failed: stderr={}",
    String::from_utf8_lossy(&out.stderr)
  );

  let project_root = workspace.path().join(project_name);
  cli::assert_project_layout(&project_root);

  let main_rs = fs::read_to_string(project_root.join("crates/app/src/main.rs")).unwrap();
  assert!(
    main_rs.contains("/api/auth/register"),
    "generated main.rs must contain /api/auth/register"
  );

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

/// Real scenario: `forge new` → run → invalid register 422, valid register success, invalid login 422.
#[tokio::test]
async fn generated_app_register_and_login_validation_422_for_invalid() {
  let workspace = forge_e2e_lib::tmpdir::tmpdir().unwrap();
  let project_name = "validation_serve_test";

  let out = cli::run_forge_new(workspace.path(), project_name);
  assert!(
    out.status.success(),
    "forge new failed: stderr={}",
    String::from_utf8_lossy(&out.stderr)
  );

  let project_root = workspace.path().join(project_name);
  let port = 30_000u16 + (std::process::id() % 1000) as u16;
  std::fs::write(
    project_root.join("config/app.toml"),
    format!(
      r#"[app]
name = "validation_serve_test"
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
    .args(["run", "-p", "app", "--quiet"])
    .current_dir(&project_root)
    .stdout(std::process::Stdio::null())
    .stderr(std::process::Stdio::piped())
    .spawn()
    .expect("spawn cargo run");

  let client = reqwest::Client::builder()
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

  let reg_invalid = client
    .post(format!("{}/api/auth/register", base))
    .json(&serde_json::json!({ "email": "not-an-email", "password": "short" }))
    .send()
    .await
    .expect("register");
  assert_eq!(
    reg_invalid.status().as_u16(),
    422,
    "invalid register should return 422: {}",
    reg_invalid.status()
  );
  let body = reg_invalid.text().await.unwrap();
  assert!(
    body.contains("errors") || body.contains("email") || body.contains("password"),
    "422 body should contain error details: {}",
    body
  );

  let reg_ok = client
    .post(format!("{}/api/auth/register", base))
    .json(&serde_json::json!({ "email": "valid@example.com", "password": "password123" }))
    .send()
    .await
    .expect("register");
  assert!(
    reg_ok.status().is_success(),
    "valid register should succeed: {}",
    reg_ok.status()
  );

  let login_invalid = client
    .post(format!("{}/api/auth/login", base))
    .json(&serde_json::json!({ "email": "bad-email", "password": "any" }))
    .send()
    .await
    .expect("login");
  assert_eq!(
    login_invalid.status().as_u16(),
    422,
    "invalid login should return 422: {}",
    login_invalid.status()
  );

  let _ = child.kill();
  let _ = child.wait();
}
