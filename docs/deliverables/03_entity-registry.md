# Epic 3: Entity registry

## Summary

The system has a single registry of models (exposed as entity_id in the API) that are exposed for CRUD and subscriptions. The registry is the source of truth for permission keys and for what the API serves.

## Functionality

- **Single registry:** The system has a single registry of “entities” that are exposed for CRUD and (later) subscriptions.
- **Model identity and metadata:** Each model has a stable identifier (model_id) and metadata (e.g. which actions are supported, filter/sort fields, display name). Implementations use the RestModel trait in the db crate.
- **Source of truth:** The registry is the single source of truth for: which permission keys exist, which models the API serves, and which can be subscribed to.
- **Minimal onboarding:** Adding or removing a model in the registry (RestModel impl + registry dispatch) is the only step required to change the permission set and the API surface (no per-model handler code).

## Dependencies

- Epic 2 (Entity-based permissions) — permissions are derived from the registry.

## Succeeds to

- Epic 4 (Unified query spec), Epic 5 (Generic REST handler), Epic 8–10 (subscriptions and live).
