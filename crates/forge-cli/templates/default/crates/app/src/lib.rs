//! Library for the template app: route registration and test harness.
//!
//! Use [make_app] (with CWD set to a directory containing `config/app.toml` and `config/db.toml`)
//! or [build_router_for_test] for integration tests (temp config + in-memory DB).

pub use serde_json;

use std::path::PathBuf;
use std::sync::Arc;

#[cfg(all(test, not(feature = "test-utils")))]
static BUILD_ROUTER_FOR_TEST_LOCK: std::sync::Mutex<()> = std::sync::Mutex::new(());

use axum::Router;
use forge_app::App;
use tempfile::TempDir;

pub mod error;
pub mod handlers;
pub mod permissions;
pub mod tasks;

pub use error::Error;

/// Build the Forge [App] with all template routes. Requires CWD to be a directory that contains
/// `config/app.toml` and `config/db.toml` (e.g. project root or a temp dir from [build_router_for_test]).
pub fn make_app(live_backend: Arc<forge_live::InMemoryLiveBackend>) -> App {
  let app = App::new()
    .with_token_auth_only(
      |db| db::auth::Backend::new(db),
      Arc::new(move |db, raw_token| Box::pin(db::token_lookup(db, raw_token))),
    )
    .with_live_query_using(live_backend);

  app
    .route("/api/cache-demo", handlers::cache_demo::handler)
    .route("/api/cached-page", handlers::cached_page::handler)
    .route(
      "/api/observability/trace-id",
      handlers::observability::trace_id,
    )
    .route("/ws", handlers::ws::handler)
    .post_route("/api/auth/register", handlers::auth::register)
    .post_route("/api/auth/login", handlers::auth::login)
    .route("/api/auth/logout", handlers::auth::logout)
    .route("/api/auth/me", axum::routing::get(handlers::auth::get_me))
    .route("/api/auth/profiles", axum::routing::get(handlers::auth::profiles_list))
    .post_route("/api/auth/tokens", handlers::auth::create_token)
    .route("/api/auth/admin", handlers::auth::admin_only)
    // Unprotected REST API (no auth)
    .route("/api/permissions", axum::routing::get(handlers::rest::list_permissions))
    .route_methods(
      "/api/users",
      axum::routing::get(handlers::rest::list_users).post(handlers::rest::create_user),
    )
    .route("/api/users/me", axum::routing::get(handlers::rest::users_me))
    .route_methods(
      "/api/users/{id}",
      axum::routing::get(handlers::rest::get_user)
        .patch(handlers::rest::update_user)
        .delete(handlers::rest::delete_user),
    )
    .route("/api/users/{id}/organizations", axum::routing::get(handlers::rest::get_user_organizations))
    .route_methods(
      "/api/organizations",
      axum::routing::get(handlers::rest::list_organizations).post(handlers::rest::create_organization),
    )
    .route_methods(
      "/api/organizations/{id}",
      axum::routing::get(handlers::rest::get_organization)
        .patch(handlers::rest::update_organization)
        .delete(handlers::rest::delete_organization),
    )
    .route_methods(
      "/api/organizations/{id}/users",
      axum::routing::get(handlers::rest::list_org_users).post(handlers::rest::add_org_user),
    )
    .route(
      "/api/organizations/{id}/users/{user_id}/roles",
      axum::routing::post(handlers::rest::add_org_user_roles),
    )
    .route_methods(
      "/api/organizations/{id}/users/{user_id}",
      axum::routing::get(handlers::rest::get_org_user)
        .patch(handlers::rest::update_org_user)
        .delete(handlers::rest::delete_org_user),
    )
    .route_methods(
      "/api/organizations/{id}/roles",
      axum::routing::get(handlers::rest::list_org_roles).post(handlers::rest::create_org_role),
    )
    .route_methods(
      "/api/organizations/{id}/roles/{role_id}/permissions",
      axum::routing::get(handlers::rest::list_org_role_permissions)
        .post(handlers::rest::add_org_role_permission)
        .delete(handlers::rest::delete_org_role_permission),
    )
    .route_methods(
      "/api/organizations/{id}/roles/{role_id}",
      axum::routing::get(handlers::rest::get_org_role)
        .patch(handlers::rest::update_org_role)
        .delete(handlers::rest::delete_org_role),
    )
    .route("/api/audit-log", axum::routing::get(handlers::rest::list_audit_log))
    .route("/api/audit-log/{id}", axum::routing::get(handlers::rest::get_audit_log))
}

/// Guard that restores the previous working directory when dropped. Keep this alive for the
/// duration of the test so CWD is restored afterward.
pub struct TestEnvGuard {
  _temp: TempDir,
  original_cwd: PathBuf,
}

impl Drop for TestEnvGuard {
  fn drop(&mut self) {
    let _ = std::env::set_current_dir(&self.original_cwd);
  }
}

/// Write `config/app.toml` and `config/db.toml` under `dir` for integration tests (in-process path only).
#[cfg(all(test, not(feature = "test-utils")))]
fn write_test_config(
  dir: &std::path::Path,
  db_name: &str,
) -> Result<(), Box<dyn std::error::Error + Send + Sync>> {
  let config_dir = dir.join("config");
  std::fs::create_dir_all(&config_dir)?;
  std::fs::write(
    config_dir.join("app.toml"),
    r#"[app]
name = "test"

[server]
host = "127.0.0.1"
port = 0

[frontend]
port = 3000
"#,
  )?;
  // Use a file DB in the temp dir so the same connection is used for migrations and handlers (SQLite in-memory + pool can use different connections).
  let db_path = dir.join("data").join(format!("{}.db", db_name));
  std::fs::create_dir_all(dir.join("data")).ok();
  let db_url = format!("sqlite:{}?mode=rwc", db_path.display());
  let db_toml = format!(
    r#"[database]
url = "{}"
auto_migrate = true
auto_seed = true
"#,
    db_url.replace('\\', "/")
  );
  std::fs::write(config_dir.join("db.toml"), db_toml)?;
  Ok(())
}

/// Client for integration tests. When the environment variable `E2E_API_URL` is set (e.g. by bin/test-integration), uses that server; otherwise builds an in-process router.
#[cfg(any(test, feature = "test-utils"))]
#[derive(Clone)]
pub enum TestClient {
  /// Prebuilt server already running (bin/test-integration).
  Http {
    client: reqwest::Client,
    base_url: String,
  },
  /// In-process router (e.g. `cargo test` without E2E_API_URL).
  InProcess {
    router: Router,
    _guard: std::sync::Arc<std::sync::Mutex<Option<TestEnvGuard>>>,
  },
}

/// Returns a [TestClient]. If `E2E_API_URL` is set, uses that server (no per-test server).
/// When running with `--features test-utils` (e.g. bin/test-integration), E2E_API_URL must be set.
#[cfg(any(test, feature = "test-utils"))]
pub async fn test_client(
) -> Result<TestClient, Box<dyn std::error::Error + Send + Sync>> {
  if let Ok(url) = std::env::var("E2E_API_URL") {
    let base_url = url.trim_end_matches('/').to_string();
    let client = reqwest::Client::builder()
      .timeout(std::time::Duration::from_secs(10))
      .build()?;
    return Ok(TestClient::Http { client, base_url });
  }
  #[cfg(all(test, not(feature = "test-utils")))]
  {
    let (router, guard) = build_router_for_test().await?;
    return Ok(TestClient::InProcess {
      router,
      _guard: std::sync::Arc::new(std::sync::Mutex::new(Some(guard))),
    });
  }
  #[cfg(any(not(test), feature = "test-utils"))]
  Err("integration tests require E2E_API_URL (e.g. run bin/test-integration)".into())
}

/// Build the API router for integration tests (in-process). Only compiled when not using `test-utils` feature.
/// When using `test-utils`, use E2E_API_URL and the HTTP client instead.
#[cfg(all(test, not(feature = "test-utils")))]
pub async fn build_router_for_test(
) -> Result<(Router, TestEnvGuard), Box<dyn std::error::Error + Send + Sync>> {
  let temp = TempDir::new()?;
  let original_cwd = std::env::current_dir()?;
  let db_name = format!("testdb_{}", uuid::Uuid::new_v4());
  write_test_config(temp.path(), &db_name)?;

  let _lock = BUILD_ROUTER_FOR_TEST_LOCK.lock().expect("test router build lock");

  std::env::set_current_dir(temp.path())?;
  let guard = TestEnvGuard {
    original_cwd,
    _temp: temp,
  };
  forge_app::init_tracing();
  let live_backend = Arc::new(forge_live::InMemoryLiveBackend::new());
  let app = make_app(live_backend.clone());

  let (router, db_conn, _cron_runner, response_cache) = app.into_router_before_state().await;

  // Run migrations and seeds on the same db_conn the router uses (only when building in-process path, not when feature test-utils).
  #[cfg(all(test, not(feature = "test-utils")))]
  {
    use sea_orm::{ConnectionTrait, Statement};
    use sea_orm_migration::MigratorTrait;
    migrations::Migrator::up(&db_conn, None).await.map_err(|e| e.to_string())?;
    migrations::run_seeds(db_conn.clone())
      .await
      .map_err(|e| e.to_string())?;
    let _ = db_conn
      .execute(Statement::from_string(
        db_conn.get_database_backend(),
        "SELECT 1 FROM user LIMIT 1".to_string(),
      ))
      .await
      .map_err(|e| format!("user table check after migration: {}", e))?;
  }

  let task_state = Arc::new(tasks::TaskState::new());
  let api_router = router
    .with_state(db_conn.clone())
    .layer(axum::extract::Extension(db_conn))
    .layer(axum::extract::Extension(task_state));

  let router = if let Some(cache_layer) = response_cache {
    api_router.layer(cache_layer)
  } else {
    api_router
  };

  Ok((router, guard))
}

/// Default password for all seed users (must match db seeds).
pub const SEED_PASSWORD: &str = "password";

#[cfg(any(test, feature = "test-utils"))]
async fn login_as_seed_user_impl(
  client: &TestClient,
  email: &str,
  password: &str,
) -> Result<String, Box<dyn std::error::Error + Send + Sync>> {
  let (status, body) = test_request_impl(client, "POST", "/api/auth/login", None, Some(&serde_json::json!({ "email": email, "password": password }).to_string()), None).await?;
  if status != axum::http::StatusCode::OK {
    let msg = String::from_utf8_lossy(&body);
    return Err(format!("login failed {}: {}", status, msg).into());
  }
  let json: serde_json::Value = serde_json::from_slice(&body)?;
  let token = json["token"].as_str().ok_or("token not found in response")?;
  Ok(token.to_string())
}

#[cfg(any(test, feature = "test-utils"))]
async fn test_request_impl(
  client: &TestClient,
  method: &str,
  path: &str,
  token: Option<&str>,
  body: Option<&str>,
  extra_headers: Option<&[(&str, &str)]>,
) -> Result<(axum::http::StatusCode, Vec<u8>), Box<dyn std::error::Error + Send + Sync>> {
  match client {
    TestClient::Http { client: reqwest_client, base_url } => {
      let url = format!("{}{}", base_url, path);
      let method = method.parse::<reqwest::Method>().unwrap_or(reqwest::Method::GET);
      let mut req = reqwest_client.request(method, &url);
      if let Some(t) = token {
        req = req.header("authorization", format!("Bearer {}", t));
      }
      if let Some(headers) = extra_headers {
        for (k, v) in headers.iter() {
          req = req.header(*k, *v);
        }
      }
      let res = if let Some(b) = body {
        req.header("content-type", "application/json").body(b.to_string()).send().await?
      } else {
        req.send().await?
      };
      let status = axum::http::StatusCode::from_u16(res.status().as_u16())
        .unwrap_or(axum::http::StatusCode::INTERNAL_SERVER_ERROR);
      let bytes = res.bytes().await?.to_vec();
      Ok((status, bytes))
    }
    TestClient::InProcess { router, .. } => {
      use axum::body::Body;
      use axum::http::Request;
      use tower::util::ServiceExt;
      let mut builder = Request::builder().method(method).uri(path);
      if let Some(t) = token {
        builder = builder.header("authorization", format!("Bearer {}", t));
      }
      if let Some(headers) = extra_headers {
        for (k, v) in headers.iter() {
          builder = builder.header(*k, *v);
        }
      }
      let req = if let Some(b) = body {
        builder.header("content-type", "application/json").body(Body::from(b.to_string()))?
      } else {
        builder.body(Body::empty())?
      };
      let res = router.clone().oneshot(req).await?;
      let status = res.status();
      let bytes = axum::body::to_bytes(res.into_body(), usize::MAX).await?;
      Ok((status, bytes.to_vec()))
    }
  }
}

/// Log in as a seed user via POST /api/auth/login; returns the Bearer token.
/// Use [test_client] (uses E2E_API_URL when set, else in-process router). Seed users: admin@admin.com, viewer@default.org, etc.
#[cfg(any(test, feature = "test-utils"))]
pub async fn login_as_seed_user(
  client: &TestClient,
  email: &str,
  password: &str,
) -> Result<String, Box<dyn std::error::Error + Send + Sync>> {
  login_as_seed_user_impl(client, email, password).await
}

/// Test helper: run one request and return status and body. Used by integration tests.
/// Pass optional `extra_headers` for scope (e.g. `[("X-Organization-Id", org_id), ("X-Role-Id", role_id)]`) when calling dashboard APIs.
#[cfg(any(test, feature = "test-utils"))]
pub async fn test_request(
  client: &TestClient,
  method: &str,
  path: &str,
  token: Option<&str>,
  body: Option<&str>,
  extra_headers: Option<&[(&str, &str)]>,
) -> Result<(axum::http::StatusCode, Vec<u8>), Box<dyn std::error::Error + Send + Sync>> {
  test_request_impl(client, method, path, token, body, extra_headers).await
}

/// Log in and return (token, org_id, role_id) from the first profile. Use for dashboard tests that need scope headers (X-Organization-Id, X-Role-Id).
#[cfg(any(test, feature = "test-utils"))]
pub async fn auth_with_profile(
  client: &TestClient,
  email: &str,
  password: &str,
) -> Result<(String, String, String), Box<dyn std::error::Error + Send + Sync>> {
  let token = login_as_seed_user_impl(client, email, password).await?;
  let (status, body) = test_request_impl(client, "GET", "/api/auth/profiles", Some(&token), None, None).await?;
  if status != axum::http::StatusCode::OK {
    let msg = String::from_utf8_lossy(&body);
    return Err(format!("GET /api/auth/profiles failed {}: {}", status, msg).into());
  }
  let json: serde_json::Value = serde_json::from_slice(&body)?;
  let profiles = json["profiles"].as_array().ok_or("profiles array missing")?;
  let first = profiles.first().ok_or("no profiles")?;
  let org_id = first["org_id"].as_str().ok_or("org_id missing")?;
  let role_id = first["role_id"].as_str().ok_or("role_id missing")?;
  Ok((token, org_id.to_string(), role_id.to_string()))
}
