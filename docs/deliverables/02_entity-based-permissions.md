# Epic 2: Entity-based permissions

## Summary

Permissions are defined per entity and per action (create, read, update, delete). The set of permission keys is derived from the set of entities. Roles are assigned entity permissions. Authorization uses these permissions consistently.

## Functionality

- **Permission shape:** Permissions are defined per entity and per action: create, read, update, delete (e.g. `<entity>.<create|read|update|delete>`).
- **Derived permission set:** The full set of permission keys is determined by the set of entities (no separate, hand-maintained permission list).
- **Role assignment:** Roles are assigned a subset of these entity permissions (per scope where applicable).
- **Authorization:** Every data operation (read/list, create, update, delete) is allowed or denied based on whether the current identity has the corresponding entity permission in the current scope.
- **Transport consistency:** The permission model is the same for all transports (REST, RPC, and later subscriptions).

## Dependencies

- Epic 1 (Scoped session and profile selection) — authorization and scope must be established.

## Succeeds to

- Epic 3 (Entity registry), Epic 5 (Generic REST handler), Epic 7 (RPC parity), Epic 8–10 (subscriptions and live).
