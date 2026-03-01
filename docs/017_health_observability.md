# Health & Observability

Forge provides **health endpoints** (see [008_health](008_health.md)) and recommends **distributed tracing** with OpenTelemetry. Metrics (e.g. Prometheus) are not included; users implement them if needed.

## Objectives

1. **Health:** Existing `/healthz`, `/livez`, `/readyz` (minimal, no component disclosure).
2. **No built-in metrics:** No Prometheus or other metrics endpoint in the framework; users add their own.
3. **Distributed tracing:** OpenTelemetry with automatic instrumentation for all Axum routes and optional DB spans.
4. **Handler API:** Use the `tracing` crate inside handlers for custom spans and events under the request span.

## Health endpoints (existing)

- **/livez** — Liveness; no dependency checks.
- **/readyz** — Readiness; includes DB ping; 503 if DB unreachable.
- **/healthz** — Legacy health; same as livez.

No metrics endpoint is provided. If you need Prometheus (or similar), expose `/metrics` yourself and restrict it (e.g. separate listener on localhost or auth).

## Distributed tracing (OpenTelemetry)

### Automatic instrumentation for all Axum routes

Use the **axum-tracing-opentelemetry** crate ([GitHub: davidB/tracing-opentelemetry-instrumentation-sdk](https://github.com/davidB/tracing-opentelemetry-instrumentation-sdk), highest adoption among Axum + OTel crates). It provides a Tower layer that:

- Creates a span for each request (method, path, etc.).
- Propagates W3C TraceContext so downstream services and DB spans share the same trace.
- Records status and duration on response.

Add the layer to your router so **every route** is traced without per-handler code. Configure your OpenTelemetry pipeline (OTLP, Jaeger, Zipkin, etc.) and subscriber so spans are exported.

If you prefer not to depend on this crate, you can implement an equivalent layer that creates an OTel span per request and sets it as the current span; the pattern is the same.

### Custom spans and events inside handlers

The request span is the **current span** for the duration of the handler. Use the **tracing** crate directly:

- **Child spans:** `tracing::info_span!("name", key = %value)` or `#[tracing::instrument]` on a function. They become children of the request span.
- **Events:** `tracing::info!(key = value, "message")` (or `debug!`, `warn!`, `error!`). They are attached to the current span.
- **Attributes:** Add fields to spans and events; with `tracing-opentelemetry` as the layer, these show up in your OTel backend.

No need to pass the request span into handlers; **current span** is implicit. This keeps the API easy and consistent with automatic route instrumentation.

### OpenTelemetry at the database level

Use **SeaORM’s built-in tracing** so database operations appear as spans under the request span:

- **ConnectOptions::set_auto_tracing(true)** — When building the SeaORM connection, call `set_auto_tracing(true)` on `ConnectOptions`. This attaches spans to queries and keeps them under the current trace context.

Alternatively, use a wrapper such as **sea-orm-tracing** if you need a traced `DatabaseConnection` with different behavior. For most apps, `set_auto_tracing(true)` is sufficient.

Ensure the OpenTelemetry/tracing subscriber is initialized before any DB work so request and DB spans share the same trace.

## Summary

| Area | Approach |
|------|----------|
| **Health** | Existing livez / readyz / healthz (008). |
| **Metrics** | Not in Forge; implement yourself (e.g. Prometheus) and secure as needed. |
| **Route tracing** | Use **axum-tracing-opentelemetry** (or equivalent layer) for automatic spans on all routes. |
| **Handler API** | Use **tracing** (`span!`, `#[instrument]`, `info!`, etc.); current span is implicit. |
| **DB tracing** | Use **SeaORM `set_auto_tracing(true)`** (and optionally sea-orm-tracing). |

## Status

**Design and recommendations only.** Forge does not yet add axum-tracing-opentelemetry or configure SeaORM auto-tracing. This document describes the intended approach: use the recommended crate for route-level OTel, the tracing crate API in handlers, and SeaORM’s option for DB spans.
