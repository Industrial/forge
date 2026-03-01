# 009_rate_limiting: Rate Limiting

## Overview

Forge provides **rate limiting** via [tower-governor](https://crates.io/crates/tower-governor) (backed by [governor](https://crates.io/crates/governor)), so applications can protect endpoints from abuse. Both **per-IP** and **per-organization** (multi-tenant) limits are supported.

## Goals

- **Default: per-IP**: Unauthenticated or generic traffic is limited by client IP.
- **Optional: per-org**: Authenticated requests can be limited by `organization_id` so one tenant cannot starve others.
- **Tower-native**: Middleware integrates with Axum and existing Forge layers.
- **Configurable**: Limits (e.g. requests per minute) can be set in config or per-route.

## Architecture

### Keying Strategies

| Strategy | Key source | Use case |
|----------|------------|----------|
| **Per-IP** | Peer IP (or `X-Forwarded-For` when trusted) | Login, signup, public API. |
| **Per-org** | `auth.organization_id()` from session | Authenticated API; fair usage across tenants. |

### Integration

- **Per-IP**: Use tower-governor’s built-in key extractor (peer IP). Applied as a layer to the router or to specific routes.
- **Per-org**: Custom key extractor that uses `AuthSession` (or equivalent) to get `organization_id`; when missing, fall back to per-IP or a shared key. Requires auth to be installed.

### Response

When the limit is exceeded, the middleware returns **429 Too Many Requests**. Optionally include a `Retry-After` header (if governor/tower-governor supports it).

## Configuration

Optional section in `config/app.toml` (or a dedicated config file):

```toml
[rate_limit]
# Per-IP default (e.g. 60 requests per minute per IP)
requests_per_minute = 60
# Per-org (when using org key)
requests_per_minute_per_org = 300
```

If not set, use sensible defaults (e.g. 60/min per IP). Per-route overrides may be supported later.

## Usage

**Default (per-IP on entire app):**

```rust
App::new()
    .with_rate_limit_per_ip(60)  // 60 req/min per IP
    .route("/", handler)
    .serve()
    .await
```

**Per-org (when auth is enabled):**

```rust
App::new()
    .with_auth(db, backend_factory)
    .with_rate_limit_per_org(300)  // 300 req/min per organization
    .route("/api/projects", handler)
    .serve()
    .await
```

Routes that should not be rate-limited (e.g. `/healthz`) are added without the layer or excluded by convention.

## Dependencies

- **tower-governor**: Tower middleware for rate limiting.
- **governor**: Rate limiter algorithm (quota, keyed limiters).

## Success Criteria

1. Per-IP rate limiting returns 429 when the limit is exceeded.
2. Per-org rate limiting uses `organization_id` when the user is authenticated; unauthenticated requests use per-IP or a default key.
3. Health and other internal endpoints can be excluded from rate limiting.
4. Limits are configurable (e.g. via `config/app.toml` or builder methods).

## Future Extensions

- Per-route limits (e.g. stricter limit on `/auth/login`).
- Configurable key extractors (e.g. API key instead of org).
- Metrics (e.g. rate limit hit count).
