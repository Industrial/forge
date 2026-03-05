# Technical choices: Epic 5 — Single generic REST handler for entities

This document records technical decisions for the fifth deliverable (single generic REST handler). It fixes how the handler is exposed, how it dispatches, and how it applies the query spec, authorization, and validation.

**Epic reference:** [05_single-generic-rest-handler](../deliverables/05_single-generic-rest-handler.md)

**Migration:** For how to migrate from legacy routes (`/api/users`, `/api/organizations`, etc.) to the generic handler or keep both, see [Migration path: legacy routes → generic entity handler](../migration-legacy-routes-to-generic-entity-handler.md).

**Implementation (trait-based):** The generic handler lives in `handlers/generic_entity.rs`. It performs auth, scope, and query-spec parsing, then calls into the **registry** module (`db/src/registry.rs`). The registry dispatches by **model_id** (path param `entity_id`) to the appropriate **RestModel** implementor (in `db/src/models/`). Each model defines metadata (model_id, filter/sort/response fields, display name, supported_actions) and required hooks (row_to_json, apply_filter, default_sort, apply_sort); **list** and **get** use default trait implementations; **create**, **update**, **delete** can be overridden (e.g. Organization) or return "not supported" (user, role, permission, audit). The generic handler serves all registered models (organization, user, role, permission, audit); only organization has full CUD via the handler today. Request body for create/update must be a JSON object (400 otherwise). After successful CUD, a **change event** is published (Epic 9). Unknown entity_id → 404.

---

## 1. Single handler, dispatch by entity and action

- **Choice:** All entity CRUD is served by **one** generic handler (or one dispatch layer). The handler identifies the **entity** (e.g. from path or body) and the **action** (list, get, create, update, delete), looks up the entity in the registry, and runs the same logic for every entity. There is no per-entity handler code.
- **Implication:** Routing maps a small set of path patterns (or one pattern with entity_id as a parameter) to this handler. **In the trait-based implementation:** the handler calls `registry::is_known_model(entity_id)` for 404, then `registry::list_models`, `registry::get_model`, `registry::create_model`, `registry::update_model`, or `registry::delete_model`, which dispatch by match on model_id to the corresponding `RestModel` impl.

---

## 2. Route and URL shape

- **Choice:** A consistent URL shape is used for all entities. For example: **list** `GET /api/entities/:entity_id` (or `GET /api/:entity_id`), **get** `GET /api/entities/:entity_id/:id`, **create** `POST /api/entities/:entity_id`, **update** `PATCH /api/entities/:entity_id/:id` (or `PUT`), **delete** `DELETE /api/entities/:entity_id/:id`. The exact prefix (`/api/entities` vs `/api`) is implementation-defined; the important point is that **entity_id** is a path parameter and the same handler serves every entity_id that exists in the registry.
- **Implication:** Unknown entity_id → 404 (per Epic 3). The handler parses entity_id from the path and looks it up in the registry before performing any action.

---

## 3. List: query spec on request

- **Choice:** List accepts the unified query specification (filter, sort, pagination) as defined in Epic 4. The spec may be sent via **query string** (e.g. for GET) or in a **request body** (e.g. for POST if list is exposed as POST). The same logical structure is used; wire format is implementation-defined. Scope is never taken from the query spec; it is taken from request context (headers).
- **Implication:** The handler applies scope from context, then applies filter/sort/pagination. Invalid spec → 400/422 per Epic 4. Response includes the list of rows (entity data only, no embedded relations per Epic 6) and pagination metadata (e.g. next cursor, total count if applicable).

---

## 4. Get, create, update, delete

- **Choice:**  
  - **Get:** `GET /api/entities/:entity_id/:id` returns a single row by primary key (or canonical id). 404 if not found or if the row is outside the request scope.  
  - **Create:** `POST /api/entities/:entity_id` with body; validation from entity model/schema; 400/422 on validation failure.  
  - **Update:** `PATCH /api/entities/:entity_id/:id` (or PUT) with body; partial or full update; validation from model/schema; 404 if not found, 400/422 on validation failure.  
  - **Delete:** `DELETE /api/entities/:entity_id/:id`; 404 if not found or outside scope; 204 or 200 with minimal body on success.
- **Implication:** All actions check entity permission (Epic 2): list/get → `<entity>.read`, create → `<entity>.create`, update → `<entity>.update`, delete → `<entity>.delete`. Scope is applied so that only rows within scope are visible or mutable.

---

## 5. Validation from model/schema

- **Choice:** Create and update validation (required fields, types, constraints) is **derived from the entity’s model/schema** (e.g. SeaORM or DB schema). No hand-coded validation per entity. The registry may optionally override or extend (e.g. custom rules); the base is derivation from the model.
- **Implication:** The handler (or a shared validation layer) uses the registry’s backing model/schema to validate the request body. Invalid payload → 400 or 422 with clear errors.

---

## 6. No embedded relations in responses

- **Choice:** List and get responses return **only** the requested entity’s columns (one “table”/resource). No embedded or expanded related entities. Relationships are represented by their identifiers (e.g. foreign key columns) in the response. This matches Epic 6.
- **Implication:** The generic handler never joins or loads related entities for inclusion in the response. Clients that need related data issue separate list/get requests with the appropriate filter (e.g. by foreign key).

---

## 7. Authorization and scope

- **Choice:** Before executing any action, the handler checks that the authenticated identity has the required entity permission in the current scope (Epic 2). Scope is taken from request context (headers); the handler applies it to restrict which rows are visible and mutable. Scope overrides any scope-like criteria in the query spec (Epic 4).
- **Implication:** 403 when the user lacks the required permission. 404 when the resource is not found or is outside the user’s scope. No leakage of existence of out-of-scope rows.

---

## 8. Error handling

- **Choice:**  
  - **Unknown entity_id:** 404 Not Found (or 400 Bad Request with a clear message).  
  - **Missing or invalid permission:** 403 Forbidden.  
  - **Invalid query spec (filter/sort/pagination):** 400 or 422 per Epic 4.  
  - **Validation failure (create/update body):** 400 or 422 with field-level errors where applicable.  
  - **Resource not found (get/update/delete):** 404.  
  - **Conflict or constraint violation:** 409 Conflict or 422 as appropriate.
- **Implication:** Consistent error response shape (e.g. JSON with `error` or `errors`) so that clients can handle them uniformly.

---

## Summary table

| Topic | Choice |
|-------|--------|
| Handler | Single generic handler; dispatch by entity_id + action. |
| URL shape | Consistent pattern with entity_id and (for get/update/delete) id in path. |
| List | Query spec (filter, sort, pagination) on request; scope from context. |
| Get/Create/Update/Delete | Standard semantics; validation from model/schema. |
| Validation | Derived from entity model/schema; optional overrides via registry. |
| Responses | Entity data only; no embedded relations (Epic 6). |
| Authz and scope | Entity permission check; scope from context; scope overrides filter. |
| Errors | 404 unknown entity / not found; 403 forbidden; 400/422 validation/spec. |
