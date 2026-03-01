//! E2E tests for Forge rate limiting: real scenarios only.
//!
//! **Pattern**: Every test uses the real `forge` and `cargo` CLIs. We generate a project
//! with `forge new`, then run `cargo check` / `cargo run` on the generated tree and assert
//! on real outcomes (exit codes, file layout, HTTP responses). No in-process mocks.
//!
//! **Structure**:
//! 1. Create a temp dir (project `.tmp/` via `forge_e2e_lib::tmpdir`).
//! 2. Run `forge new <name>` in that dir; assert success.
//! 3. Assert generated layout.
//! 4. Run `cargo check` or `cargo run` in the project dir (target is `project_dir/target`, under `.tmp/`).
//! 5. For “run” scenarios: start the app, hit endpoints, then cleanup.

use std::process::Command;
use std::time::Duration;

use forge_e2e_lib::cli;

// --- Tests (real CLI scenarios) ---

/// Real scenario: `forge new` → assert layout → `cargo check` succeeds.
#[test]
fn forge_new_rate_limit_project_builds() {
  let workspace = forge_e2e_lib::tmpdir::tmpdir().unwrap();
  let project_name = "rate_limit_build_test";

  let out = cli::run_forge_new(workspace.path(), project_name);
  assert!(
    out.status.success(),
    "forge new failed: stderr={}",
    String::from_utf8_lossy(&out.stderr)
  );

  let project_root = workspace.path().join(project_name);
  cli::assert_project_layout(&project_root);

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

/// Real scenario: `forge new` → run → GET /healthz repeatedly → all 200 (health excluded from rate limiting).
#[tokio::test]
async fn forge_new_project_health_endpoints_not_rate_limited() {
  let workspace = forge_e2e_lib::tmpdir::tmpdir().unwrap();
  let project_name = "rate_limit_serve_test";

  let out = cli::run_forge_new(workspace.path(), project_name);
  assert!(
    out.status.success(),
    "forge new failed: stderr={}",
    String::from_utf8_lossy(&out.stderr)
  );

  let project_root = workspace.path().join(project_name);
  cli::assert_project_layout(&project_root);

  let port = 30_000u16 + (std::process::id() % 1000) as u16;
  std::fs::write(
    project_root.join("config/app.toml"),
    format!(
      r#"[app]
name = "rate_limit_serve_test"
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
    .timeout(Duration::from_secs(5))
    .build()
    .unwrap();
  let base = format!("http://127.0.0.1:{}/healthz", port);

  for i in 0..300 {
    tokio::time::sleep(Duration::from_millis(200)).await;
    if client.get(&base).send().await.is_ok() {
      for _ in 0..5 {
        let r = client.get(&base).send().await.unwrap();
        assert_eq!(r.status().as_u16(), 200, "healthz must not be rate limited");
      }
      let _ = child.kill();
      let _ = child.wait();
      return;
    }
    if i == 299 {
      let _ = child.kill();
      let _ = child.wait();
      panic!("server did not become ready in time");
    }
  }
  let _ = child.wait();
}
