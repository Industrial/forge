use axum::response::{Html, IntoResponse};
use axum::routing::get;
use tower_http::services::ServeDir;

use app::make_app;

#[tokio::main]
async fn main() -> Result<(), Box<dyn std::error::Error + Send + Sync>> {
  forge_app::init_tracing();
  let live_backend = std::sync::Arc::new(forge_live::InMemoryLiveBackend::new());
  let app = make_app(live_backend.clone())
    .with_migrations(migrations::Migrator)
    .with_seed(|db| {
      Box::pin(async move {
        migrations::run_seeds(db)
          .await
          .map_err(|e| -> Box<dyn std::error::Error> { e.to_string().into() })
      })
    });

  let env = forge_config::effective_environment_from_config(app.config());
  let app = if env.eq_ignore_ascii_case("production") {
    app.with_rate_limit_per_ip(60).with_rate_limit_per_user(60)
  } else {
    app
  };

  let is_production = env.eq_ignore_ascii_case("production");
  let host = app.config().server.host.clone();
  let port = app.config().server.port;

  let (router, db_conn, cron_runner, response_cache) = app.into_router_before_state().await;

  let task_state = std::sync::Arc::new(app::tasks::TaskState::new());
  {
    let task_state = task_state.clone();
    let live_backend = live_backend.clone();
    tokio::spawn(async move {
      let mut interval = tokio::time::interval(std::time::Duration::from_secs(8));
      loop {
        interval.tick().await;
        task_state.tick(&live_backend).await;
      }
    });
  }

  let subscription_store = forge_live::SubscriptionStore::new();
  subscription_store.spawn_change_worker();
  let api_router = router
    .with_state(db_conn.clone())
    .layer(axum::extract::Extension(db_conn.clone()))
    .layer(axum::extract::Extension(task_state.clone()))
    .layer(axum::extract::Extension(subscription_store));

  let mut router = api_router;

  if let Some(cache_layer) = response_cache {
    router = router.layer(cache_layer);
  }

  if is_production {
    let dist = std::path::Path::new("frontend/dist");
    if dist.join("index.html").exists() {
      router = router
        .nest_service("/assets", ServeDir::new("frontend/dist/assets"))
        .fallback(get(serve_spa_or_404));
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
  .with_graceful_shutdown(forge_app::shutdown_signal_future())
  .await?;
  Ok(())
}

/// Fallback: serve SPA for non-API paths only. /api/* must not get SPA (200) so API routes can return 401/403.
async fn serve_spa_or_404(req: axum::extract::Request) -> impl axum::response::IntoResponse {
  if req.uri().path().starts_with("/api") {
    return (axum::http::StatusCode::NOT_FOUND, "Not Found").into_response();
  }
  match tokio::fs::read("frontend/dist/index.html").await {
    Ok(html) => Html(html).into_response(),
    Err(_) => (
      axum::http::StatusCode::NOT_FOUND,
      "Not found. Run the frontend build.",
    )
      .into_response(),
  }
}
