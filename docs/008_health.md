# 008_health: Health Check Endpoints (healthz, livez, readyz)

## Overview

Forge exposes three health endpoints with **no plugin or crate**: status code and minimal body only. No component or server details are disclosed (security).

## Goals

- **Kubernetes-style**: Machines rely on HTTP status code; body is minimal.
- **No information disclosure**: Responses do not reveal database, components, or stack.
- **Correct lifecycle**: liveness vs readiness semantics.

## Endpoints

| Path      | Method | Meaning | When 200 | When non-200 | Body |
|-----------|--------|---------|----------|----------------|------|
| **/livez**  | GET    | Process is alive | Always (process running) | — | `ok` |
| **/readyz** | GET    | Ready to accept traffic | DB ping succeeds | 503 if DB unreachable | `ok` or `unavailable` |
| **/healthz**| GET    | Legacy/simple health | Same as livez | — | `ok` |

- **livez**: No dependency checks. Used by orchestrators to decide whether to restart the process.
- **readyz**: Pings the database; 200 = ready, 503 = not ready. Used to remove the instance from load balancing when it cannot serve traffic.
- **healthz**: Returns 200 with minimal body for backward compatibility; no dependency checks.

All responses are **plain text** (`ok` or `unavailable`), not JSON. No `components`, `database`, or other internal details.

## Implementation

- Handlers live in `crates/forge/src/health.rs` (no external health crate).
- **livez** / **healthz**: return `(StatusCode::OK, "ok")`.
- **readyz**: takes `State<DatabaseConnection>`, runs `db.execute_unprepared("SELECT 1").await`; on success returns 200 and `"ok"`, on failure 503 and `"unavailable"`.

## Configuration

None. Endpoints are always mounted when the app runs.

## Success Criteria

1. GET /healthz returns 200 and body `ok` (no JSON, no components).
2. GET /livez returns 200 and body `ok`.
3. GET /readyz returns 200 and body `ok` when DB is reachable; 503 and body `unavailable` when DB is not.
4. No response discloses database, components, or stack (E2E asserts no `components` or `database` in body).

## E2E

- `008_health.rs`: asserts 200 and body content for /healthz, /livez, /readyz when server and DB are up; asserts no component/database disclosure.
