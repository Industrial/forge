# Caching

Forge provides a configurable caching layer using **Moka** as the in-process, in-memory backend. Both **application cache** (key-value get/set/delete) and **HTTP response cache** (caching full responses for matching requests) are supported. Cache busting is **manual** via an easy-to-use API and is not enforced. All behavior is driven from a central config file.

## Objectives

1. **Single backend for now:** Moka only (in-process; no Redis or other external cache).
2. **Configurable:** Enable/disable per environment, TTLs, what to cache; config in one place.
3. **Two use cases:** Application cache (arbitrary keys) and HTTP response cache (middleware).
4. **Manual invalidation:** Handlers call the cache API to bust when needed; the framework does not auto-invalidate.

## Sync vs async cache

Use **async** (e.g. `moka::future::Cache`). Axum and Forge run on Tokio; async cache avoids blocking worker threads and fits `async fn` handlers. Sync cache would require `spawn_blocking` or similar and is not recommended for request handlers.

## Configuration: `config/cache.toml`

All cache-related settings live in **`config/cache.toml`** (or equivalent section merged by your config loader). Figment can merge this with `config/app.toml` if you use a single provider list.

### Enable/disable

- **Disable everything:** e.g. `enabled = false` so that in local development no caching is performed (no app cache, no HTTP response cache). When `enabled = false`, the cache API can no-op or return misses.
- **Enable:** `enabled = true` (default for production-like environments). Application cache and HTTP response cache respect this.

Example shape:

```toml
# config/cache.toml

enabled = true   # false to disable all caching (e.g. local dev)

[application]
# Application (key-value) cache
enabled = true
max_capacity = 10_000
default_ttl_secs = 300

[http_response]
# HTTP response cache (middleware)
enabled = true
default_ttl_secs = 60
# Optional: path patterns or methods to exclude from caching
# no_cache_paths = ["/api/debug/*", "/healthz"]
```

### TTL

- **Default TTL:** `default_ttl_secs` for both application and HTTP response cache. Entries expire after this period unless overridden per key or per route.
- **Overrides (optional):** Config or code can define per-route or per-key-prefix TTLs (e.g. `/api/static/*` 3600, user keys 300). Document the override mechanism if the implementation supports it.

## Application cache

- **Backend:** Moka only (in-process, no separate server).
- **API:** Easy to use and not enforced: handlers (or services) can call:
  - `get(key)` → optional value
  - `set(key, value, ttl_override?)`
  - `delete(key)` for manual cache busting
- **Access:** Cache instance in Axum state (or request extensions) so any handler can use it. When cache is disabled, these calls no-op or return miss.
- **What to cache:** Decided in code (which keys you set). Config only controls whether caching is enabled and default TTL/limits (e.g. max capacity).

## HTTP response cache

- **Behavior:** Middleware (e.g. Tower layer) caches full HTTP responses for selected requests (e.g. GET by path/method). Subsequent matching requests are served from cache until TTL or manual invalidation.
- **Configurable:** Enable/disable via `[http_response] enabled`; default TTL; optionally which paths or methods to exclude (e.g. don’t cache `/api/debug/*` or `/healthz`).
- **Headers:** Cached responses can set or preserve `Cache-Control`, `ETag`, `Last-Modified` so clients and CDNs can cache too. Implementation may use tower-http-cache or a similar layer.

## Cache busting (manual)

- **No automatic invalidation:** The framework does not invalidate cache on mutations. Handlers must invalidate when data changes.
- **API:** Easy and visible: e.g. `cache.delete(key)` or `cache.invalidate_pattern(prefix)`. Use after create/update/delete so the next read gets fresh data.
- **Not enforced:** No checks that every mutation invalidates; it’s the developer’s responsibility to call the API where appropriate.

## Session store

- **Where sessions live** is configurable (e.g. SQLite, or in the future Redis/Moka) via session store selection in app config. That is separate from “application cache” and “HTTP response cache”: session store is for auth sessions; application cache is for arbitrary key-value data.
- **Config:** Session store type and options can live in `config/app.toml` or in `config/cache.toml` under a `[session]` section, whichever keeps config central and clear. This doc assumes a central place; exact key names are implementation-defined.

## Status

- **Application cache:** Implemented. Forge loads optional `config/cache.toml`; when `enabled` and `[application].enabled` are true, a Moka `future::Cache` is created and injected into request extensions as `Option<Arc<AppCache>>`. Handlers use `Extension<Option<Arc<AppCache>>>` to get/set/delete. API: `get(key)`, `set(key, value)`, `delete(key)`.
- **HTTP response cache:** Implemented. When `[http_response].enabled` is true, a Tower layer caches full GET responses (by method + path). Cached responses get a `Cache-Control: public, max-age=<ttl>` header. Config supports `no_cache_paths` (e.g. `/`, `/api/auth`, `/healthz`) so localized or auth routes are not cached. Layer wraps the router; only 2xx responses are cached; body read limit 10 MiB.
- **Scaffold:** `forge new` generates `config/cache.toml` (enabled, [application], [http_response] with `no_cache_paths`), GET `/api/cache-demo` (app cache), and GET `/api/cached-page` (response-cache demo; two requests return same body).
- **E2E:** `test/e2e/016_caching.rs` asserts config shape, app cache (same body twice for `/api/cache-demo`), response cache (same body twice for `/api/cached-page`), and presence of `Cache-Control` on the cached response.
