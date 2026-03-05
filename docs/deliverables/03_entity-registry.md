# Epic 3: Entity registry

## Summary

The system has a single registry of entities that are exposed for CRUD and (later) subscriptions. The registry is the source of truth for permission keys and for what the API serves.

## Functionality

- **Single registry:** The system has a single registry of “entities” that are exposed for CRUD and (later) subscriptions.
- **Entity identity and metadata:** Each entity has a stable identifier and metadata (e.g. which actions are supported, how it is named for permissions and API).
- **Source of truth:** The registry is the single source of truth for: which permission keys exist, which entities the API serves, and (later) which entities can be subscribed to.
- **Minimal onboarding:** Adding or removing an entity in the registry is the only step required to change the permission set and the API surface for that entity (no per-entity handler code).

## Dependencies

- Epic 2 (Entity-based permissions) — permissions are derived from the registry.

## Succeeds to

- Epic 4 (Unified query spec), Epic 5 (Generic REST handler), Epic 8–10 (subscriptions and live).
