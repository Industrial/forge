# Frontend implementation and choices

This document records how the frontend aligns with the backend (generic handler, model registry, RPC, subscriptions) and the decisions made per epic. It is the single source of truth for frontend implementation choices.

**Related:** [Frontend alignment plan](frontend-alignment-plan.md), deliverables 01–10, technical-choices 01–10.

---

## Epic 1 – Scoped session

### Decisions

- **Token persistence:** Token is stored in **localStorage** (existing).
- **Scope/profile selection:** A dedicated **scope selection screen** exists; multi-org users are redirected there when `needs_profile_select` is true (e.g. after login).
- **Current scope in UI:** A **dropdown in the navbar** lists available scopes. **Gap:** the **current scope is not highlighted**; this should be fixed so the active org/role is visually indicated.

### Implementation notes

- Scope is sent on every request via `X-Organization-Id` and `X-Role-Id` (see `httpClientWithAuth` or equivalent). Auth state (user, scopes, permissions, current scope, `needs_scope_select`) comes from `GET /api/auth/me` and is held in `AuthenticationStateSnapshot` / `AuthenticationStore`.
- **Task:** Highlight the current scope in the navbar scope dropdown.

---

## Epic 2 – Entity-based permissions

### Decisions

- **Source of permissions:** The frontend obtains effective permissions from **`GET /api/auth/me`** with scope headers. The response includes a `permissions` array (model-driven keys, e.g. `organization.read`, `user.create`). This is already implemented: `AuthenticationStoreLive.fetchMe` calls `/api/auth/me`, and the parsed state includes `permissions`; `AuthenticationStateSnapshot` exposes them.
- **Permission-driven UI:** This is a **gap in the model**. Permissions are now **model-driven** (e.g. `organization.read`, `user.create`) rather than screen-driven (e.g. `dashboard.users.read`). Mapping “which UI parts to show/hide or disable” from model permissions is undefined. A **proper solution is required** (e.g. a convention or a small mapping layer: screen/action → required permission key; then check `state.permissions`).

### Implementation notes

- No concrete pattern (hide vs disable vs show and 403) is fixed yet; the solution will define that.
- **Task:** Design and document the model-driven permission → UI mapping; then implement it (e.g. helper or hook that takes required permission key(s) and returns whether to show/disable).

---

## Epic 5 – Generic REST handler

### Decisions

- **Migration order:** Align **authentication** first, then **home**, then **profile**, then **dashboard** (in that order).
- **UI approach:** Build a **generic CRUD UI** (list/get/create/update/delete driven by entity_id and query spec). **Customize or extend** it per screen where required, rather than building fully custom screens per entity.

### Implementation notes

- Data fetching will use **REST** (not RPC) for list/get/create/update/delete. A **new Effect.ts service** (or services) for the generic entity API is likely needed so the frontend can call `GET/POST /api/entities/:entity_id` and `GET/PATCH/DELETE /api/entities/:entity_id/:id` with the same contract (filter, sort, pagination) as the backend.
- **Tasks:** Implement generic entity REST client/service; build generic CRUD UI component(s); align auth, home, profile, and dashboard to use them in order.

---

## Epic 7 – RPC parity and streaming

### Decisions

- **Data:** Use **REST** for CRUD and list/get (see Epic 5).
- **Streaming:** Use **HTTP/2** (or the same transport) for **streaming updates** to the client (invalidation events). A **new Effect.ts service** for streaming is likely needed (and possibly one for REST entity API as above).
- **RPC for entity operations:** Not required on the frontend for now; REST is sufficient for data. RPC may be used later for subscribe (see Epic 8).

### Implementation notes

- The backend exposes **GET /api/subscriptions/stream** for invalidation events (see Epic 8). So the “streaming” service is primarily for that SSE stream. HTTP/2 is the intended transport where available.

---

## Epic 8 – Subscriptions

### Decisions

- **Stream contract (from backend):** The client opens the invalidation stream at **GET /api/subscriptions/stream**. It is a long-lived **SSE** stream (`Content-Type: text/event-stream`). The server sends:
  - First event: `data: {"type":"ready"}\n\n`
  - Then one event per invalidation: `data: {"subscription_id":"<uuid>"}\n\n`
- **Subscribe:** To create a subscription, the client calls **POST /api/rpc** with body `{ method: "subscribe", entity_id: "<model_id>", params: { filter, sort, order, offset, limit } }`. The server validates params per model (ListQuerySpec) and returns `{ result: { subscription_id: "<uuid>" } }`. The client must **store the subscription_id** and the **params used** for that subscription so it can refetch when it receives an invalidation for that id.
- **Refetch strategy:** On each invalidation event, **refetch with the same parameters** as the original fetch for that subscription. **No separate cache layer to invalidate;** replace the frontend state with the result of the refetch (keep behavior uniform).
- **Views to make live first:** Same order as Epic 5 — **authentication** (if applicable), then **home**, **profile**, then **dashboard**.

### Implementation notes

- Client flow: (1) Call subscribe RPC with entity_id + params; (2) Open GET /api/subscriptions/stream (with auth headers); (3) On `ready`, consider stream connected; (4) On each `subscription_id` event, find the subscription by id and refetch using the stored params; (5) Replace state with the new data.
- **Tasks:** Effect.ts service(s) for REST entity API and for subscription stream; subscribe RPC call; open stream and handle events; refetch-by-subscription-id with same params; wire into auth, home, profile, dashboard in order. Epic 10 (E2E) is skipped for now.

---

## Epic 10 – End-to-end

### Decisions

- **E2E:** **Skipping E2E as a whole for now.** No Playwright (or other) E2E tests for the full path (login → scope → list → subscribe → CUD → invalidation/refetch) in the current plan.

### Implementation notes

- When E2E is reintroduced, the backend and frontend contracts in this doc and in the technical-choices should be enough to define the scenarios.

---

## Summary table

| Epic | Key choices |
|------|-------------|
| 1 | Token in localStorage; scope selector exists; navbar dropdown shows scopes but **current scope not highlighted** → fix. |
| 2 | Permissions from `GET /api/auth/me`; **gap:** model-driven permissions → UI mapping needs a proper solution. |
| 5 | Migration order: auth → home → profile → dashboard. **Generic CRUD UI**, extend per screen. New Effect.ts service for REST entity API. |
| 7 | REST for data; HTTP/2 for streaming. New Effect.ts service(s) for REST and streaming. |
| 8 | Stream: **GET /api/subscriptions/stream** (SSE). Subscribe: **POST /api/rpc** method `subscribe`. Refetch with **same params**; replace state. Live views: auth, home, profile, dashboard. |
| 10 | E2E skipped for now. |
