//! Library for the template app: route registration and test harness.
//!
//! Use [make_app] (with CWD set to a directory containing `config/app.toml` and `config/db.toml`)
//! or [build_router_for_test] for integration tests (temp config + in-memory DB).

pub use serde_json;

use std::path::PathBuf;
use std::sync::Arc;
use std::time::Duration;

use axum::Router;
use db::auth::Backend;
use forge::{App, CronSchedule};
use tempfile::TempDir;

pub mod handlers;
pub mod tasks;

/// Build the Forge [App] with all template routes. Requires CWD to be a directory that contains
/// `config/app.toml` and `config/db.toml` (e.g. project root or a temp dir from [build_router_for_test]).
pub fn make_app(live_backend: Arc<forge::live::InMemoryLiveBackend>) -> App {
  let app = App::new()
    .with_migrations(db::Migrator)
    .with_seed(|db| Box::pin(db::run_seeds(db)))
    .with_auth(|db| Backend::new(db))
    .with_token_auth(db::token_lookup)
    .with_cron(
      "heartbeat",
      CronSchedule::Interval(Duration::from_secs(60)),
      |_db| async move { Ok(()) },
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
    .route("/api/auth/profile", handlers::auth::profile)
    .route("/api/auth/profiles", handlers::auth::profiles_list)
    .post_route("/api/auth/switch-profile", handlers::auth::switch_profile)
    .route("/api/auth/session", handlers::auth::session_json)
    .post_route("/api/auth/tokens", handlers::auth::create_token)
    .route("/api/auth/admin", handlers::auth::admin_only)
    .route("/api/dashboard/permissions", axum::routing::get(handlers::dashboard::list_permissions))
    .route_methods(
      "/api/dashboard/role-permissions",
      axum::routing::get(handlers::dashboard::list_role_permissions)
        .post(handlers::dashboard::add_role_permission)
        .delete(handlers::dashboard::delete_role_permission),
    )
    .route("/api/dashboard/tasks", axum::routing::get(handlers::dashboard::list_tasks))
    .route("/api/dashboard/audit-log", axum::routing::get(handlers::dashboard::list_audit_log))
    .route_methods(
      "/api/dashboard/organizations",
      axum::routing::get(handlers::dashboard::list_organizations)
        .post(handlers::dashboard::create_organization)
        .patch(handlers::dashboard::update_organization)
        .delete(handlers::dashboard::delete_organization),
    )
    .route_methods(
      "/api/dashboard/users",
      axum::routing::get(handlers::dashboard::list_users)
        .post(handlers::dashboard::create_user)
        .patch(handlers::dashboard::update_user)
        .delete(handlers::dashboard::delete_user),
    )
    .route_methods(
      "/api/dashboard/roles",
      axum::routing::get(handlers::dashboard::list_roles)
        .post(handlers::dashboard::create_role)
        .patch(handlers::dashboard::update_role)
        .delete(handlers::dashboard::delete_role),
    )
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

/// Write `config/app.toml` and `config/db.toml` under `dir` for integration tests (in-memory DB, auto_migrate, auto_seed).
fn write_test_config(dir: &std::path::Path) -> Result<(), Box<dyn std::error::Error + Send + Sync>> {
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
  // Use shared in-memory SQLite so the connection pool (migrations + seed) share the same DB.
  std::fs::write(
    config_dir.join("db.toml"),
    r#"[database]
url = "sqlite:file:testdb?mode=memory&cache=shared"
auto_migrate = true
auto_seed = true
"#,
  )?;
  Ok(())
}

/// Build the API router for integration tests: temp directory with config, in-memory DB, migrations and seed run.
/// Returns the router and a guard; keep the guard for the test duration so CWD is restored on drop.
pub async fn build_router_for_test(
) -> Result<(Router, TestEnvGuard), Box<dyn std::error::Error + Send + Sync>> {
  let temp = TempDir::new()?;
  let original_cwd = std::env::current_dir()?;
  write_test_config(temp.path())?;
  std::env::set_current_dir(temp.path())?;

  let guard = TestEnvGuard {
    original_cwd,
    _temp: temp,
  };

  forge::init_tracing();
  let live_backend = Arc::new(forge::live::InMemoryLiveBackend::new());
  let app = make_app(live_backend.clone());

  let (router, db_conn, _cron_runner, response_cache) = app.into_router_before_state().await;

  let task_state = Arc::new(tasks::TaskState::new());
  let api_router = router
    .with_state(db_conn)
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

/// Log in as a seed user via POST /api/auth/login; returns the session cookie value to use as the `Cookie` header.
/// Use with [build_router_for_test]. Seed users: admin@admin.com, viewer@default.org, editor@default.org, etc.
pub async fn login_as_seed_user(
  router: &axum::Router,
  email: &str,
  password: &str,
) -> Result<String, Box<dyn std::error::Error + Send + Sync>> {
  use axum::body::Body;
  use axum::http::{Request, StatusCode};
  use tower::util::ServiceExt;

  let body = serde_json::json!({ "email": email, "password": password }).to_string();
  let req = Request::builder()
    .method("POST")
    .uri("/api/auth/login")
    .header("content-type", "application/json")
    .body(Body::from(body))?;
  let res = router.clone().oneshot(req).await?;
  let status = res.status();
  if status != StatusCode::OK {
    let bytes = axum::body::to_bytes(res.into_body(), usize::MAX).await?;
    let msg = String::from_utf8_lossy(&bytes);
    return Err(format!("login failed {}: {}", status, msg).into());
  }
  let headers = res.headers();
  let cookie = headers
    .get_all("set-cookie")
    .iter()
    .filter_map(|v| v.to_str().ok())
    .collect::<Vec<_>>()
    .join("; ");
  if cookie.is_empty() {
    return Err("login succeeded but no Set-Cookie in response".into());
  }
  Ok(cookie)
}

/// Test helper: run one request and return status and body. Used by integration tests.
pub async fn test_request(
  router: &axum::Router,
  method: &str,
  path: &str,
  cookie: Option<&str>,
  body: Option<&str>,
) -> Result<(axum::http::StatusCode, Vec<u8>), Box<dyn std::error::Error + Send + Sync>> {
  use axum::body::Body;
  use axum::http::Request;
  use tower::util::ServiceExt;

  let mut builder = Request::builder().method(method).uri(path);
  if let Some(c) = cookie {
    builder = builder.header("cookie", c);
  }
  let req = if let Some(b) = body {
    builder
      .header("content-type", "application/json")
      .body(Body::from(b.to_string()))?
  } else {
    builder.body(Body::empty())?
  };
  let res = router.clone().oneshot(req).await?;
  let status = res.status();
  let bytes = axum::body::to_bytes(res.into_body(), usize::MAX).await?;
  Ok((status, bytes.to_vec()))
}
