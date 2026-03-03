//! HTTP response cache layer: caches full GET responses in Moka.

use std::future::Future;
use std::pin::Pin;
use std::sync::Arc;
use std::time::Duration;

use axum::body::{Body, to_bytes};
use bytes::Bytes;
use forge_config::CacheConfig;
use http::{HeaderMap, HeaderValue, Method, Request, Response, StatusCode};
use moka::future::Cache;
use tower::{Layer, Service};
use tracing::debug;

#[derive(Clone)]
struct CachedResponse {
  status: StatusCode,
  headers: HeaderMap,
  body: Bytes,
}

/// Tower layer that caches full HTTP responses for GET requests.
#[derive(Clone)]
pub struct HttpResponseCacheLayer {
  cache: Arc<Cache<String, CachedResponse>>,
  ttl_secs: u64,
  no_cache_paths: Vec<String>,
}

impl HttpResponseCacheLayer {
  pub fn from_config(config: &CacheConfig) -> Option<Self> {
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

  fn cache_key(path: &str, query: Option<&str>) -> String {
    if let Some(q) = query {
      format!("GET:{}?{}", path, q)
    } else {
      format!("GET:{}", path)
    }
  }

  /// Inserts a raw cache entry (for testing cache-hit path that adds cache-control when missing).
  #[cfg(test)]
  pub async fn test_insert_raw(&self, key: &str, status: StatusCode, headers: HeaderMap, body: Bytes) {
    self
      .cache
      .insert(key.to_string(), CachedResponse { status, headers, body })
      .await;
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

#[derive(Clone)]
pub struct HttpResponseCacheService<S> {
  inner: S,
  cache: Arc<Cache<String, CachedResponse>>,
  ttl_secs: u64,
  no_cache_paths: Vec<String>,
}

impl<S> HttpResponseCacheService<S> {
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
      const BODY_LIMIT: usize = 10 * 1024 * 1024;
      let body_bytes = to_bytes(body, BODY_LIMIT).await.map_err(|_| ());
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
  use forge_config::HttpResponseCacheConfig;
  use std::task::Poll;
  use tower::ServiceExt;

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
  fn from_config_http_response_disabled_returns_none() {
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

  /// Mock inner service that returns a fixed 200 OK with body.
  #[derive(Clone)]
  struct MockInner {
    status: StatusCode,
    body: Bytes,
  }

  impl MockInner {
    fn ok(body: &str) -> Self {
      Self {
        status: StatusCode::OK,
        body: Bytes::from(body.to_string()),
      }
    }
    fn with_status(status: StatusCode) -> Self {
      Self {
        status,
        body: Bytes::new(),
      }
    }
  }

  impl Service<Request<Body>> for MockInner {
    type Response = Response<Body>;
    type Error = std::convert::Infallible;
    type Future = std::future::Ready<Result<Self::Response, Self::Error>>;

    fn poll_ready(
      &mut self,
      _cx: &mut std::task::Context<'_>,
    ) -> Poll<Result<(), Self::Error>> {
      Poll::Ready(Ok(()))
    }

    fn call(&mut self, _req: Request<Body>) -> Self::Future {
      let res = Response::builder()
        .status(self.status)
        .body(Body::from(self.body.clone()))
        .unwrap();
      std::future::ready(Ok(res))
    }
  }

  #[tokio::test]
  async fn layer_get_request_cache_miss_then_hit() {
    let cfg = CacheConfig {
      enabled: true,
      application: None,
      http_response: Some(HttpResponseCacheConfig {
        enabled: true,
        default_ttl_secs: 300,
        no_cache_paths: None,
      }),
    };
    let layer = HttpResponseCacheLayer::from_config(&cfg).unwrap();
    let mut svc = layer.layer(MockInner::ok("hello"));

    let req = Request::builder()
      .method(Method::GET)
      .uri("/api/foo")
      .body(Body::empty())
      .unwrap();
    let res = svc.ready().await.unwrap().call(req).await.unwrap();
    assert_eq!(res.status(), StatusCode::OK);
    let body = to_bytes(res.into_body(), 1024).await.unwrap();
    assert_eq!(body.as_ref(), b"hello");

    // Same path: cache hit
    let req2 = Request::builder()
      .method(Method::GET)
      .uri("/api/foo")
      .body(Body::empty())
      .unwrap();
    let res2 = svc.ready().await.unwrap().call(req2).await.unwrap();
    assert_eq!(res2.status(), StatusCode::OK);
    let body2 = to_bytes(res2.into_body(), 1024).await.unwrap();
    assert_eq!(body2.as_ref(), b"hello");
  }

  #[tokio::test]
  async fn layer_non_get_passes_through() {
    let cfg = CacheConfig {
      enabled: true,
      application: None,
      http_response: Some(HttpResponseCacheConfig {
        enabled: true,
        default_ttl_secs: 300,
        no_cache_paths: None,
      }),
    };
    let layer = HttpResponseCacheLayer::from_config(&cfg).unwrap();
    let mut svc = layer.layer(MockInner::ok("post-body"));

    let req = Request::builder()
      .method(Method::POST)
      .uri("/api/foo")
      .body(Body::empty())
      .unwrap();
    let res = svc.ready().await.unwrap().call(req).await.unwrap();
    assert_eq!(res.status(), StatusCode::OK);
    let body = to_bytes(res.into_body(), 1024).await.unwrap();
    assert_eq!(body.as_ref(), b"post-body");
  }

  #[tokio::test]
  async fn layer_skip_path_not_cached() {
    let cfg = CacheConfig {
      enabled: true,
      application: None,
      http_response: Some(HttpResponseCacheConfig {
        enabled: true,
        default_ttl_secs: 300,
        no_cache_paths: Some(vec!["/health".into()]),
      }),
    };
    let layer = HttpResponseCacheLayer::from_config(&cfg).unwrap();
    let mut svc = layer.layer(MockInner::ok("health"));

    let req = Request::builder()
      .method(Method::GET)
      .uri("/health")
      .body(Body::empty())
      .unwrap();
    let res = svc.ready().await.unwrap().call(req).await.unwrap();
    assert_eq!(res.status(), StatusCode::OK);
    let body = to_bytes(res.into_body(), 1024).await.unwrap();
    assert_eq!(body.as_ref(), b"health");
  }

  #[tokio::test]
  async fn layer_non_success_not_cached() {
    let cfg = CacheConfig {
      enabled: true,
      application: None,
      http_response: Some(HttpResponseCacheConfig {
        enabled: true,
        default_ttl_secs: 300,
        no_cache_paths: None,
      }),
    };
    let layer = HttpResponseCacheLayer::from_config(&cfg).unwrap();
    let mut svc = layer.layer(MockInner::with_status(StatusCode::NOT_FOUND));

    let req = Request::builder()
      .method(Method::GET)
      .uri("/api/missing")
      .body(Body::empty())
      .unwrap();
    let res = svc.ready().await.unwrap().call(req).await.unwrap();
    assert_eq!(res.status(), StatusCode::NOT_FOUND);
  }

  #[tokio::test]
  async fn layer_root_skip_exact() {
    let cfg = CacheConfig {
      enabled: true,
      application: None,
      http_response: Some(HttpResponseCacheConfig {
        enabled: true,
        default_ttl_secs: 300,
        no_cache_paths: Some(vec!["/".into()]),
      }),
    };
    let layer = HttpResponseCacheLayer::from_config(&cfg).unwrap();
    let mut svc = layer.layer(MockInner::ok("root"));

    let req = Request::builder()
      .method(Method::GET)
      .uri("/")
      .body(Body::empty())
      .unwrap();
    let res = svc.ready().await.unwrap().call(req).await.unwrap();
    assert_eq!(res.status(), StatusCode::OK);
    let body = to_bytes(res.into_body(), 1024).await.unwrap();
    assert_eq!(body.as_ref(), b"root");
  }

  #[tokio::test]
  async fn layer_get_with_query_string_caches_by_full_uri() {
    let cfg = CacheConfig {
      enabled: true,
      application: None,
      http_response: Some(HttpResponseCacheConfig {
        enabled: true,
        default_ttl_secs: 300,
        no_cache_paths: None,
      }),
    };
    let layer = HttpResponseCacheLayer::from_config(&cfg).unwrap();
    let mut svc = layer.layer(MockInner::ok("with-query"));

    let req = Request::builder()
      .method(Method::GET)
      .uri("/api/foo?bar=baz")
      .body(Body::empty())
      .unwrap();
    let res = svc.ready().await.unwrap().call(req).await.unwrap();
    assert_eq!(res.status(), StatusCode::OK);
    let body = to_bytes(res.into_body(), 1024).await.unwrap();
    assert_eq!(body.as_ref(), b"with-query");

    // Same path + query: cache hit
    let req2 = Request::builder()
      .method(Method::GET)
      .uri("/api/foo?bar=baz")
      .body(Body::empty())
      .unwrap();
    let res2 = svc.ready().await.unwrap().call(req2).await.unwrap();
    assert_eq!(res2.status(), StatusCode::OK);
    let body2 = to_bytes(res2.into_body(), 1024).await.unwrap();
    assert_eq!(body2.as_ref(), b"with-query");
  }

  #[tokio::test]
  async fn layer_cache_hit_adds_cache_control_when_missing() {
    let cfg = CacheConfig {
      enabled: true,
      application: None,
      http_response: Some(HttpResponseCacheConfig {
        enabled: true,
        default_ttl_secs: 300,
        no_cache_paths: None,
      }),
    };
    let layer = HttpResponseCacheLayer::from_config(&cfg).unwrap();
    layer
      .test_insert_raw(
        "GET:/api/raw",
        StatusCode::OK,
        HeaderMap::new(),
        Bytes::from("raw"),
      )
      .await;
    let mut svc = layer.layer(MockInner::ok("ignored"));

    let req = Request::builder()
      .method(Method::GET)
      .uri("/api/raw")
      .body(Body::empty())
      .unwrap();
    let res = svc.ready().await.unwrap().call(req).await.unwrap();
    assert_eq!(res.status(), StatusCode::OK);
    assert!(res.headers().contains_key("cache-control"));
    let body = to_bytes(res.into_body(), 1024).await.unwrap();
    assert_eq!(body.as_ref(), b"raw");
  }

  #[tokio::test]
  async fn layer_body_exceeding_limit_returns_500() {
    const BODY_LIMIT: usize = 10 * 1024 * 1024;
    let oversized = vec![0u8; BODY_LIMIT + 1];
    let cfg = CacheConfig {
      enabled: true,
      application: None,
      http_response: Some(HttpResponseCacheConfig {
        enabled: true,
        default_ttl_secs: 300,
        no_cache_paths: None,
      }),
    };
    let layer = HttpResponseCacheLayer::from_config(&cfg).unwrap();
    let inner = MockInner {
      status: StatusCode::OK,
      body: Bytes::from(oversized),
    };
    let mut svc = layer.layer(inner);

    let req = Request::builder()
      .method(Method::GET)
      .uri("/api/large")
      .body(Body::empty())
      .unwrap();
    let res = svc.ready().await.unwrap().call(req).await.unwrap();
    assert_eq!(res.status(), StatusCode::INTERNAL_SERVER_ERROR);
  }
}
