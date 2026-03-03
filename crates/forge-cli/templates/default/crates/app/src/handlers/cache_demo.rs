use axum::{extract::State, response::IntoResponse};
use forge_cache::AppCache;
use forge_db::DbConnection;
use std::sync::Arc;

const CACHE_KEY: &str = "demo";

pub async fn handler(
  State(_db): State<DbConnection>,
  cache: axum::extract::Extension<Option<Arc<AppCache>>>,
) -> impl IntoResponse {
  tracing::debug!(target: "app::handlers", "route: GET /api/cache-demo");
  let value = if let Some(c) = cache.0.as_ref() {
    if let Some(v) = c.get(CACHE_KEY).await {
      v
    } else {
      let v = format!("cached-{}", uuid::Uuid::new_v4());
      c.set(CACHE_KEY, v.clone()).await;
      v
    }
  } else {
    "cache-disabled".to_string()
  };
  value.into_response()
}
