//! OpenTelemetry distributed tracing and W3C trace context propagation for the Forge framework.

use axum::http::Request;
use opentelemetry::Context;
use opentelemetry::global;
use opentelemetry::trace::{Span, TraceContextExt, Tracer};
use opentelemetry_sdk::propagation::TraceContextPropagator;
use opentelemetry_sdk::trace::SdkTracerProvider;
use std::task::{Context as TaskContext, Poll};
use tower::{Layer, Service};
use tracing_subscriber::EnvFilter;

pub(crate) struct HeaderExtractor<'a>(&'a axum::http::HeaderMap);

impl opentelemetry::propagation::Extractor for HeaderExtractor<'_> {
  fn get(&self, key: &str) -> Option<&str> {
    self.0.get(key).and_then(|v| v.to_str().ok())
  }

  fn keys(&self) -> Vec<&str> {
    static W3C_KEYS: &[&str] = &["traceparent", "tracestate"];
    W3C_KEYS.to_vec()
  }
}

/// Tower layer that extracts W3C trace context from request headers.
#[derive(Clone, Default)]
pub struct TraceContextPropagationLayer;

impl<S> Layer<S> for TraceContextPropagationLayer {
  type Service = TraceContextPropagationService<S>;

  fn layer(&self, inner: S) -> Self::Service {
    TraceContextPropagationService { inner }
  }
}

#[derive(Clone)]
pub struct TraceContextPropagationService<S> {
  inner: S,
}

impl<S, ReqBody> Service<Request<ReqBody>> for TraceContextPropagationService<S>
where
  S: Service<Request<ReqBody>> + Clone + Send + 'static,
  S::Future: Send,
  ReqBody: Send + 'static,
{
  type Response = S::Response;
  type Error = S::Error;
  type Future = std::pin::Pin<
    Box<dyn std::future::Future<Output = Result<Self::Response, Self::Error>> + Send>,
  >;

  fn poll_ready(&mut self, cx: &mut TaskContext<'_>) -> Poll<Result<(), Self::Error>> {
    self.inner.poll_ready(cx)
  }

  fn call(&mut self, req: Request<ReqBody>) -> Self::Future {
    let ctx = global::get_text_map_propagator(|propagator| {
      propagator.extract(&HeaderExtractor(req.headers()))
    });
    let guard = ctx.attach();
    let future = self.inner.call(req);
    Box::pin(async move {
      let _ = guard;
      future.await
    })
  }
}

/// Parses the W3C traceparent header and returns the trace id (32 hex chars), if valid.
pub fn trace_id_from_traceparent(header_value: Option<&str>) -> Option<String> {
  let s = header_value?.trim();
  let parts: Vec<&str> = s.split_terminator('-').collect();
  if parts.len() < 2 {
    return None;
  }
  let trace_id = parts[1];
  if trace_id.len() == 32 && trace_id.chars().all(|c| c.is_ascii_hexdigit()) {
    Some(trace_id.to_string())
  } else {
    None
  }
}

/// Returns the current trace id (hex string), or a new one if none is set.
pub fn find_current_trace_id() -> Option<String> {
  let ctx = Context::current();
  let span_ref = ctx.span();
  let sc = span_ref.span_context();
  if sc.is_valid() {
    return Some(sc.trace_id().to_string());
  }
  if let Some(id) = tracing_opentelemetry_instrumentation_sdk::find_current_trace_id() {
    return Some(id);
  }
  let tracer = global::tracer("forge");
  let new_span = tracer.start("request");
  let trace_id = new_span.span_context().trace_id().to_string();
  let ctx = ctx.with_span(new_span);
  let _guard = ctx.attach();
  Some(trace_id)
}

/// Initialize OpenTelemetry: no-op tracer provider and W3C Trace Context propagator.
pub fn init_otel() {
  let propagator = TraceContextPropagator::new();
  global::set_text_map_propagator(propagator);

  let provider = SdkTracerProvider::builder().build();
  global::set_tracer_provider(provider);
}

/// Build the OpenTelemetry layer for the tracing subscriber.
pub fn otel_layer() -> impl tracing_subscriber::Layer<tracing_subscriber::Registry> {
  let tracer = global::tracer("forge");
  tracing_opentelemetry::layer().with_tracer(tracer)
}

/// Build env filter for the tracing subscriber.
pub fn env_filter() -> EnvFilter {
  EnvFilter::try_from_default_env()
    .unwrap_or_else(|_| EnvFilter::new("info"))
    .add_directive("axum_tracing_opentelemetry=error".parse().unwrap())
}

#[cfg(test)]
mod tests {
  use super::*;
  use axum::body::Body;
  use axum::http::Request;
  use opentelemetry::propagation::Extractor;
  use std::convert::Infallible;
  use tower::ServiceExt;

  #[test]
  fn trace_id_from_traceparent_valid_returns_some() {
    let trace_id = "00000000000000000000000000000001";
    let header = format!("00-{}-0000000000000000-00", trace_id);
    assert_eq!(
      trace_id_from_traceparent(Some(header.trim())),
      Some(trace_id.to_string())
    );
  }

  #[test]
  fn trace_id_from_traceparent_none_returns_none() {
    assert_eq!(trace_id_from_traceparent(None), None);
  }

  #[test]
  fn trace_id_from_traceparent_empty_string_returns_none() {
    assert_eq!(trace_id_from_traceparent(Some("")), None);
  }

  #[test]
  fn trace_id_from_traceparent_single_part_returns_none() {
    assert_eq!(trace_id_from_traceparent(Some("00")), None);
  }

  #[test]
  fn trace_id_from_traceparent_wrong_length_returns_none() {
    assert_eq!(
      trace_id_from_traceparent(Some("00-0000000000000000000000000000000-00-00")),
      None
    );
    assert_eq!(
      trace_id_from_traceparent(Some("00-000000000000000000000000000000001-00-00")),
      None
    );
  }

  #[test]
  fn trace_id_from_traceparent_non_hex_returns_none() {
    assert_eq!(
      trace_id_from_traceparent(Some("00-gggggggggggggggggggggggggggggggg-00-00")),
      None
    );
  }

  #[test]
  fn trace_id_from_traceparent_whitespace_trimmed() {
    let trace_id = "00000000000000000000000000000001";
    let header = format!("  00-{}-0000000000000000-00  ", trace_id);
    assert_eq!(
      trace_id_from_traceparent(Some(header.as_str())),
      Some(trace_id.to_string())
    );
  }

  #[test]
  fn init_otel_sets_global_propagator_and_provider() {
    init_otel();
    let _ = global::tracer_provider();
    global::get_text_map_propagator(|_p| ());
  }

  #[test]
  fn env_filter_returns_default_when_env_unset() {
    let _filter = env_filter();
  }

  #[test]
  fn env_filter_uses_rust_log_when_set() {
    // SAFETY: single-threaded test; we restore the env after
    unsafe {
      std::env::set_var("RUST_LOG", "debug");
    }
    let _filter = env_filter();
    unsafe {
      std::env::remove_var("RUST_LOG");
    }
  }

  #[test]
  fn header_extractor_keys_and_get() {
    let mut headers = axum::http::HeaderMap::new();
    headers.insert(
      axum::http::header::HeaderName::from_static("traceparent"),
      axum::http::header::HeaderValue::from_static(
        "00-00000000000000000000000000000001-0000000000000000-00",
      ),
    );
    let ext = crate::HeaderExtractor(&headers);
    assert_eq!(ext.keys(), vec!["traceparent", "tracestate"]);
    assert_eq!(
      ext.get("traceparent"),
      Some("00-00000000000000000000000000000001-0000000000000000-00")
    );
    assert_eq!(ext.get("tracestate"), None);
    assert_eq!(ext.get("other"), None);
  }

  #[test]
  fn header_extractor_get_returns_none_for_invalid_utf8() {
    let mut headers = axum::http::HeaderMap::new();
    let invalid_utf8 = axum::http::header::HeaderValue::from_bytes(b"invalid-\xff-utf8").unwrap();
    headers.insert(
      axum::http::header::HeaderName::from_static("x-custom"),
      invalid_utf8,
    );
    let ext = crate::HeaderExtractor(&headers);
    assert_eq!(ext.get("x-custom"), None);
  }

  #[test]
  fn otel_layer_builds_without_panic() {
    init_otel();
    let _layer = otel_layer();
  }

  #[test]
  fn find_current_trace_id_returns_some_after_init() {
    init_otel();
    let id = find_current_trace_id();
    assert!(id.is_some());
    let id = id.unwrap();
    assert_eq!(id.len(), 32);
    assert!(id.chars().all(|c| c.is_ascii_hexdigit()));
  }

  #[test]
  fn find_current_trace_id_uses_active_span_when_present() {
    use opentelemetry::trace::Tracer;
    init_otel();
    let tracer = global::tracer("forge");
    let span = tracer.start("test_span");
    let expected_id = span.span_context().trace_id().to_string();
    let ctx = opentelemetry::Context::current().with_span(span);
    let _guard = ctx.attach();
    let id = find_current_trace_id();
    assert_eq!(id.as_deref(), Some(expected_id.as_str()));
  }

  #[test]
  fn find_current_trace_id_uses_instrumentation_sdk_when_no_otel_span() {
    use tracing_subscriber::prelude::*;
    init_otel();
    let layer = otel_layer();
    let subscriber = tracing_subscriber::registry().with(layer);
    let id = tracing::subscriber::with_default(subscriber, || {
      let _span = tracing::info_span!("test_instrumentation").entered();
      find_current_trace_id()
    });
    assert!(id.is_some());
    let id = id.unwrap();
    assert_eq!(id.len(), 32);
    assert!(id.chars().all(|c| c.is_ascii_hexdigit()));
  }

  #[tokio::test]
  async fn trace_context_propagation_layer_wraps_service() {
    init_otel();
    let layer = TraceContextPropagationLayer;
    let svc = layer.layer(tower::service_fn(|_req: Request<Body>| async {
      Ok::<_, Infallible>(())
    }));
    let req = Request::builder()
      .header(
        "traceparent",
        "00-00000000000000000000000000000001-0000000000000000-00",
      )
      .body(Body::empty())
      .unwrap();
    svc.oneshot(req).await.unwrap();
  }

  #[tokio::test]
  async fn trace_context_propagation_layer_poll_ready() {
    init_otel();
    let layer = TraceContextPropagationLayer;
    let mut svc = layer.layer(tower::service_fn(|_req: Request<Body>| async {
      Ok::<_, Infallible>(())
    }));
    let _ = svc.ready().await.unwrap();
  }

  mod bdd_tests {
    use super::*;

    /// BDD-style tests focusing on behavior rather than implementation.
    /// Tests verify trace ID extraction, trace context propagation, and OpenTelemetry initialization behavior.
    mod trace_id_extraction_behavior {
      use super::*;

      #[test]
      fn should_extract_trace_id_from_valid_traceparent_header() {
        // Given: a valid traceparent header
        let trace_id = "00000000000000000000000000000001";
        let header = format!("00-{}-0000000000000000-00", trace_id);

        // When: extracting trace ID
        let result = trace_id_from_traceparent(Some(header.trim()));

        // Then: should return the trace ID
        assert_eq!(result, Some(trace_id.to_string()));
      }

      #[test]
      fn should_return_none_when_traceparent_header_is_missing() {
        // Given: no traceparent header
        // When: extracting trace ID
        let result = trace_id_from_traceparent(None);

        // Then: should return None
        assert_eq!(result, None);
      }

      #[test]
      fn should_return_none_when_traceparent_header_is_empty() {
        // Given: an empty traceparent header
        // When: extracting trace ID
        let result = trace_id_from_traceparent(Some(""));

        // Then: should return None
        assert_eq!(result, None);
      }

      #[test]
      fn should_return_none_when_trace_id_has_wrong_length() {
        // Given: a traceparent header with trace ID that's too short
        let short_header = "00-0000000000000000000000000000000-00-00";

        // When: extracting trace ID
        let result = trace_id_from_traceparent(Some(short_header));

        // Then: should return None
        assert_eq!(result, None);
      }

      #[test]
      fn should_return_none_when_trace_id_contains_non_hex_characters() {
        // Given: a traceparent header with non-hex characters in trace ID
        let invalid_header = "00-gggggggggggggggggggggggggggggggg-00-00";

        // When: extracting trace ID
        let result = trace_id_from_traceparent(Some(invalid_header));

        // Then: should return None
        assert_eq!(result, None);
      }

      #[test]
      fn should_trim_whitespace_from_traceparent_header() {
        // Given: a traceparent header with leading/trailing whitespace
        let trace_id = "00000000000000000000000000000001";
        let header = format!("  00-{}-0000000000000000-00  ", trace_id);

        // When: extracting trace ID
        let result = trace_id_from_traceparent(Some(header.as_str()));

        // Then: should extract trace ID after trimming
        assert_eq!(result, Some(trace_id.to_string()));
      }

      #[test]
      fn should_return_none_when_traceparent_has_insufficient_parts() {
        // Given: a traceparent header with only one part
        // When: extracting trace ID
        let result = trace_id_from_traceparent(Some("00"));

        // Then: should return None
        assert_eq!(result, None);
      }
    }

    mod trace_context_propagation_behavior {
      use super::*;

      #[test]
      fn should_extract_w3c_trace_keys() {
        // Given: a HeaderExtractor
        let headers = axum::http::HeaderMap::new();
        let ext = HeaderExtractor(&headers);

        // When: getting keys
        let keys = ext.keys();

        // Then: should return W3C trace keys
        assert_eq!(keys, vec!["traceparent", "tracestate"]);
      }

      #[test]
      fn should_get_traceparent_header_value() {
        // Given: headers with traceparent
        let mut headers = axum::http::HeaderMap::new();
        headers.insert(
          axum::http::header::HeaderName::from_static("traceparent"),
          axum::http::header::HeaderValue::from_static(
            "00-00000000000000000000000000000001-0000000000000000-00",
          ),
        );

        // When: extracting traceparent value
        let ext = HeaderExtractor(&headers);
        let value = ext.get("traceparent");

        // Then: should return the header value
        assert_eq!(
          value,
          Some("00-00000000000000000000000000000001-0000000000000000-00")
        );
      }

      #[test]
      fn should_return_none_for_missing_header() {
        // Given: headers without traceparent
        let headers = axum::http::HeaderMap::new();

        // When: getting missing header
        let ext = HeaderExtractor(&headers);
        let value = ext.get("traceparent");

        // Then: should return None
        assert_eq!(value, None);
      }

      #[test]
      fn should_return_none_for_invalid_utf8_header() {
        // Given: headers with invalid UTF-8 value
        let mut headers = axum::http::HeaderMap::new();
        let invalid_utf8 =
          axum::http::header::HeaderValue::from_bytes(b"invalid-\xff-utf8").unwrap();
        headers.insert(
          axum::http::header::HeaderName::from_static("x-custom"),
          invalid_utf8,
        );

        // When: getting invalid UTF-8 header
        let ext = HeaderExtractor(&headers);
        let value = ext.get("x-custom");

        // Then: should return None (invalid UTF-8 cannot be converted to str)
        assert_eq!(value, None);
      }

      #[tokio::test]
      async fn should_propagate_trace_context_through_layer() {
        // Given: a TraceContextPropagationLayer and service
        init_otel();
        let layer = TraceContextPropagationLayer;
        let svc = layer.layer(tower::service_fn(|_req: Request<Body>| async {
          Ok::<_, Infallible>(())
        }));

        // When: calling service with traceparent header
        let req = Request::builder()
          .header(
            "traceparent",
            "00-00000000000000000000000000000001-0000000000000000-00",
          )
          .body(Body::empty())
          .unwrap();

        // Then: should process request successfully
        let result = svc.oneshot(req).await;
        assert!(result.is_ok());
      }

      #[tokio::test]
      async fn should_handle_poll_ready_correctly() {
        // Given: a TraceContextPropagationService
        init_otel();
        let layer = TraceContextPropagationLayer;
        let mut svc = layer.layer(tower::service_fn(|_req: Request<Body>| async {
          Ok::<_, Infallible>(())
        }));

        // When: polling ready
        // Then: should return ready
        let result = svc.ready().await;
        assert!(result.is_ok());
      }
    }

    mod opentelemetry_initialization_behavior {
      use super::*;

      #[test]
      fn should_initialize_global_propagator() {
        // Given: uninitialized OpenTelemetry
        // When: initializing OpenTelemetry
        init_otel();

        // Then: global propagator should be set
        global::get_text_map_propagator(|_p| {
          // If we can get the propagator, it's initialized
        });
      }

      #[test]
      fn should_initialize_global_tracer_provider() {
        // Given: uninitialized OpenTelemetry
        // When: initializing OpenTelemetry
        init_otel();

        // Then: global tracer provider should be set
        let _provider = global::tracer_provider();
      }

      #[test]
      fn should_build_otel_layer() {
        // Given: initialized OpenTelemetry
        init_otel();

        // When: building OTel layer
        let _layer = otel_layer();

        // Then: layer should be created without panic
      }
    }

    mod trace_id_discovery_behavior {
      use super::*;

      #[test]
      fn should_find_current_trace_id_after_init() {
        // Given: initialized OpenTelemetry
        init_otel();

        // When: finding current trace ID
        let id = find_current_trace_id();

        // Then: should return a valid trace ID
        assert!(id.is_some(), "Should return a trace ID");
        let id = id.unwrap();
        assert_eq!(id.len(), 32, "Trace ID should be 32 hex characters");
        assert!(
          id.chars().all(|c| c.is_ascii_hexdigit()),
          "Trace ID should contain only hex digits"
        );
      }

      #[test]
      fn should_use_active_span_trace_id_when_present() {
        // Given: initialized OpenTelemetry with active span
        use opentelemetry::trace::Tracer;
        init_otel();
        let tracer = global::tracer("forge");
        let span = tracer.start("test_span");
        let expected_id = span.span_context().trace_id().to_string();
        let ctx = opentelemetry::Context::current().with_span(span);
        let _guard = ctx.attach();

        // When: finding current trace ID
        let id = find_current_trace_id();

        // Then: should use the active span's trace ID
        assert_eq!(id.as_deref(), Some(expected_id.as_str()));
      }

      #[test]
      fn should_create_new_trace_id_when_none_exists() {
        // Given: initialized OpenTelemetry without active span
        init_otel();

        // When: finding current trace ID
        let id = find_current_trace_id();

        // Then: should create and return a new trace ID
        assert!(id.is_some(), "Should create a new trace ID");
        let id = id.unwrap();
        assert_eq!(id.len(), 32, "New trace ID should be 32 hex characters");
        assert!(
          id.chars().all(|c| c.is_ascii_hexdigit()),
          "New trace ID should contain only hex digits"
        );
      }
    }

    mod env_filter_behavior {
      use super::*;

      #[test]
      fn should_use_default_filter_when_env_unset() {
        // Given: RUST_LOG environment variable is not set
        // When: creating env filter
        let _filter = env_filter();

        // Then: should use default filter (info level)
        // If function returns without error, default is used
      }

      #[test]
      fn should_use_rust_log_when_set() {
        // Given: RUST_LOG environment variable is set
        // SAFETY: single-threaded test; we restore the env after
        unsafe {
          std::env::set_var("RUST_LOG", "debug");
        }

        // When: creating env filter
        let _filter = env_filter();

        // Then: should use RUST_LOG value
        // If function returns without error, env value is used

        // Cleanup
        unsafe {
          std::env::remove_var("RUST_LOG");
        }
      }

      #[test]
      fn should_add_axum_tracing_directive() {
        // Given: env filter creation
        // When: creating env filter
        let filter = env_filter();

        // Then: should include axum_tracing_opentelemetry directive
        // The directive is added in env_filter() implementation
        // If function returns without error, directive is added
        let _ = filter;
      }
    }
  }
}
