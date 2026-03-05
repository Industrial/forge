# Epic 10: End-to-end live, authorized, scoped entities

## Summary

Entities are readable and writable via REST and RPC, and subscribable via the same query specification, with one consistent model of scope and entity permissions. Every operation is authorized; subscribers receive only what they are allowed to read.

## Functionality

- **Unified model:** Entities are readable and writable via REST and RPC, and subscribable via the same query specification, with one consistent model of scope and entity permissions.
- **Authorization everywhere:** Every operation (REST, RPC, or subscription) is authorized using entity-based permissions and the current session scope.
- **No leakage:** A subscriber receives updates only for entities and rows they are permitted to read in that scope; no updates are delivered for data the subscriber is not allowed to read.
- **Complete path:** The full path is in place: scoped session, model registry (entity_id in API), entity permissions, unified query spec, generic REST handler (list/get for all models; CUD for organization), no embedded relations, RPC parity (entity.list/get/create/update/delete), query subscriptions (params validated per model), and subscription matching/delivery (change events, scope-aware matching, worker)—all working together for live, authorized, scoped entities.

## Rollout & documentation

Per [technical-choices §2–§3](../technical-choices/10-end-to-end-live-authorized-scoped-entities.md): **no feature flags** (new stack is default); **skip** a dedicated “Live entities overview” doc for now.

## Dependencies

- Epics 1–9 (all prior deliverables).

## Succeeds to

- None (final epic in this sequence).
