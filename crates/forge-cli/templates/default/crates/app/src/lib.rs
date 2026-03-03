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
pub mod permissions;
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
    .post_route("/api/auth/switch-profile", handlers::auth::set_profile)
    .post_route("/api/auth/set-profile", handlers::auth::set_profile)
    .route("/api/auth/session", handlers::auth::session_json)
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
      "/api/users/:id",
      axum::routing::get(handlers::rest::get_user)
        .patch(handlers::rest::update_user)
        .delete(handlers::rest::delete_user),
    )
    .route("/api/users/:id/organizations", axum::routing::get(handlers::rest::get_user_organizations))
    .route_methods(
      "/api/organizations",
      axum::routing::get(handlers::rest::list_organizations).post(handlers::rest::create_organization),
    )
    .route_methods(
      "/api/organizations/:id",
      axum::routing::get(handlers::rest::get_organization)
        .patch(handlers::rest::update_organization)
        .delete(handlers::rest::delete_organization),
    )
    .route_methods(
      "/api/organizations/:id/users",
      axum::routing::get(handlers::rest::list_org_users).post(handlers::rest::add_org_user),
    )
    .route(
      "/api/organizations/:id/users/:user_id/roles",
      axum::routing::post(handlers::rest::add_org_user_roles),
    )
    .route_methods(
      "/api/organizations/:id/users/:user_id",
      axum::routing::get(handlers::rest::get_org_user)
        .patch(handlers::rest::update_org_user)
        .delete(handlers::rest::delete_org_user),
    )
    .route_methods(
      "/api/organizations/:id/roles",
      axum::routing::get(handlers::rest::list_org_roles).post(handlers::rest::create_org_role),
    )
    .route_methods(
      "/api/organizations/:id/roles/:role_id/permissions",
      axum::routing::get(handlers::rest::list_org_role_permissions)
        .post(handlers::rest::add_org_role_permission)
        .delete(handlers::rest::delete_org_role_permission),
    )
    .route_methods(
      "/api/organizations/:id/roles/:role_id",
      axum::routing::get(handlers::rest::get_org_role)
        .patch(handlers::rest::update_org_role)
        .delete(handlers::rest::delete_org_role),
    )
    .route("/api/audit-log", axum::routing::get(handlers::rest::list_audit_log))
    .route("/api/audit-log/:id", axum::routing::get(handlers::rest::get_audit_log))
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
  // Use a unique in-memory DB name per test so parallel tests do not share the same DB (avoids UNIQUE constraint on seaql_migrations).
  let db_toml = format!(
    r#"[database]
url = "sqlite:file:{}?mode=memory&cache=shared"
auto_migrate = true
auto_seed = true
"#,
    db_name
  );
  std::fs::write(config_dir.join("db.toml"), db_toml)?;
  Ok(())
}

/// Build the API router for integration tests: temp directory with config, in-memory DB, migrations and seed run.
/// Returns the router and a guard; keep the guard for the test duration so CWD is restored on drop.
pub async fn build_router_for_test(
) -> Result<(Router, TestEnvGuard), Box<dyn std::error::Error + Send + Sync>> {
  let temp = TempDir::new()?;
  let original_cwd = std::env::current_dir()?;
  let db_name = format!("testdb_{}", uuid::Uuid::new_v4());
  write_test_config(temp.path(), &db_name)?;
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
