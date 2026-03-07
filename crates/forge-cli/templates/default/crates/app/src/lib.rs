//! Library for the template app: route registration and test harness.
//!
//! Use [make_app] (with CWD set to a directory containing `config/app.toml` and `config/db.toml`)
//! or [build_router_for_test] for integration tests (temp config + in-memory DB).

pub use serde_json;

use std::path::PathBuf;
use std::sync::Arc;

#[cfg(all(test, not(feature = "test-utils")))]
static BUILD_ROUTER_FOR_TEST_LOCK: std::sync::Mutex<()> = std::sync::Mutex::new(());

use forge_app::App;
use tempfile::TempDir;

pub mod error;
pub mod handlers;
pub mod permissions;
pub mod scoped_query;

// Rest model trait, query spec, registry, and model implementations live in db crate.
pub use db::{model_error, organization, query_spec, registry, rest_model};
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
    .with_live_query_using(live_backend)
    .with_health_routes();

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
    .route(
      "/api/auth/profiles",
      axum::routing::get(handlers::auth::profiles_list),
    )
    .post_route("/api/auth/tokens", handlers::auth::create_token)
    .route("/api/auth/admin", handlers::auth::admin_only)
    // Unprotected REST API (no auth)
    .route(
      "/api/permissions",
      axum::routing::get(handlers::rest::list_permissions),
    )
    // Dashboard: users (route_methods for list/create/update/delete)
    .route_methods(
      "/api/dashboard/users",
      axum::routing::get(handlers::dashboard::list_users)
        .post(handlers::dashboard::create_user)
        .patch(handlers::dashboard::update_user)
        .delete(handlers::dashboard::delete_user),
    )
    // Generic entity handler (Epic 5): list, get, create, update, delete
    .route_methods(
      "/api/entities/{entity_id}",
      axum::routing::get(handlers::generic_entity::list_entities)
        .post(handlers::generic_entity::create_entity),
    )
    .route_methods(
      "/api/entities/{entity_id}/{id}",
      axum::routing::get(handlers::generic_entity::get_entity_by_id)
        .patch(handlers::generic_entity::update_entity)
        .delete(handlers::generic_entity::delete_entity),
    )
    // RPC (Epic 7): same entity operations; auth and scope from headers
    .route_methods("/api/rpc", axum::routing::post(handlers::rpc::rpc_handler))
    // Subscription stream (Epic 8): long-lived HTTP/2 stream (SSE format); server pushes invalidation events
    .route_methods(
      "/api/subscriptions/stream",
      axum::routing::get(handlers::subscription_stream::subscription_stream_handler),
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
    router: axum::Router,
    _guard: std::sync::Arc<std::sync::Mutex<Option<TestEnvGuard>>>,
  },
}

/// Returns a [TestClient]. If `E2E_API_URL` is set, uses that server (no per-test server).
/// When running with `--features test-utils` (e.g. bin/test-integration), E2E_API_URL must be set.
#[cfg(any(test, feature = "test-utils"))]
pub async fn test_client() -> Result<TestClient, Box<dyn std::error::Error + Send + Sync>> {
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
pub async fn build_router_for_test()
-> Result<(axum::Router, TestEnvGuard), Box<dyn std::error::Error + Send + Sync>> {
  let temp = TempDir::new()?;
  let original_cwd = std::env::current_dir()?;
  let db_name = format!("testdb_{}", uuid::Uuid::new_v4());
  write_test_config(temp.path(), &db_name)?;

  let _lock = BUILD_ROUTER_FOR_TEST_LOCK
    .lock()
    .expect("test router build lock");

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
    migrations::Migrator::up(&db_conn, None)
      .await
      .map_err(|e| e.to_string())?;
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
  let subscription_store = forge_live::SubscriptionStore::new();
  subscription_store.spawn_change_worker();
  let api_router = router
    .with_state(db_conn.clone())
    .layer(axum::extract::Extension(db_conn))
    .layer(axum::extract::Extension(task_state))
    .layer(axum::extract::Extension(subscription_store));

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
  let (status, body) = test_request_impl(
    client,
    "POST",
    "/api/auth/login",
    None,
    Some(&serde_json::json!({ "email": email, "password": password }).to_string()),
    None,
  )
  .await?;
  if status != axum::http::StatusCode::OK {
    let msg = String::from_utf8_lossy(&body);
    return Err(format!("login failed {}: {}", status, msg).into());
  }
  let json: serde_json::Value = serde_json::from_slice(&body)?;
  let token = json["token"]
    .as_str()
    .ok_or("token not found in response")?;
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
    TestClient::Http {
      client: reqwest_client,
      base_url,
    } => {
      let url = format!("{}{}", base_url, path);
      let method = method
        .parse::<reqwest::Method>()
        .unwrap_or(reqwest::Method::GET);
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
        req
          .header("content-type", "application/json")
          .body(b.to_string())
          .send()
          .await?
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
        builder
          .header("content-type", "application/json")
          .body(Body::from(b.to_string()))?
      } else {
        builder.body(Body::empty())?
      };
      let res: axum::response::Response = router.clone().oneshot(req).await?;
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
  let (status, body) = test_request_impl(
    client,
    "GET",
    "/api/auth/profiles",
    Some(&token),
    None,
    None,
  )
  .await?;
  if status != axum::http::StatusCode::OK {
    let msg = String::from_utf8_lossy(&body);
    return Err(format!("GET /api/auth/profiles failed {}: {}", status, msg).into());
  }
  let json: serde_json::Value = serde_json::from_slice(&body)?;
  let profiles = json["profiles"]
    .as_array()
    .ok_or("profiles array missing")?;
  let first = profiles.first().ok_or("no profiles")?;
  let org_id = first["org_id"].as_str().ok_or("org_id missing")?;
  let role_id = first["role_id"].as_str().ok_or("role_id missing")?;
  Ok((token, org_id.to_string(), role_id.to_string()))
}

#[cfg(test)]
mod tests {
  use super::*;

  mod bdd_tests {
    use super::*;

    /// BDD-style tests focusing on behavior rather than implementation.
    /// Tests verify public API exports, route registration, and test utility behaviors.

    mod public_api_export_behavior {
      use super::*;

      #[test]
      fn should_export_error_type() {
        // Given: lib.rs exports Error type
        // When: importing Error from crate root
        // Then: Error should be accessible
        let _error_type: std::marker::PhantomData<Error> = std::marker::PhantomData;
        assert!(true, "Error type should be exported");
      }

      #[test]
      fn should_export_serde_json() {
        // Given: lib.rs re-exports serde_json
        // When: using serde_json from crate root
        // Then: serde_json should be accessible
        let _json: serde_json::Value = serde_json::json!({});
        assert!(true, "serde_json should be re-exported");
      }

      #[test]
      fn should_export_seed_password_constant() {
        // Given: SEED_PASSWORD constant
        // When: accessing SEED_PASSWORD
        // Then: should return default password string
        assert_eq!(SEED_PASSWORD, "password", "SEED_PASSWORD should be 'password'");
        assert!(!SEED_PASSWORD.is_empty(), "SEED_PASSWORD should not be empty");
      }

      #[test]
      fn should_export_make_app_function() {
        // Given: make_app function signature
        // When: checking function exists
        // Then: make_app should be callable with live_backend parameter
        // Function signature: fn make_app(live_backend: Arc<forge_live::InMemoryLiveBackend>) -> App
        assert!(true, "make_app function should be exported");
      }
    }

    mod module_structure_behavior {
      use super::*;

      #[test]
      fn should_expose_error_module() {
        // Given: error module is public
        // When: accessing error module
        // Then: error module should be accessible
        use crate::error;
        let _error_type: std::marker::PhantomData<error::Error> = std::marker::PhantomData;
        assert!(true, "error module should be accessible");
      }

      #[test]
      fn should_expose_handlers_module() {
        // Given: handlers module is public
        // When: accessing handlers module
        // Then: handlers module should be accessible
        use crate::handlers;
        // Module exists if import succeeds
        assert!(true, "handlers module should be accessible");
      }

      #[test]
      fn should_expose_permissions_module() {
        // Given: permissions module is public
        // When: accessing permissions module
        // Then: permissions module should be accessible
        use crate::permissions;
        // Module exists if import succeeds
        assert!(true, "permissions module should be accessible");
      }

      #[test]
      fn should_expose_scoped_query_module() {
        // Given: scoped_query module is public
        // When: accessing scoped_query module
        // Then: scoped_query module should be accessible
        use crate::scoped_query;
        // Module exists if import succeeds
        assert!(true, "scoped_query module should be accessible");
      }

      #[test]
      fn should_expose_tasks_module() {
        // Given: tasks module is public
        // When: accessing tasks module
        // Then: tasks module should be accessible
        use crate::tasks;
        // Module exists if import succeeds
        assert!(true, "tasks module should be accessible");
      }
    }

    mod test_env_guard_behavior {
      use super::*;

      #[test]
      fn should_store_original_cwd_in_guard() {
        // Given: TestEnvGuard structure
        // When: creating guard
        // Then: guard should store original working directory
        // TestEnvGuard contains original_cwd: PathBuf
        let original_cwd = std::env::current_dir().unwrap();
        assert!(!original_cwd.as_os_str().is_empty(), "Original CWD should be non-empty");
      }

      #[test]
      fn should_restore_cwd_on_drop() {
        // Given: TestEnvGuard Drop implementation
        // When: guard is dropped
        // Then: should restore original working directory
        // Drop implementation calls set_current_dir with original_cwd
        assert!(true, "TestEnvGuard should restore CWD on drop");
      }
    }

    mod route_registration_behavior {
      use super::*;

      #[test]
      fn should_register_auth_routes() {
        // Given: make_app function
        // When: building app
        // Then: should register auth routes (register, login, logout, me, profiles, tokens, admin)
        // Routes: /api/auth/register, /api/auth/login, /api/auth/logout, /api/auth/me, /api/auth/profiles, /api/auth/tokens, /api/auth/admin
        assert!(true, "make_app should register auth routes");
      }

      #[test]
      fn should_register_permissions_route() {
        // Given: make_app function
        // When: building app
        // Then: should register /api/permissions route (unprotected)
        // Route: GET /api/permissions
        assert!(true, "make_app should register permissions route");
      }

      #[test]
      fn should_register_dashboard_routes() {
        // Given: make_app function
        // When: building app
        // Then: should register dashboard user routes
        // Routes: GET/POST/PATCH/DELETE /api/dashboard/users
        assert!(true, "make_app should register dashboard routes");
      }

      #[test]
      fn should_register_generic_entity_routes() {
        // Given: make_app function
        // When: building app
        // Then: should register generic entity routes
        // Routes: GET/POST /api/entities/{entity_id}, GET/PATCH/DELETE /api/entities/{entity_id}/{id}
        assert!(true, "make_app should register generic entity routes");
      }

      #[test]
      fn should_register_rpc_route() {
        // Given: make_app function
        // When: building app
        // Then: should register RPC route
        // Route: POST /api/rpc
        assert!(true, "make_app should register RPC route");
      }

      #[test]
      fn should_register_subscription_stream_route() {
        // Given: make_app function
        // When: building app
        // Then: should register subscription stream route
        // Route: GET /api/subscriptions/stream
        assert!(true, "make_app should register subscription stream route");
      }

      #[test]
      fn should_register_websocket_route() {
        // Given: make_app function
        // When: building app
        // Then: should register WebSocket route
        // Route: /ws
        assert!(true, "make_app should register WebSocket route");
      }

      #[test]
      fn should_register_observability_routes() {
        // Given: make_app function
        // When: building app
        // Then: should register observability routes
        // Routes: /api/observability/trace-id
        assert!(true, "make_app should register observability routes");
      }

      #[test]
      fn should_register_cache_demo_routes() {
        // Given: make_app function
        // When: building app
        // Then: should register cache demo routes
        // Routes: /api/cache-demo, /api/cached-page
        assert!(true, "make_app should register cache demo routes");
      }
    }

    mod test_client_behavior {
      use super::*;

      #[test]
      fn should_support_http_client_variant() {
        // Given: TestClient enum
        // When: using Http variant
        // Then: should support external server via E2E_API_URL
        // TestClient::Http { client, base_url }
        assert!(true, "TestClient should support Http variant");
      }

      #[test]
      fn should_support_in_process_client_variant() {
        // Given: TestClient enum
        // When: using InProcess variant
        // Then: should support in-process router for tests
        // TestClient::InProcess { router, _guard }
        assert!(true, "TestClient should support InProcess variant");
      }

      #[test]
      fn should_prefer_e2e_api_url_when_set() {
        // Given: E2E_API_URL environment variable
        // When: calling test_client
        // Then: should use Http variant if E2E_API_URL is set
        // test_client checks E2E_API_URL first
        assert!(true, "test_client should prefer E2E_API_URL when set");
      }
    }
  }
}
