//! Health observability (017): OpenTelemetry distributed tracing and W3C trace context propagation.
//! No built-in metrics; use the `tracing` crate in handlers for custom spans and events.

use axum::http::Request;
use opentelemetry::Context;
use opentelemetry::global;
use opentelemetry::propagation::Extractor;
use opentelemetry::trace::{Span, TraceContextExt, Tracer};
use opentelemetry_sdk::propagation::TraceContextPropagator;
use opentelemetry_sdk::trace::SdkTracerProvider;
use std::task::{Context as TaskContext, Poll};
use tower::{Layer, Service};
use tracing_subscriber::EnvFilter;

/// Wrapper so we can use [Request] headers as an OTel [Extractor] for trace context.
struct HeaderExtractor<'a>(&'a axum::http::HeaderMap);

impl Extractor for HeaderExtractor<'_> {
  fn get(&self, key: &str) -> Option<&str> {
    self.0.get(key).and_then(|v| v.to_str().ok())
  }

  fn keys(&self) -> Vec<&str> {
    static W3C_KEYS: &[&str] = &["traceparent", "tracestate"];
    W3C_KEYS.to_vec()
  }
}

/// Tower layer that extracts W3C trace context from request headers and attaches it so
/// [find_current_trace_id] and downstream see the propagated trace id.
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
  /// Inner Tower service (router or next layer).
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

/// Parses the W3C traceparent header value and returns the trace id (32 hex chars), if valid.
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

/// Returns the current trace id (hex string), or a new one for this request if none is set.
/// Prefers Context::current() (set by [TraceContextPropagationLayer] from traceparent), then
/// the tracing span's context; if still none, starts a new span so the endpoint always returns an id.
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
  // No active trace: start a span so we return an id.
  let tracer = global::tracer("forge");
  let new_span = tracer.start("request");
  let trace_id = new_span.span_context().trace_id().to_string();
  let ctx = ctx.with_span(new_span);
  let _guard = ctx.attach();
  Some(trace_id)
}

/// Initialize OpenTelemetry: no-op tracer provider (no export) and W3C Trace Context propagator.
/// Call once before init_tracing so the subscriber can use the global tracer.
pub fn init_otel() {
  let propagator = TraceContextPropagator::new();
  global::set_text_map_propagator(propagator);

  let provider = SdkTracerProvider::builder().build();
  global::set_tracer_provider(provider);
}

/// Build the OpenTelemetry layer for the tracing subscriber (use with registry).
/// Requires [init_otel] to have been called so [global::tracer_provider] is set.
pub fn otel_layer() -> impl tracing_subscriber::Layer<tracing_subscriber::Registry> {
  let tracer = global::tracer("forge");
  tracing_opentelemetry::layer().with_tracer(tracer)
}

/// Build env filter for the tracing subscriber (forge, app binary, tower_http, optional RUST_LOG).
pub fn env_filter() -> EnvFilter {
  EnvFilter::from_default_env()
    .add_directive("forge=info".parse().unwrap())
    .add_directive("app=info".parse().unwrap())
    .add_directive("tower_http=info".parse().unwrap())
}
