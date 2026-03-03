# Multi-crate split and Live Query system design

This document describes a proposed **multi-crate architecture** for the Forge framework and the design of a new **Live Query** system for real-time synchronization of data across multiple connected clients. It is intended as a detailed reference for future implementation.

---

## 1. Context and goals

### 1.1 Current state

- **forge**: A single library crate containing all framework functionality: app builder, config, database, cache, auth, audit, cron, health, observability, rate limiting, security headers, seed, token auth, validation, and integration with **forge-authz**.
- **forge-cli**: CLI for scaffolding and running applications (`forge new`, `forge dev`, etc.).
- **forge-macros**: Procedural macros used by applications built with Forge.
- **forge-authz**: Separate crate for authorization (policies, `AuthzContext`, `Action`).

The **forge** crate is a monolith: every application pulls in all dependencies (axum, sea_orm, tower-sessions, moka, opentelemetry, apalis, etc.) even if it uses only a subset of features. Module boundaries exist internally but there is no way to depend on “only cache” or “only db” without depending on the whole framework.

### 1.2 Goals of the split

1. **Clear boundaries**: Each crate has a single, well-defined responsibility.
2. **Reuse**: Applications and other crates can depend only on what they need (e.g. `forge-cache` without auth or DB).
3. **Dependency direction**: Acyclic graph; “core” at the bottom, “app” at the top. No crate in the middle should depend on the app builder.
4. **Incremental adoption**: The split can be done gradually; the main **forge** crate can remain a facade that re-exports and wires the smaller crates for backward compatibility.
5. **Live Query**: Introduce a dedicated **Live Query** system (new crate) for real-time sync—when data changes, all relevant clients (e.g. users in the same organization viewing the same UI) receive updates automatically. This system should be designed so it can use a **swappable backend** (in-memory by default, Redis later for multi-instance deployments), consistent with the existing cache layer’s intended evolution.

---

## 2. Terminology

| Term | Meaning |
|------|--------|
| **Live Query** | A system that lets clients subscribe to a “query” or “scope” (e.g. “users in org X”) and receive real-time updates when the underlying data changes. Named to align with established usage (e.g. Supabase Realtime, Firebase). |
| **Real-time sync** | The behavior: multiple users see the same data update in their browsers without refresh when one user (or the system) mutates it. |
| **Pub/sub over WebSockets** | The mechanism: clients subscribe to channels (e.g. `org:{id}`); the server publishes events to those channels when mutations occur. |
| **Application-level broadcast** | The pattern: in each handler that performs a mutation, after the DB write the server calls a “broadcast” or “notify” function that pushes an event to the right set of connected clients. |
| **Swappable backend** | An abstraction (trait or enum) that allows the same API to be backed by different implementations: e.g. in-memory for single process, Redis for multi-instance. |

---

## 3. Cache vs Live Query (distinction)

The existing **cache** and the new **Live Query** system solve different problems. Both may eventually share a “swappable backend” design, but their roles differ.

| Concern | Cache (existing) | Live Query (new) |
|--------|------------------|------------------|
| **Role** | Store computed or serialized values; optionally cache HTTP responses. | Notify subscribed clients when data changes so UIs stay in sync. |
| **API** | `get(key)`, `set(key, value)`, `delete(key)`. | Subscribe to a scope (e.g. org, resource); receive events on create/update/delete. |
| **Backend** | Key-value store (Moka today; Redis later = shared cache across instances). | Pub/sub or fan-out (in-memory today; Redis Pub/Sub later = broadcast across instances). |
| **Consumer** | Same process (handlers reading/writing cache). | Remote clients (browsers) over WebSocket (or SSE). |

- **Cache**: “What is the current value for this key?” and “invalidate when …” (manual today).
- **Live Query**: “Who is subscribed to this scope?” and “push an event to those connections when something changes.”

The **swappable backend** idea applies to both:

- **Cache**: Swap Moka for Redis (or another key-value backend) so multiple server instances share the same cache.
- **Live Query**: Swap in-memory pub/sub for Redis Pub/Sub so that “broadcast to org X” reaches all subscribed connections across all instances.

The two systems are independent: an app can use cache without Live Query, or Live Query without the application cache. They may share a small common abstraction (e.g. a trait for “backend” configuration) in a core crate, but Live Query does **not** depend on the key-value cache for its core behavior.

---

## 4. Proposed crate split

The following is a **full** decomposition of the current **forge** library into crates. Each crate lists its **contents** (modules/types) and **internal dependencies** (other Forge crates only; std/tokio/serde/axum/etc. omitted for brevity).

### 4.1 Layer 0: Foundation

| Crate | Contents | Internal deps |
|-------|----------|----------------|
| **forge-core** | `error::Error`, `validation::Valid` / `Validate`, and any shared types used across multiple crates (e.g. re-exports of `uuid`, `serde` if desired). Minimal surface. | None |

**Purpose**: Single place for errors and validation so that other crates depend on a small, stable core instead of duplicating types.

---

### 4.2 Layer 1: Configuration and database

| Crate | Contents | Internal deps |
|-------|----------|----------------|
| **forge-config** | `ForgeConfig`, `load_config()`, `AppConfig`, `ServerConfig`, `DatabaseConfig`, `CacheConfig` (or reference to cache config), `FrontendConfig`, `effective_environment()`. Config loading via Figment. | forge-core (if error types live there) |
| **forge-db** | `initialize_database()`, `DbConnection` type (re-export of `sea_orm_tracing::TracedConnection`), migration helpers. | forge-config |

**Note**: Today `ForgeConfig` references `crate::cache::CacheConfig`; that can remain a dependency from config to cache, or `CacheConfig` can move into forge-config with forge-cache only adding behavior. Either way, forge-db stays above forge-config.

---

### 4.3 Layer 2: Cross-cutting features

| Crate | Contents | Internal deps |
|-------|----------|----------------|
| **forge-cache** | `AppCache`, `NoOpAppCache`, `CacheConfig` (if not in config), `HttpResponseCacheLayer` / `HttpResponseCacheService`. | forge-config |
| **forge-authz** | *(Already exists.)* Policies, `AuthzContext`, `Action`, permission checks. | — |
| **forge-auth** | `token_auth` (Bearer/session), session management (tower-sessions, axum-login), authn. | forge-core, forge-authz |
| **forge-audit** | `AuditEvent`, `EventKind`, `Outcome`, `AuditError`. | forge-core, forge-authz |
| **forge-observability** | Tracing setup, OpenTelemetry layers, trace context propagation, `find_current_trace_id`, `trace_id_from_traceparent`. | forge-core |
| **forge-security** | `security_headers::add_security_headers`. | — |
| **forge-rate-limit** | `RequesterOrgKey`, governor-based rate limiting layer. | forge-authz |

---

### 4.4 Layer 3: Database-dependent features

| Crate | Contents | Internal deps |
|-------|----------|----------------|
| **forge-cron** | `CronSchedule`, `CronRunner`, cron task execution. | forge-db |
| **forge-health** | `healthz`, `livez`, `readyz` endpoints. | forge-db (optional for readyz) |
| **forge-seed** | `Seeder`, seed execution. | forge-db |
| **forge-jobs** | `ScheduledTaskJob`, Apalis integration. | forge-db |

---

### 4.5 Layer 4: Live Query (new)

| Crate | Contents | Internal deps |
|-------|----------|----------------|
| **forge-live** | Live Query system: **channel/scope model** (e.g. `org:{org_id}`, `resource:{type}:{id}`), **subscription registry** (which connections are in which channel), **broadcast API** (e.g. `broadcast_to_org(org_id, event)`), **swappable backend** (in-memory implementation by default; Redis Pub/Sub later for multi-instance). Optional: WebSocket endpoint and connection lifecycle (accept upgrade, register connection to channels, heartbeat). Alternatively, WebSocket can remain in the app crate and forge-live only provides the broadcast + channel abstraction. | forge-core |

**Design choices**:

- **Minimal deps**: forge-live does **not** depend on forge-db, forge-auth, or forge-cache. The **app** (orchestrator) wires DB, auth, and Live Query: handlers perform mutations, then call the Live Query broadcast API. Auth is enforced at the HTTP/WebSocket layer before a connection is allowed to subscribe to a channel.
- **Transport**: If forge-live owns the WebSocket server, it will depend on **axum** (with `ws`). If the app owns the WebSocket and only uses forge-live for “broadcast to channel,” forge-live can avoid axum and only provide a trait or struct that the app passes to handlers (e.g. `Arc<dyn LiveBroadcaster>`).
- **Swappable backend**: A trait (e.g. `LiveBackend`) with implementations `InMemoryLiveBackend` and, later, `RedisLiveBackend`. Both forge-cache and forge-live can follow the same pattern (in-memory default, Redis optional) without forge-live depending on forge-cache.

---

### 4.6 Layer 5: Application builder (facade)

| Crate | Contents | Internal deps |
|-------|----------|----------------|
| **forge** | `App` builder, `init_tracing()`, re-exports of public APIs from all other crates, `into_router()`, `serve()`, registration of routes/layers for health, cache, auth, cron, observability, rate limit, security, and optionally Live Query. | forge-config, forge-db, forge-cache, forge-auth, forge-authz, forge-audit, forge-observability, forge-security, forge-rate-limit, forge-cron, forge-health, forge-seed, forge-jobs, forge-live (optional), forge-macros |

**Purpose**: One entry point for “full framework” usage; composes all crates and preserves backward compatibility for existing apps.

---

## 5. Dependency graph (summary)

```
forge (facade)
  ├── forge-live
  │     └── forge-core
  ├── forge-cron, forge-health, forge-seed, forge-jobs
  │     └── forge-db
  │           └── forge-config
  │                 └── forge-core
  ├── forge-cache
  │     └── forge-config
  ├── forge-auth
  │     ├── forge-core
  │     └── forge-authz
  ├── forge-audit
  │     ├── forge-core
  │     └── forge-authz
  ├── forge-observability
  │     └── forge-core
  ├── forge-rate-limit
  │     └── forge-authz
  └── forge-security (no internal deps)
```

**forge-live** depends only on **forge-core** (and optionally axum if it owns the WebSocket). It does **not** depend on forge-db, forge-auth, or forge-cache. The app crate is responsible for:

- Passing a `LiveBroadcaster` (or similar) into request state or handlers.
- After a mutation in a handler, calling e.g. `broadcast_to_org(org_id, LiveEvent::UsersUpdated(...))`.
- Enforcing auth so that only authorized connections can subscribe to a given channel (e.g. org-scoped).

---

## 6. Live Query crate in detail

### 6.1 Responsibilities

1. **Channel / scope model**: Define channel names (e.g. `org:uuid`, `resource:users:uuid`) and rules for who can subscribe (enforcement can be in the app; forge-live only stores the mapping connection ↔ channels).
2. **Subscription registry**: For in-memory backend, maintain a map `channel → set of connection IDs` (or similar) and `connection_id → set of channels`.
3. **Broadcast API**: `broadcast(channel, payload)` or `broadcast_to_org(org_id, event)` that serializes the event and sends it to every connection subscribed to that channel.
4. **Swappable backend**: Trait `LiveBackend` (or equivalent) with:
   - `subscribe(connection_id, channel)`
   - `unsubscribe(connection_id, channel)`
   - `broadcast(channel, message)`
   - `connections_in_channel(channel)` if needed for debugging or metrics.
   - Implementations: `InMemoryLiveBackend`, and later `RedisLiveBackend` that publishes to Redis and subscribes to Redis so all server instances receive and forward to their local WebSocket connections.

### 6.2 Event shape

Events pushed to clients should be serializable (e.g. JSON) and include at least:

- **Topic / type**: e.g. `users.updated`, `users.created`, `users.deleted`.
- **Payload**: Opaque or typed (e.g. the new or deleted entity id, or a full DTO). The frontend can refetch or patch local state based on this.

Optional: `id` (for idempotency or ordering), `timestamp`, `actor` (user id) for audit or UX.

### 6.3 Where broadcast is invoked

In the **application** (or in a shared handler layer), after any mutation that should be visible to other users:

1. Perform the DB write (SeaORM).
2. If successful, call e.g. `live.broadcast_to_org(org_id, LiveEvent::UsersUpdated { user_id, ... })`.
3. No need to touch the cache for this; cache remains for “store this computed value” and “invalidate when …” if desired.

### 6.4 Relation to existing WebSocket doc

The existing **docs/014_real_time_websockets.md** describes raw WebSocket and SSE usage with Axum. The Live Query system **builds on** that: the app still uses Axum’s WebSocket upgrade; the difference is that forge-live provides the **channel and broadcast abstraction** so that handlers do not manage connection sets by hand. So 014 remains valid; Live Query is an additional, higher-level layer (and can be documented in a dedicated doc, e.g. `021_live_query.md`).

---

## 7. Migration path and trade-offs

### 7.1 Phased migration

1. **Phase 1**: Extract **forge-core** and **forge-config**; have **forge** depend on them and re-export. No breaking change if the public API of **forge** is unchanged.
2. **Phase 2**: Extract **forge-db**, **forge-cache**, **forge-auth**, **forge-observability**, **forge-security**, **forge-rate-limit**, **forge-audit**, **forge-cron**, **forge-health**, **forge-seed**, **forge-jobs** one by one; **forge** composes them. Each step can be done so that **forge** still exposes the same API.
3. **Phase 3**: Add **forge-live** with in-memory backend only; integrate into **forge** app builder (optional feature or method like `with_live_query()`).
4. **Phase 4**: Introduce **LiveBackend** trait and Redis implementation; document configuration for multi-instance deployments.

### 7.2 Trade-offs

- **Many small crates**: Clear boundaries, better reuse and incremental compile; more Cargo.toml and version management; refactors that cross crates require more coordination.
- **Fewer, coarser crates**: e.g. **forge-core** (error, config, validation), **forge-db**, **forge-auth** (authn + authz), **forge-cache**, **forge-background** (cron + jobs), **forge-observability** (health + tracing), **forge-live**, **forge**. Simpler workspace and fewer version bumps; less fine-grained reuse.
- **Recommendation**: Start by extracting **forge-core**, **forge-cache**, and **forge-live** as the highest-value splits; keep the rest inside **forge** until boundaries are stable. This validates the direction (including Live Query as its own crate and its dependency set) without a big-bang split.

---

## 8. Naming and public API

- **Live Query**: The feature and crate name. Public types could be prefixed accordingly (e.g. `live_query::LiveBackend`, `live_query::broadcast_to_org`) or live under a `live` module in the facade.
- **forge-live**: Crate name. Package name could be `forge-live` to match the existing `forge-authz` style.
- **Cache**: Existing names **AppCache**, **CacheConfig**, **NoOpAppCache**, **HttpResponseCacheLayer** stay as-is; they live in **forge-cache** and are re-exported from **forge** for backward compatibility.

---

## 9. Summary

| Question | Answer |
|----------|--------|
| Which crates would we have after a full split? | forge-core, forge-config, forge-db, forge-cache, forge-authz (existing), forge-auth, forge-audit, forge-observability, forge-security, forge-rate-limit, forge-cron, forge-health, forge-seed, forge-jobs, **forge-live**, and **forge** (facade). |
| What would **forge-live** depend on? | **forge-core** only (and optionally **axum** if the crate owns the WebSocket endpoint). It does **not** depend on forge-db, forge-auth, or forge-cache. |
| How does Live Query relate to the cache? | Cache = key-value storage and optional HTTP response cache. Live Query = pub/sub broadcast so clients see updates in real time. Both can share a “swappable backend” pattern (in-memory → Redis) but are independent systems. |
| Where is broadcast invoked? | In the **app** (handlers or a shared layer), after mutations, by calling the Live Query broadcast API with the appropriate channel (e.g. org) and event payload. |

This document should be updated as the design is implemented or refined (e.g. exact trait names, event schema, and Redis backend details).
