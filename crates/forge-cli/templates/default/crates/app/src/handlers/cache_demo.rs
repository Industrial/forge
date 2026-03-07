use axum::{extract::State, response::IntoResponse};
use forge_cache::AppCache;
use forge_db::DbConnection;
use std::sync::Arc;

const CACHE_KEY: &str = "demo";

pub async fn handler(
  State(_db): State<DbConnection>,
  cache: Option<axum::extract::Extension<Option<Arc<AppCache>>>>,
) -> impl IntoResponse {
  tracing::debug!(target: "app::handlers", "route: GET /api/cache-demo");
  let value = if let Some(c) = cache.as_ref().and_then(|e| e.0.as_ref()) {
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

#[cfg(test)]
mod bdd_tests {
  use super::*;
  use crate::build_router_for_test;
  use axum::body::Body;
  use axum::http::{Request, StatusCode};
  use tower::ServiceExt;

  /// BDD-style tests focusing on behavior rather than implementation.
  /// Tests verify cache demo handler behavior with and without cache enabled.

  mod cache_enabled_behavior {
    use super::*;

    #[tokio::test]
    async fn should_return_cached_value_on_subsequent_requests() {
      // Given: a router with cache enabled
      let (router, _guard) = build_router_for_test().await.unwrap();
      let cache_config = forge_config::CacheConfig {
        enabled: true,
        application: Some(forge_config::ApplicationCacheConfig {
          enabled: true,
          max_capacity: 1000,
          default_ttl_secs: 300,
        }),
        http_response: None,
      };
      let cache = forge_cache::AppCache::from_config(&cache_config).unwrap();
      let router = router.layer(axum::extract::Extension(Some(Arc::new(cache))));

      // When: making first request to cache demo endpoint
      let req1 = Request::builder()
        .uri("/api/cache-demo")
        .body(Body::empty())
        .unwrap();
      let res1 = router.clone().oneshot(req1).await.unwrap();

      // Then: should return a cached value (first request generates it)
      assert_eq!(res1.status(), StatusCode::OK);
      let body1 = axum::body::to_bytes(res1.into_body(), usize::MAX)
        .await
        .unwrap();
      let value1 = std::str::from_utf8(&body1).unwrap();
      assert!(
        value1.starts_with("cached-"),
        "Should return cached value: {}",
        value1
      );

      // When: making second request to same endpoint
      let req2 = Request::builder()
        .uri("/api/cache-demo")
        .body(Body::empty())
        .unwrap();
      let res2 = router.oneshot(req2).await.unwrap();

      // Then: should return the same cached value (not generate new UUID)
      assert_eq!(res2.status(), StatusCode::OK);
      let body2 = axum::body::to_bytes(res2.into_body(), usize::MAX)
        .await
        .unwrap();
      let value2 = std::str::from_utf8(&body2).unwrap();
      assert_eq!(
        value1, value2,
        "Should return same cached value on second request"
      );
    }

    #[tokio::test]
    async fn should_generate_new_value_when_cache_miss() {
      // Given: a router with cache enabled but empty cache
      let (router, _guard) = build_router_for_test().await.unwrap();
      let cache_config = forge_config::CacheConfig {
        enabled: true,
        application: Some(forge_config::ApplicationCacheConfig {
          enabled: true,
          max_capacity: 1000,
          default_ttl_secs: 300,
        }),
        http_response: None,
      };
      let cache = Arc::new(forge_cache::AppCache::from_config(&cache_config).unwrap());
      let router = router.layer(axum::extract::Extension(Some(cache.clone())));

      // When: making request to cache demo endpoint (cache miss)
      let req = Request::builder()
        .uri("/api/cache-demo")
        .body(Body::empty())
        .unwrap();
      let res = router.oneshot(req).await.unwrap();

      // Then: should generate and cache a new UUID-based value
      assert_eq!(res.status(), StatusCode::OK);
      let body = axum::body::to_bytes(res.into_body(), usize::MAX)
        .await
        .unwrap();
      let value = std::str::from_utf8(&body).unwrap();
      assert!(
        value.starts_with("cached-"),
        "Should generate cached value: {}",
        value
      );

      // And: value should be stored in cache
      let cached_value = cache.get("demo").await;
      assert_eq!(
        cached_value,
        Some(value.to_string()),
        "Value should be cached"
      );
    }
  }

  mod cache_disabled_behavior {
    use super::*;

    #[tokio::test(flavor = "multi_thread")]
    async fn should_return_cache_disabled_message_when_cache_not_provided() {
      // Given: a router without cache extension
      let (router, _guard) = build_router_for_test().await.unwrap();

      // When: making request to cache demo endpoint
      let req = Request::builder()
        .uri("/api/cache-demo")
        .body(Body::empty())
        .unwrap();
      let res = router.oneshot(req).await.unwrap();

      // Then: should return "cache-disabled" message
      assert_eq!(res.status(), StatusCode::OK);
      let body = axum::body::to_bytes(res.into_body(), usize::MAX)
        .await
        .unwrap();
      let value = std::str::from_utf8(&body).unwrap();
      assert_eq!(
        value, "cache-disabled",
        "Should return cache-disabled when cache not available"
      );
    }

    #[tokio::test]
    async fn should_return_cache_disabled_when_cache_extension_is_none() {
      // Given: a router with cache extension set to None
      let (router, _guard) = build_router_for_test().await.unwrap();
      let router = router.layer(axum::extract::Extension(None::<Arc<AppCache>>));

      // When: making request to cache demo endpoint
      let req = Request::builder()
        .uri("/api/cache-demo")
        .body(Body::empty())
        .unwrap();
      let res = router.oneshot(req).await.unwrap();

      // Then: should return "cache-disabled" message
      assert_eq!(res.status(), StatusCode::OK);
      let body = axum::body::to_bytes(res.into_body(), usize::MAX)
        .await
        .unwrap();
      let value = std::str::from_utf8(&body).unwrap();
      assert_eq!(
        value, "cache-disabled",
        "Should return cache-disabled when cache is None"
      );
    }
  }

  mod cache_consistency_behavior {
    use super::*;

    #[tokio::test]
    async fn should_maintain_consistent_value_across_multiple_requests() {
      // Given: a router with cache enabled
      let (router, _guard) = build_router_for_test().await.unwrap();
      let cache_config = forge_config::CacheConfig {
        enabled: true,
        application: Some(forge_config::ApplicationCacheConfig {
          enabled: true,
          max_capacity: 1000,
          default_ttl_secs: 300,
        }),
        http_response: None,
      };
      let cache = Arc::new(forge_cache::AppCache::from_config(&cache_config).unwrap());
      let router = router.layer(axum::extract::Extension(Some(cache)));

      // When: making multiple requests to cache demo endpoint
      let mut values = Vec::new();
      for _ in 0..5 {
        let req = Request::builder()
          .uri("/api/cache-demo")
          .body(Body::empty())
          .unwrap();
        let res = router.clone().oneshot(req).await.unwrap();
        let body = axum::body::to_bytes(res.into_body(), usize::MAX)
          .await
          .unwrap();
        let value = std::str::from_utf8(&body).unwrap().to_string();
        values.push(value);
      }

      // Then: all requests should return the same cached value
      let first_value = &values[0];
      for (i, value) in values.iter().enumerate() {
        assert_eq!(
          value, first_value,
          "Request {} should return same cached value, got: {}",
          i, value
        );
      }
    }
  }
}
