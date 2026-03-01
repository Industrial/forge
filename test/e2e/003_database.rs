//! End-to-End tests for Forge CLI database module

use axum::extract::State;
use forge::App;
use forge::sea_orm::{ConnectionTrait, DatabaseConnection};
use http::{Request, StatusCode};
use std::fs;
use std::process::Command;
use tower::ServiceExt; // for oneshot

/// Helper function to get the path to the forge binary
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
async fn forge_new_generates_db_config_file() {
  let temp_dir = tempfile::tempdir().unwrap();
  let project_name = "db_test_app";
  let forge_binary = get_forge_binary_path();

  let new_result = Command::new(&forge_binary)
    .arg("new")
    .arg(project_name)
    .current_dir(&temp_dir)
    .output()
    .expect("Failed to run forge new");

  assert!(new_result.status.success());

  let db_config_path = temp_dir.path().join(project_name).join("config/db.toml");
  assert!(db_config_path.exists());

  let content = fs::read_to_string(db_config_path).unwrap();
  assert!(content.contains("[database]"));
  assert!(content.contains("sqlite://db.sqlite"));
}

#[tokio::test]
async fn forge_app_initializes_database_in_process() {
  let temp_dir = tempfile::tempdir().unwrap();
  let original_cwd = std::env::current_dir().unwrap();
  std::env::set_current_dir(temp_dir.path()).unwrap();

  // 1. Manually setup the environment (simulating forge new)
  fs::create_dir_all("config").unwrap();
  fs::write(
    "config/app.toml",
    r#"[app]
name = "test"
environment = "test"
[server]
host = "127.0.0.1"
port = 3000
"#,
  )
  .unwrap();
  fs::write(
    "config/db.toml",
    r#"[database]
url = "sqlite::memory:"
"#,
  )
  .unwrap();

  // 2. Initialize App in-process
  let app = App::new().route(
    "/db-check",
    |State(db): State<DatabaseConnection>| async move {
      let backend = db.get_database_backend();
      format!("Connected to {:?}", backend)
    },
  );

  let (router, _) = app.into_router().await;

  // 3. Send a virtual request
  let response = router
    .oneshot(
      Request::builder()
        .uri("/db-check")
        .body(axum::body::Body::empty())
        .unwrap(),
    )
    .await
    .unwrap();

  assert_eq!(response.status(), StatusCode::OK);

  let body = axum::body::to_bytes(response.into_body(), usize::MAX)
    .await
    .unwrap();
  let body_str = String::from_utf8_lossy(&body);

  assert!(body_str.contains("Sqlite"));

  std::env::set_current_dir(original_cwd).unwrap();
}
