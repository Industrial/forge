//! HTTP response cache layer: caches full GET responses in Moka; adds Cache-Control.

use std::future::Future;
use std::pin::Pin;
use std::sync::Arc;
use std::time::Duration;

use axum::body::{Body, to_bytes};
use bytes::Bytes;
use http::{HeaderMap, HeaderValue, Method, Request, Response, StatusCode};
use moka::future::Cache;
use tower::{Layer, Service};
use tracing::debug;

/// Cached response: status, headers, body. Stored in Moka for GET response cache.
#[derive(Clone)]
struct CachedResponse {
  /// HTTP status code of the cached response.
  status: StatusCode,
  /// Response headers (including Cache-Control when stored).
  headers: HeaderMap,
  /// Response body bytes.
  body: Bytes,
}

/// Tower layer that caches full HTTP responses for GET requests.
#[derive(Clone)]
pub struct HttpResponseCacheLayer {
  /// Moka cache for path+query -> CachedResponse.
  cache: Arc<Cache<String, CachedResponse>>,
  /// TTL in seconds for cached entries.
  ttl_secs: u64,
  /// Paths (exact "/" or prefix) to skip for caching.
  no_cache_paths: Vec<String>,
}

impl HttpResponseCacheLayer {
  /// Build from config. Returns None if HTTP response cache is disabled.
  pub fn from_config(config: &crate::cache::CacheConfig) -> Option<Self> {
    if !config.enabled {
      return None;
    }
    let http = config.http_response.as_ref().filter(|h| h.enabled)?;
    let cache = Cache::builder()
      .max_capacity(10_000)
      .time_to_live(Duration::from_secs(http.default_ttl_secs))
      .build();
    let no_cache_paths = http.no_cache_paths.clone().unwrap_or_default();
    Some(Self {
      cache: Arc::new(cache),
      ttl_secs: http.default_ttl_secs,
      no_cache_paths,
    })
  }

  /// Builds cache key from method (GET), path and optional query.
  fn cache_key(path: &str, query: Option<&str>) -> String {
    if let Some(q) = query {
      format!("GET:{}?{}", path, q)
    } else {
      format!("GET:{}", path)
    }
  }
}

impl<S> Layer<S> for HttpResponseCacheLayer {
  type Service = HttpResponseCacheService<S>;

  fn layer(&self, inner: S) -> Self::Service {
    HttpResponseCacheService {
      inner,
      cache: self.cache.clone(),
      ttl_secs: self.ttl_secs,
      no_cache_paths: self.no_cache_paths.clone(),
    }
  }
}

/// Service that wraps an inner service and caches GET responses.
#[derive(Clone)]
pub struct HttpResponseCacheService<S> {
  /// Inner axum service.
  inner: S,
  /// Shared Moka cache for responses.
  cache: Arc<Cache<String, CachedResponse>>,
  /// TTL in seconds for stored entries.
  ttl_secs: u64,
  /// Paths to exclude from caching.
  no_cache_paths: Vec<String>,
}

impl<S> HttpResponseCacheService<S> {
  /// Returns true if this path should not be cached (exact or prefix match).
  fn should_skip(&self, path: &str) -> bool {
    self.no_cache_paths.iter().any(|prefix| {
      if prefix == "/" {
        path == "/"
      } else {
        path.starts_with(prefix)
      }
    })
  }
}

impl<S, ReqBody> Service<Request<ReqBody>> for HttpResponseCacheService<S>
where
  S: Service<Request<ReqBody>, Response = Response<Body>> + Clone + Send + 'static,
  S::Future: Send + 'static,
  ReqBody: Send + 'static,
{
  type Response = Response<Body>;
  type Error = S::Error;
  type Future = Pin<Box<dyn Future<Output = Result<Self::Response, Self::Error>> + Send>>;

  fn poll_ready(
    &mut self,
    cx: &mut std::task::Context<'_>,
  ) -> std::task::Poll<Result<(), Self::Error>> {
    self.inner.poll_ready(cx)
  }

  fn call(&mut self, req: Request<ReqBody>) -> Self::Future {
    if req.method() != Method::GET {
      return Box::pin(self.inner.call(req));
    }

    let path = req.uri().path().to_string();
    if self.should_skip(&path) {
      return Box::pin(self.inner.call(req));
    }

    let query = req.uri().query().map(|q: &str| q.to_string());
    let key = HttpResponseCacheLayer::cache_key(&path, query.as_deref());

    let cache = self.cache.clone();
    let ttl_secs = self.ttl_secs;
    let mut inner = self.inner.clone();

    Box::pin(async move {
      if let Some(cached) = cache.get(&key).await {
        debug!("response cache hit: {}", key);
        let mut headers = cached.headers.clone();
        if !headers.contains_key("cache-control") {
          let _ = headers.insert(
            "cache-control",
            HeaderValue::try_from(format!("public, max-age={}", ttl_secs))
              .unwrap_or_else(|_| HeaderValue::from_static("public, max-age=60")),
          );
        }
        let response = Response::builder()
          .status(cached.status)
          .body(Body::from(cached.body))
          .unwrap();
        let (mut parts, body) = response.into_parts();
        parts.headers = headers;
        return Ok(Response::from_parts(parts, body));
      }

      let response: Response<Body> = inner.call(req).await?;
      let status = response.status();
      if !status.is_success() {
        return Ok(response);
      }

      let (parts, body) = response.into_parts();
      const BODY_LIMIT: usize = 10 * 1024 * 1024; // 10 MiB
      let body_bytes = to_bytes(body, BODY_LIMIT).await.map_err(|e| {
        // Map body error into service error if possible; otherwise log and return 500
        tracing::warn!("response cache: failed to read body: {}", e);
      });
      let body_bytes = match body_bytes {
        Ok(b) => b,
        Err(()) => {
          return Ok(
            Response::builder()
              .status(StatusCode::INTERNAL_SERVER_ERROR)
              .body(Body::empty())
              .unwrap(),
          );
        }
      };

      let cache_control = format!("public, max-age={}", ttl_secs);
      let mut headers = parts.headers.clone();
      let _ = headers.insert(
        "cache-control",
        HeaderValue::try_from(cache_control.as_str())
          .unwrap_or_else(|_| HeaderValue::from_static("public, max-age=60")),
      );

      let cached = CachedResponse {
        status: parts.status,
        headers: headers.clone(),
        body: body_bytes.clone(),
      };
      cache.insert(key.clone(), cached).await;
      debug!("response cache stored: {}", key);

      let mut response = Response::builder()
        .status(parts.status)
        .version(parts.version)
        .body(Body::from(body_bytes))
        .unwrap();
      *response.headers_mut() = headers;
      Ok(response)
    })
  }
}

#[cfg(test)]
mod tests {
  use super::*;
  use crate::cache::{CacheConfig, HttpResponseCacheConfig};

  #[test]
  fn from_config_disabled_returns_none() {
    let cfg = CacheConfig {
      enabled: false,
      application: None,
      http_response: Some(HttpResponseCacheConfig {
        enabled: true,
        default_ttl_secs: 60,
        no_cache_paths: None,
      }),
    };
    assert!(HttpResponseCacheLayer::from_config(&cfg).is_none());
  }

  #[test]
  fn from_config_no_http_response_returns_none() {
    let cfg = CacheConfig {
      enabled: true,
      application: None,
      http_response: None,
    };
    assert!(HttpResponseCacheLayer::from_config(&cfg).is_none());
  }

  #[test]
  fn from_config_http_disabled_returns_none() {
    let cfg = CacheConfig {
      enabled: true,
      application: None,
      http_response: Some(HttpResponseCacheConfig {
        enabled: false,
        default_ttl_secs: 60,
        no_cache_paths: None,
      }),
    };
    assert!(HttpResponseCacheLayer::from_config(&cfg).is_none());
  }

  #[test]
  fn from_config_enabled_returns_some() {
    let cfg = CacheConfig {
      enabled: true,
      application: None,
      http_response: Some(HttpResponseCacheConfig {
        enabled: true,
        default_ttl_secs: 120,
        no_cache_paths: Some(vec!["/healthz".into()]),
      }),
    };
    let layer = HttpResponseCacheLayer::from_config(&cfg).unwrap();
    let _ = layer;
  }
}
