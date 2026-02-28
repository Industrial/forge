//! End-to-End tests for Forge Authentication and Session Management

use axum_login::{AuthSession, AuthnBackend};
use forge::prelude::*;
use forge::ForgeAuthUser;
use http::{header, Request, StatusCode};
use serde::Deserialize;
use std::fs;
use std::process::Command;
use tower::ServiceExt;
use uuid::Uuid;

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
async fn forge_new_generates_auth_ready_workspace() {
  let temp_dir = tempfile::tempdir().unwrap();
  let project_name = "auth_test_app";
  let forge_binary = get_forge_binary_path();

  let new_result = Command::new(&forge_binary)
    .arg("new")
    .arg(project_name)
    .current_dir(&temp_dir)
    .output()
    .expect("Failed to run forge new");

  if !new_result.status.success() {
    println!("STDOUT: {}", String::from_utf8_lossy(&new_result.stdout));
    println!("STDERR: {}", String::from_utf8_lossy(&new_result.stderr));
  }
  assert!(new_result.status.success());
  let root = temp_dir.path().join(project_name);

  // Debug: list files
  let entries = fs::read_dir(root.join("crates/db/src")).unwrap();
  for entry in entries {
    println!("File: {:?}", entry.unwrap().path());
  }

  // Check for auth-related files
  assert!(
    root.join("crates/db/src/auth.rs").exists(),
    "crates/db/src/auth.rs missing"
  );

  let cargo_toml = fs::read_to_string(root.join("crates/db/Cargo.toml")).unwrap();
  assert!(
    cargo_toml.contains("axum-login"),
    "db/Cargo.toml should depend on axum-login"
  );
  assert!(
    cargo_toml.contains("tower-sessions"),
    "db/Cargo.toml should depend on tower-sessions"
  );
}

#[derive(Clone, Debug, ForgeAuthUser)]
struct MockUser {
  id: Uuid,
  email: String,
  password_hash: String,
}

#[derive(Clone, Debug)]
struct MockBackend;

#[derive(Debug, Deserialize)]
struct Credentials {
  #[allow(dead_code)]
  email: String,
}

#[async_trait::async_trait]
impl AuthnBackend for MockBackend {
  type User = MockUser;
  type Credentials = Credentials;
  type Error = forge::Error;

  async fn authenticate(
    &self,
    creds: Self::Credentials,
  ) -> Result<Option<Self::User>, Self::Error> {
    Ok(Some(MockUser {
      id: Uuid::new_v4(),
      email: creds.email,
      password_hash: "hash".to_string(),
    }))
  }

  async fn get_user(
    &self,
    _user_id: &axum_login::UserId<Self>,
  ) -> Result<Option<Self::User>, Self::Error> {
    // Return the mock user if we have one?
    // In a real backend, this would fetch from DB.
    // For the test, we'll just return Some if any ID is provided.
    Ok(Some(MockUser {
      id: Uuid::new_v4(),
      email: "test@example.com".to_string(),
      password_hash: "hash".to_string(),
    }))
  }
}

#[tokio::test]
async fn forge_app_handles_full_auth_flow_in_process() {
  let temp_dir = tempfile::tempdir().unwrap();
  let original_cwd = std::env::current_dir().unwrap();
  std::env::set_current_dir(temp_dir.path()).unwrap();

  // 1. Manually setup the environment
  fs::create_dir_all("config").unwrap();
  fs::write(
    "config/app.toml",
    r#"[app]
name = "auth_test"
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
auto_migrate = true
auto_seed = false
"#,
  )
  .unwrap();

  // 2. Initialize App with auth
  let app = App::new()
    .with_auth(|_db| MockBackend)
    .route(
      "/auth/login",
      |mut session: AuthSession<MockBackend>| async move {
        let creds = Credentials {
          email: "test@example.com".to_string(),
        };
        let user = session.authenticate(creds).await.unwrap().unwrap();
        session.login(&user).await.unwrap();
        StatusCode::OK
      },
    )
    .route(
      "/auth/profile",
      |session: AuthSession<MockBackend>| async move {
        if let Some(user) = session.user {
          format!("Hello, {}!", user.email)
        } else {
          "Not logged in".to_string()
        }
      },
    );

  let router = app.into_router().await;

  // 3. Test login and session
  let response = router
    .clone()
    .oneshot(
      Request::builder()
        .method("GET")
        .uri("/auth/login")
        .body(axum::body::Body::empty())
        .unwrap(),
    )
    .await
    .unwrap();
  assert_eq!(response.status(), StatusCode::OK);

  let cookie = response
    .headers()
    .get(header::SET_COOKIE)
    .expect("No session cookie returned");
  println!("Cookie: {:?}", cookie);

  // 4. Test profile access with cookie
  let response = router
    .oneshot(
      Request::builder()
        .uri("/auth/profile")
        .header(header::COOKIE, cookie)
        .body(axum::body::Body::empty())
        .unwrap(),
    )
    .await
    .unwrap();
  assert_eq!(response.status(), StatusCode::OK);
  let body = axum::body::to_bytes(response.into_body(), usize::MAX)
    .await
    .unwrap();
  println!("Body: {:?}", String::from_utf8_lossy(&body));
  assert!(String::from_utf8_lossy(&body).contains("test@example.com"));

  std::env::set_current_dir(original_cwd).unwrap();
}
