use std::time::Duration;

use axum::response::{Html, IntoResponse};
use axum::routing::get;
use db::auth::Backend;
use forge::{App, CronSchedule};
use tower_http::services::ServeDir;

mod handlers;

#[tokio::main]
async fn main() -> Result<(), Box<dyn std::error::Error + Send + Sync>> {
  forge::init_tracing();
  let app = App::new()
    .with_migrations(db::Migrator)
    .with_seed(|db| Box::pin(db::run_seeds(db)))
    .with_auth(|db| Backend::new(db))
    .with_token_auth(db::token_lookup)
    .with_cron(
      "heartbeat",
      CronSchedule::Interval(Duration::from_secs(60)),
      |_db| async move { Ok(()) },
    );

  let app = if forge::config::effective_environment().eq_ignore_ascii_case("production") {
    app.with_rate_limit_per_ip(60).with_rate_limit_per_user(60)
  } else {
    app
  };

  let is_production = forge::config::effective_environment().eq_ignore_ascii_case("production");
  let host = app.config().server.host.clone();
  let port = app.config().server.port;

  let app = app
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
    .route("/api/dashboard/permissions", get(handlers::dashboard::list_permissions))
    .route(
      "/api/dashboard/role-permissions",
      get(handlers::dashboard::list_role_permissions)
        .post(handlers::dashboard::add_role_permission)
        .delete(handlers::dashboard::delete_role_permission),
    );

  let (router, db_conn, cron_runner, response_cache) = app.into_router_before_state().await;

  let api_router = router.with_state(db_conn.clone());

  let mut router = api_router;

  if let Some(cache_layer) = response_cache {
    router = router.layer(cache_layer);
  }

  if is_production {
    let dist = std::path::Path::new("frontend/dist");
    if dist.join("index.html").exists() {
      router = router
        .nest_service("/assets", ServeDir::new("frontend/dist/assets"))
        .fallback(get(serve_spa_index));
    }
  }

  if let Some((db, runner)) = cron_runner {
    runner.spawn(db);
  }

  let addr = format!("{}:{}", host, port);
  let listener = tokio::net::TcpListener::bind(&addr).await?;
  tracing::info!(target: "forge::app", "HTTP server listening on http://{}", addr);
  axum::serve(
    listener,
    router.into_make_service_with_connect_info::<std::net::SocketAddr>(),
  )
  .with_graceful_shutdown(forge::app::shutdown_signal_future())
  .await?;
  Ok(())
}

/// Serves frontend/dist/index.html for SPA fallback (production only).
async fn serve_spa_index() -> impl axum::response::IntoResponse {
  match tokio::fs::read("frontend/dist/index.html").await {
    Ok(html) => Html(html).into_response(),
    Err(_) => (
      axum::http::StatusCode::NOT_FOUND,
      "Not found. Run the frontend build.",
    )
      .into_response(),
  }
}
