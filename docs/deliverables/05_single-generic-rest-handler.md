# Epic 5: Single generic REST handler for entities

## Summary

All entity CRUD operations are served by one generic handler. The handler dispatches by entity name and action, uses the unified query spec for list, and enforces entity-based authorization.

## Functionality

- **Single handler:** All entity CRUD operations (list, get by id, create, update, delete) are served by one generic handler (or one dispatch layer); there is no per-entity handler code.
- **Dispatch:** The handler identifies the entity and action from the request (e.g. path or body).
- **List:** List uses the unified query specification.
- **Get:** Get returns a single entity by id.
- **Create/update:** Create and update accept a body and are validated.
- **Delete:** Delete removes by id.
- **Validation:** Validation rules for create/update are derived from the entity (e.g. from schema/metadata), not hand-coded per entity.
- **Authorization:** Authorization for each request is based on entity-based permissions and the current scope.
- **Registry:** The handler uses the entity registry to know which entities exist and how to serve them.

## Dependencies

- Epic 2 (Entity-based permissions), Epic 3 (Entity registry), Epic 4 (Unified query specification).

## Succeeds to

- Epic 6 (No embedded relations), Epic 7 (RPC parity), Epic 8–10 (subscriptions and live).
