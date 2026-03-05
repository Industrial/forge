# Epic 5: Single generic REST handler for entities

## Summary

All entity CRUD operations are served by one generic handler. The handler dispatches by entity name and action, uses the unified query spec for list, and enforces entity-based authorization.

## Functionality

- **Single handler:** All model CRUD operations (list, get by id, create, update, delete) are served by one generic handler; there is no per-model handler code. List and get use default RestModel trait implementations for all registered models (organization, user, role, permission, audit); only organization has full create/update/delete via the handler today.
- **Dispatch:** The handler identifies the model (entity_id in path) and action from the request.
- **List:** List uses the unified query specification.
- **Get:** Get returns a single resource by id.
- **Create/update:** Create and update require a JSON object body; per-model validation (e.g. DTO deserialization) applies where implemented.
- **Delete:** Delete removes by id.
- **Validation:** Request body must be a JSON object (400 otherwise); per-model validation is in the RestModel impl (e.g. Organization uses DTOs).
- **Authorization:** Authorization for each request is based on entity-based permissions and the current scope.
- **Registry:** The handler uses the registry (RestModel implementors) to know which models exist and how to serve them. After successful CUD, a change event is published for subscription matching (Epic 9).

## Dependencies

- Epic 2 (Entity-based permissions), Epic 3 (Entity registry), Epic 4 (Unified query specification).

## Succeeds to

- Epic 6 (No embedded relations), Epic 7 (RPC parity), Epic 8–10 (subscriptions and live).
