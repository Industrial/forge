use std::time::Duration;

use axum::routing::get;
use axum_inertia::{vite, InertiaConfig};
use axum_login::AuthManagerLayerBuilder;
use db::auth::Backend;
use forge::{App, CronSchedule};
use sea_orm::{ConnectionTrait, DbBackend};
use tower_http::services::ServeDir;
use tower_sessions::{Expiry, SessionManagerLayer};
use tower_sessions_sqlx_store::SqliteStore;

mod handlers;
mod state;

use state::AppState;

#[tokio::main]
async fn main() -> Result<(), Box<dyn std::error::Error + Send + Sync>> {
  forge::init_tracing();
  let app = App::new()
    .with_migrations(db::Migrator)
    .with_seed(|db| Box::pin(db::run_seeds(db)))
    .with_auth(|db| Backend::new(db))
    .with_token_auth(db::token_lookup)
    .with_cron("heartbeat", CronSchedule::Interval(Duration::from_secs(60)), |_db| async move { Ok(()) });

  let app = if forge::config::effective_environment().eq_ignore_ascii_case("production") {
    app
      .with_rate_limit_per_ip(60)
      .with_rate_limit_per_user(60)
  } else {
    app
  };

  let is_production = forge::config::effective_environment().eq_ignore_ascii_case("production");
  let host = app.config().server.host.clone();
  let port = app.config().server.port;
  let frontend_port = app.config().frontend.port;

  let app = app
    .route("/api/cache-demo", handlers::cache_demo::handler)
    .route("/api/cached-page", handlers::cached_page::handler)
    .route("/api/observability/trace-id", handlers::observability::trace_id)
    .route("/ws", handlers::ws::handler)
    .post_route("/api/auth/register", handlers::auth::register)
    .post_route("/api/auth/login", handlers::auth::login)
    .route("/api/auth/logout", handlers::auth::logout)
    .route("/api/auth/profile", handlers::auth::profile)
    .post_route("/api/auth/tokens", handlers::auth::create_token)
    .route("/api/auth/admin", handlers::auth::admin_only);

  let (router, db_conn, cron_runner, response_cache) = app.into_router_before_state().await;

  let inertia: InertiaConfig = if is_production {
    vite::Production::new("frontend/dist/.vite/manifest.json", "src/main.tsx")
      .map_err(|e| format!("Inertia production config: {}", e))?
      .lang("en")
      .title("App")
      .into_config()
  } else {
    vite::Development::default()
      .port(frontend_port)
      .main("src/main.tsx")
      .lang("en")
      .title("App")
      .react()
      .into_config()
  };

  let api_router = router.with_state(db_conn.clone());
  let app_state = AppState {
    db: db_conn.clone(),
    inertia,
  };

  // Inertia page routes need the same auth+session stack as API routes so OptionalRequireAuth
  // and Session work (same session store and cookie). We build the same session + auth layer
  // and apply it to the inertia router so both routers share the session (same DB, same cookie).
  let inertia_routes = axum::Router::new()
    .route("/", get(handlers::inertia::home))
    .route("/login", get(handlers::inertia::login_page))
    .route("/register", get(handlers::inertia::register_page))
    .route("/dashboard", get(handlers::inertia::dashboard))
    .route("/ws-demo", get(handlers::inertia::ws_demo_page))
    .route("/photos", get(handlers::inertia::photos_page))
    .route("/ideas", get(handlers::inertia::ideas_page))
    .route("/files", get(handlers::inertia::files_page))
    .with_state(app_state);

  let inertia_router = if db_conn.get_database_backend() == DbBackend::Sqlite {
    let pool = db_conn.inner().get_sqlite_connection_pool();
    let session_store = SqliteStore::new(pool.clone());
    if let Err(e) = session_store.migrate().await {
      tracing::warn!("Session store migrate for inertia routes: {}", e);
    }
    let secure = !forge::config::effective_environment().eq_ignore_ascii_case("development");
    let session_layer = SessionManagerLayer::new(session_store)
      .with_http_only(true)
      .with_secure(secure)
      .with_same_site(tower_sessions::cookie::SameSite::Lax)
      .with_path("/")
      .with_expiry(Expiry::OnInactivity(
        tower_sessions::cookie::time::Duration::days(30),
      ));
    let backend = Backend::new(db_conn.clone());
    let auth_layer = AuthManagerLayerBuilder::new(backend, session_layer).build();
    inertia_routes.layer(auth_layer)
  } else {
    inertia_routes
  };

  let mut router = api_router.merge(inertia_router);

  if let Some(cache_layer) = response_cache {
    router = router.layer(cache_layer);
  }

  if is_production {
    router = router.nest_service("/assets", ServeDir::new("frontend/dist/assets"));
  }

  if let Some((db, runner)) = cron_runner {
    runner.spawn(db);
  }

  let addr = format!("{}:{}", host, port);
  let listener = tokio::net::TcpListener::bind(&addr).await?;
  tracing::info!(target: "forge::app", "HTTP server listening on http://{}", addr);
  axum::serve(listener, router.into_make_service_with_connect_info::<std::net::SocketAddr>())
    .with_graceful_shutdown(forge::app::shutdown_signal_future())
    .await?;
  Ok(())
}
