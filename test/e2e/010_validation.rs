//! E2E tests for Forge request validation (010).
//!
//! Covers: Valid<Json<T>> returns 200 for valid input, 422 with structured errors for invalid input;
//! generated app auth endpoints use validation (register/login return 422 for invalid body).

use axum::Json;
use axum::extract::State;
use axum::http::StatusCode;
use forge::sea_orm::DatabaseConnection;
use forge::validation::Valid;
use serde::Deserialize;
use std::fs;
use std::process::Command;
use std::time::Duration;
use validator::Validate;

#[derive(Debug, Deserialize, Validate)]
struct CreateUserRequest {
  #[validate(email)]
  pub email: String,
  #[validate(length(min = 8))]
  pub password: String,
}

async fn create_user(
  State(_db): State<DatabaseConnection>,
  Valid(Json(body)): Valid<Json<CreateUserRequest>>,
) -> StatusCode {
  let _ = body;
  StatusCode::OK
}

/// E2E: valid JSON body passes validation and handler returns 200.
#[tokio::test]
async fn valid_json_returns_200() {
  let temp_dir = tempfile::tempdir().unwrap();
  let original_cwd = std::env::current_dir().unwrap();
  std::env::set_current_dir(temp_dir.path()).unwrap();

  let config_dir = temp_dir.path().join("config");
  fs::create_dir_all(&config_dir).unwrap();
  fs::write(
    config_dir.join("app.toml"),
    r#"[app]
name = "validation_e2e"
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

  let app = forge::App::new().post_route("/users", create_user);
  let (router, _) = app.into_router().await;
  let listener = tokio::net::TcpListener::bind("127.0.0.1:0").await.unwrap();
  let port = listener.local_addr().unwrap().port();
  tokio::spawn(async move {
    axum::serve(
      listener,
      router.into_make_service_with_connect_info::<std::net::SocketAddr>(),
    )
    .await
  });

  tokio::time::sleep(Duration::from_millis(100)).await;

  let client = reqwest::Client::builder()
    .timeout(Duration::from_secs(5))
    .build()
    .unwrap();
  let base = format!("http://127.0.0.1:{}", port);

  let resp = client
    .post(format!("{}/users", base))
    .json(&serde_json::json!({
      "email": "user@example.com",
      "password": "password123"
    }))
    .send()
    .await
    .unwrap();

  assert_eq!(
    resp.status().as_u16(),
    200,
    "valid request should return 200, got {}",
    resp.status()
  );

  let _ = std::env::set_current_dir(&original_cwd);
}

/// E2E: invalid JSON (bad email, short password) returns 422 with structured error body.
#[tokio::test]
async fn invalid_json_returns_422_with_errors() {
  let temp_dir = tempfile::tempdir().unwrap();
  let original_cwd = std::env::current_dir().unwrap();
  std::env::set_current_dir(temp_dir.path()).unwrap();

  let config_dir = temp_dir.path().join("config");
  fs::create_dir_all(&config_dir).unwrap();
  fs::write(
    config_dir.join("app.toml"),
    r#"[app]
name = "validation_e2e"
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

  let app = forge::App::new().post_route("/users", create_user);
  let (router, _) = app.into_router().await;
  let listener = tokio::net::TcpListener::bind("127.0.0.1:0").await.unwrap();
  let port = listener.local_addr().unwrap().port();
  tokio::spawn(async move {
    axum::serve(
      listener,
      router.into_make_service_with_connect_info::<std::net::SocketAddr>(),
    )
    .await
  });

  tokio::time::sleep(Duration::from_millis(100)).await;

  let client = reqwest::Client::builder()
    .timeout(Duration::from_secs(5))
    .build()
    .unwrap();
  let base = format!("http://127.0.0.1:{}", port);

  // Invalid email + password too short
  let resp = client
    .post(format!("{}/users", base))
    .json(&serde_json::json!({
      "email": "not-an-email",
      "password": "short"
    }))
    .send()
    .await
    .unwrap();

  assert_eq!(
    resp.status().as_u16(),
    422,
    "invalid request should return 422 Unprocessable Entity, got {}",
    resp.status()
  );

  let body = resp.text().await.unwrap();
  assert!(
    body.contains("errors") || body.contains("email") || body.contains("password"),
    "response body should contain error details (errors/field names), got: {}",
    body
  );

  let _ = std::env::set_current_dir(&original_cwd);
}

/// E2E: generated app (forge new) auth endpoints use validation — invalid register/login return 422.
#[tokio::test]
async fn generated_app_validation_register_and_login_return_422_for_invalid() {
  let workspace_root = std::path::Path::new(env!("CARGO_MANIFEST_DIR")).join("../..");
  let build = Command::new("cargo")
    .args(["build", "-p", "forge-cli"])
    .current_dir(&workspace_root)
    .output()
    .expect("cargo build forge-cli");
  assert!(
    build.status.success(),
    "forge-cli must build: {}",
    String::from_utf8_lossy(&build.stderr)
  );

  let temp_dir = tempfile::tempdir().unwrap();
  let forge_binary = workspace_root.join("target").join("debug").join("forge");
  assert!(
    forge_binary.exists(),
    "forge binary not found at {} (run cargo build -p forge-cli)",
    forge_binary.display()
  );

  let out = Command::new(&forge_binary)
    .arg("new")
    .arg("validation_gen_e2e")
    .current_dir(temp_dir.path())
    .output()
    .expect("forge new");
  assert!(
    out.status.success(),
    "forge new failed: {}",
    String::from_utf8_lossy(&out.stderr)
  );

  let project_dir = temp_dir.path().join("validation_gen_e2e");
  let main_rs = fs::read_to_string(project_dir.join("crates/app/src/main.rs")).unwrap();
  assert!(
    main_rs.contains("/api/auth/register"),
    "generated main.rs must contain /api/auth/register (template may be stale)"
  );

  fs::write(
    project_dir.join("config/app.toml"),
    r#"[app]
name = "validation_gen_e2e"
environment = "test"

[server]
host = "127.0.0.1"
port = 30997
"#,
  )
  .unwrap();

  let mut child = Command::new("cargo")
    .args(["run", "-p", "app", "--quiet"])
    .current_dir(&project_dir)
    .stdout(std::process::Stdio::null())
    .stderr(std::process::Stdio::piped())
    .spawn()
    .expect("spawn cargo run");

  let client = reqwest::Client::builder()
    .timeout(Duration::from_secs(5))
    .build()
    .unwrap();
  let base = "http://127.0.0.1:30997";

  for i in 0..150 {
    tokio::time::sleep(Duration::from_millis(200)).await;
    if client.get(format!("{}/", base)).send().await.is_ok() {
      break;
    }
    if i == 149 {
      let _ = child.kill();
      panic!("server did not become ready in time");
    }
  }

  // Invalid register: bad email, short password -> 422
  let reg_invalid = client
    .post(format!("{}/api/auth/register", base))
    .json(&serde_json::json!({ "email": "not-an-email", "password": "short" }))
    .send()
    .await
    .expect("register request");
  assert_eq!(
    reg_invalid.status().as_u16(),
    422,
    "invalid register should return 422, got {}",
    reg_invalid.status()
  );
  let body = reg_invalid.text().await.unwrap();
  assert!(
    body.contains("errors") || body.contains("email") || body.contains("password"),
    "422 body should contain error details, got: {}",
    body
  );

  // Valid register -> 201
  let reg_ok = client
    .post(format!("{}/api/auth/register", base))
    .json(&serde_json::json!({ "email": "valid@example.com", "password": "password123" }))
    .send()
    .await
    .expect("register request");
  assert!(
    reg_ok.status().is_success(),
    "valid register should succeed, got {}",
    reg_ok.status()
  );

  // Invalid login: bad email -> 422
  let login_invalid = client
    .post(format!("{}/api/auth/login", base))
    .json(&serde_json::json!({ "email": "bad-email", "password": "any" }))
    .send()
    .await
    .expect("login request");
  assert_eq!(
    login_invalid.status().as_u16(),
    422,
    "invalid login body should return 422, got {}",
    login_invalid.status()
  );

  let _ = child.kill();
  let _ = child.wait();
}
