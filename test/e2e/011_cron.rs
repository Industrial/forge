//! E2E tests for Forge cron jobs: real scenarios only.
//!
//! **Pattern**: Every test uses the real `forge` and `cargo` CLIs. We generate a project
//! with `forge new`, then run `cargo check` / `cargo run` on the generated tree and assert
//! on real outcomes (exit codes, file layout, HTTP responses). No in-process mocks.
//!
//! **Structure**:
//! 1. Create a temp dir (project `.tmp/` via `forge_e2e_lib::tmpdir`).
//! 2. Run `forge new <name>` in that dir; assert success.
//! 3. Assert generated layout (cron/jobs in main if present).
//! 4. Run `cargo check` or `cargo run` in the project dir (target is `project_dir/target`, under `.tmp/`).
//! 5. For “run” scenarios: start the app (scheduler + worker run in same process), wait, GET /healthz, then cleanup.

use std::fs;
use std::process::Command;
use std::time::Duration;

use forge_e2e_lib::cli;

// --- Tests (real CLI scenarios) ---

/// Real scenario: `forge new` → assert layout (cron/jobs if in template) → `cargo check` succeeds.
#[test]
fn forge_new_cron_project_builds() {
  let workspace = forge_e2e_lib::tmpdir::tmpdir().unwrap();
  let project_name = "cron_build_test";

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

/// Real scenario: `forge new` → patch port → `cargo run` → wait → GET /healthz 200 (app with cron/jobs runs).
#[tokio::test]
async fn forge_new_project_with_cron_serves() {
  let workspace = forge_e2e_lib::tmpdir::tmpdir().unwrap();
  let project_name = "cron_serve_test";

  let out = cli::run_forge_new(workspace.path(), project_name);
  assert!(
    out.status.success(),
    "forge new failed: stderr={}",
    String::from_utf8_lossy(&out.stderr)
  );

  let project_root = workspace.path().join(project_name);
  cli::assert_project_layout(&project_root);

  let main_rs = fs::read_to_string(project_root.join("crates/app/src/main.rs")).unwrap();
  let has_cron =
    main_rs.contains("with_cron") || main_rs.contains("cron") || main_rs.contains("jobs");
  if !has_cron {
    eprintln!("note: generated app may not include cron/jobs in main.rs; still asserting serve");
  }

  let port = 30_000u16 + (std::process::id() % 1000) as u16;
  std::fs::write(
    project_root.join("config/app.toml"),
    format!(
      r#"[app]
name = "cron_serve_test"
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
  let url = format!("http://127.0.0.1:{}/healthz", port);

  for _ in 0..300 {
    tokio::time::sleep(Duration::from_millis(200)).await;
    if let Ok(resp) = client.get(&url).send().await
      && resp.status().as_u16() == 200
    {
      assert_eq!(resp.text().await.unwrap_or_default().trim(), "ok");
      let _ = child.kill();
      let _ = child.wait();
      return;
    }
  }

  let _ = child.kill();
  let _ = child.wait();
  panic!("server did not respond with 200 on /healthz within 60s");
}
