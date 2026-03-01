//! E2E tests for Forge health endpoints (008).
//!
//! Covers: GET /healthz, /livez, /readyz return correct status and minimal body;
//! no component or server details in responses (security).

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

/// Assert GET url returns status and body (no component disclosure).
async fn assert_health_endpoint(
  client: &reqwest::Client,
  url: &str,
  expected_status: u16,
  expected_body: &str,
) {
  let resp = client.get(url).send().await.expect("request");
  assert_eq!(
    resp.status().as_u16(),
    expected_status,
    "{} should return {}",
    url,
    expected_status
  );
  let body = resp.text().await.expect("body");
  assert_eq!(
    body.trim(),
    expected_body,
    "{} body should be {:?}, got {:?}",
    url,
    expected_body,
    body
  );
  assert!(
    !body.contains("components") && !body.contains("database"),
    "response must not disclose components: {:?}",
    body
  );
}

/// E2E: in-process server (config + db + router), then GET health endpoints.
#[tokio::test]
async fn healthz_livez_readyz_return_200_with_minimal_body() {
  let temp_dir = tempfile::tempdir().unwrap();
  let original_cwd = std::env::current_dir().unwrap();
  std::env::set_current_dir(temp_dir.path()).unwrap();

  let config_dir = temp_dir.path().join("config");
  fs::create_dir_all(&config_dir).unwrap();
  fs::write(
    config_dir.join("app.toml"),
    r#"[app]
name = "health_e2e"
environment = "test"

[server]
host = "127.0.0.1"
port = 0
"#,
  )
  .unwrap();
  fs::write(
    config_dir.join("db.toml"),
    r#"[database]
url = "sqlite::memory:"
"#,
  )
  .unwrap();

  let app = forge::App::new();
  let (router, _) = app.into_router().await;
  let listener = tokio::net::TcpListener::bind("127.0.0.1:0").await.unwrap();
  let port = listener.local_addr().unwrap().port();
  tokio::spawn(async move { axum::serve(listener, router).await });

  tokio::time::sleep(Duration::from_millis(100)).await;

  let client = reqwest::Client::builder()
    .timeout(Duration::from_secs(5))
    .build()
    .unwrap();
  let base = format!("http://127.0.0.1:{}", port);

  assert_health_endpoint(&client, &format!("{}/healthz", base), 200, "ok").await;
  assert_health_endpoint(&client, &format!("{}/livez", base), 200, "ok").await;
  assert_health_endpoint(&client, &format!("{}/readyz", base), 200, "ok").await;

  std::env::set_current_dir(original_cwd).unwrap();
}

/// E2E: full flow with `forge new` + run generated app (optional; can be slow).
#[tokio::test]
#[ignore = "slow: forge new + cargo run; run with --ignored"]
async fn health_endpoints_from_generated_app() {
  let temp_dir = tempfile::tempdir().unwrap();
  let forge_binary = get_forge_binary_path();
  let port = 30997u16 + (std::process::id() % 1000) as u16;

  let out = Command::new(&forge_binary)
    .arg("new")
    .arg("health_e2e_app")
    .current_dir(&temp_dir)
    .output()
    .expect("forge new");
  assert!(
    out.status.success(),
    "forge new: {}",
    String::from_utf8_lossy(&out.stderr)
  );

  let project_dir = temp_dir.path().join("health_e2e_app");
  fs::write(
    project_dir.join("config/app.toml"),
    format!(
      r#"[app]
name = "health_e2e"
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
    .current_dir(&project_dir)
    .stdout(std::process::Stdio::null())
    .stderr(std::process::Stdio::piped())
    .spawn()
    .expect("spawn cargo run");

  let client = reqwest::Client::builder()
    .timeout(Duration::from_secs(5))
    .build()
    .unwrap();
  let base = format!("http://127.0.0.1:{}", port);

  for i in 0..300 {
    tokio::time::sleep(Duration::from_millis(400)).await;
    if client.get(format!("{}/healthz", base)).send().await.is_ok() {
      assert_health_endpoint(&client, &format!("{}/healthz", base), 200, "ok").await;
      assert_health_endpoint(&client, &format!("{}/livez", base), 200, "ok").await;
      assert_health_endpoint(&client, &format!("{}/readyz", base), 200, "ok").await;
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
  // Loop exited without return/panic (unreachable); ensure child is reaped
  let _ = child.wait();
}
