# 013_openapi: OpenAPI / API Documentation (Design & Research)

## Overview

Forge may expose **OpenAPI (Swagger)** documentation so that APIs are documented from code with **no extra user code**: “we have a router and it documents it.” This doc captures the design and research; implementation is deferred until the approach is feasible.

## Goal

- **Fully automatic**: User does not add annotations or separate spec files. The framework derives the OpenAPI spec from the router (routes, methods, request/response types).
- **Single source of truth**: The running app (or build) produces the spec and, optionally, a Swagger UI endpoint (e.g. `/api/docs`).

## Challenge

- **Axum’s router** does not expose path, method, or body/response types in a way that can be inspected at runtime or at compile time without additional metadata. Handlers are type-erased.
- **Current ecosystem**: [utoipa](https://github.com/juhaku/utoipa) (and utoipa-axum) require **annotations** on handlers (`#[utoipa::path(...)]`) and derives on DTOs to generate the spec. That is “minimal” but not “no extra code.”

## Options

| Option | Description | User code |
|--------|-------------|-----------|
| **A. utoipa with annotations** | Adopt utoipa; document that users add `#[utoipa::path]` and derives. | Minimal (per handler and DTO). |
| **B. Router registry** | Forge maintains a parallel registry when routes are added (e.g. `App::route(path, handler)` also records path + method). Request/response types would still need to be provided or inferred. | Possibly none for paths; types may still need hints. |
| **C. Defer** | Do not implement until tooling allows true zero-annotation spec generation from Axum. | N/A. |

## Recommendation

- **Short term**: Defer full “no extra code” OpenAPI. If we need API docs soon, adopt **Option A** (utoipa) and document it as “add minimal annotations for documented routes.”
- **Medium term**: Explore **Option B** (router registry) so that at least path and method are recorded when the user calls `App::route()`; response/body types could remain best-effort or annotation-based.

## Success Criteria (if implemented)

1. OpenAPI spec is generated from the application (no hand-written YAML).
2. Swagger UI (or equivalent) is served at a documented path (e.g. `/api/docs`).
3. “Fully automatic” is achieved if and only if we can infer or register enough metadata without user annotations; otherwise, document the minimal annotation set.

## Dependencies (if Option A)

- **utoipa**: OpenAPI types and derive.
- **utoipa-axum**: Axum integration and `OpenApiRouter`.

## Status

**Research / design only.** No implementation in the initial feature set. Revisit when:
- utoipa or another crate supports lower-friction Axum integration, or
- Forge introduces a router abstraction that can carry OpenAPI metadata.
