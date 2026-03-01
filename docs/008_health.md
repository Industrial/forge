# 008_health: Health Check Endpoints

## Overview

Forge exposes a **health endpoint** for liveness and readiness checks, using [axum-health](https://crates.io/crates/axum-health). This allows load balancers, Kubernetes, and other orchestrators to verify that the application and its dependencies (e.g. database) are up.

## Goals

- **Zero configuration**: Health is enabled by default when using `App::new()`.
- **Convention**: Endpoint path is `/healthz` (common in Kubernetes and cloud environments).
- **Database check**: When the app has a database connection, the health response includes a database component so readiness can reflect DB availability.

## Architecture

### Endpoint

| Path      | Method | Purpose |
|-----------|--------|---------|
| `/healthz` | GET   | Combined liveness/readiness; returns overall status and component status (e.g. database). |

Response shape (JSON):

```json
{
  "status": "UP",
  "components": {
    "database": { "status": "UP" }
  }
}
```

If the database is unreachable, the component status will be `DOWN` and the overall status may be `DOWN` (depending on axum-health semantics).

### Integration

- Health is wired in `App::into_router()` after the database connection is established.
- A `DatabaseHealthIndicator` (SeaORM) is registered with axum-health’s `Health` builder and the `/healthz` route is added to the main router before `with_state(db_conn)`.
- No user code is required; the route is added automatically.

## Configuration

No dedicated config section is required. The health endpoint is always mounted when the app runs. Path is fixed at `/healthz` for now; future versions may allow configuration (e.g. in `config/app.toml`).

## Usage

Users do not need to call any method. After building the app and calling `serve()` or `into_router()`, `GET /healthz` is available:

```bash
curl http://127.0.0.1:3000/healthz
```

## Dependencies

- **axum-health** (with `sea-orm` feature): provides `Health`, `Health::builder()`, and `DatabaseHealthIndicator` for SeaORM.

## Success Criteria

1. `GET /healthz` returns 200 and JSON with `status` and `components`.
2. When the database is available, the `database` component is `UP`.
3. When the database is unavailable (e.g. wrong URL), the health response reflects the failure (e.g. component `DOWN`).
4. No extra user code or config is required for basic health checks.

## Future Extensions

- Optional separate `/live` and `/ready` endpoints (liveness vs readiness).
- Configurable path(s) via `config/app.toml`.
- Additional indicators (e.g. Redis, external API) as optional plugins.
