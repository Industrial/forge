//! End-to-End tests for Forge CLI config module

use forge::App;
use std::fs;
use std::process::Command;

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
async fn forge_new_generates_config_file() {
  let temp_dir = tempfile::tempdir().unwrap();
  let project_name = "config_test_app";
  let forge_binary = get_forge_binary_path();

  let new_result = Command::new(&forge_binary)
    .arg("new")
    .arg(project_name)
    .current_dir(&temp_dir)
    .output()
    .expect("Failed to run forge new");

  assert!(new_result.status.success());

  let config_path = temp_dir.path().join(project_name).join("config/app.toml");
  assert!(config_path.exists());
  let db_config_path = temp_dir.path().join(project_name).join("config/db.toml");
  assert!(db_config_path.exists());
}

#[tokio::test]
async fn forge_app_loads_config_in_process() {
  let temp_dir = tempfile::tempdir().unwrap();
  let original_cwd = std::env::current_dir().unwrap();
  std::env::set_current_dir(temp_dir.path()).unwrap();

  fs::create_dir_all("config").unwrap();
  fs::write(
    "config/app.toml",
    r#"[app]
name = "custom_app"
environment = "production"
[server]
host = "1.2.3.4"
port = 8080
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

  let app = App::new();
  assert_eq!(app.config().app.name, "custom_app");
  assert_eq!(app.config().server.port, 8080);
  assert_eq!(app.config().server.host, "1.2.3.4");

  std::env::set_current_dir(original_cwd).unwrap();
}

#[tokio::test]
async fn forge_serve_fails_without_config() {
  let temp_dir = tempfile::tempdir().unwrap();
  let original_cwd = std::env::current_dir().unwrap();
  std::env::set_current_dir(temp_dir.path()).unwrap();

  // No config files created here.

  // App::new() should exit(1) which we can't easily catch in-process without refactoring error handling.
  // But we can test that config::load_config() returns an error.
  let result = forge::config::load_config();
  assert!(result.is_err());
  assert!(result.unwrap_err().to_string().contains("app.toml"));

  std::env::set_current_dir(original_cwd).unwrap();
}
