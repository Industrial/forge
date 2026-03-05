# Epic 7: RPC parity with REST for entity operations

## Summary

The same entity operations (list, get, create, update, delete) can be invoked over a second transport. Authorization and scope apply identically on both transports.

## Functionality

- **Same operations:** The same entity operations (list with query spec, get by id, create, update, delete) can be invoked over RPC (POST /api/rpc with method entity.list, entity.get, entity.create, entity.update, entity.delete), not only over REST. All five are implemented and delegate to the same registry as the generic REST handler.
- **Equivalent shape:** Request and response shapes are logically equivalent to REST (same query spec, same payloads, same identifiers).
- **Same auth:** Authorization and scope apply the same way on both transports: the same identity and scope are used to enforce entity permissions.
- **Client choice:** Clients can choose either transport for any operation; behavior and permissions are consistent.

## Dependencies

- Epic 1 (Scoped session), Epic 2 (Entity-based permissions), Epic 5 (Single generic REST handler), Epic 6 (No embedded relations).

## Succeeds to

- Epic 8 (Subscriptions to queries) — subscriptions can use the same transport and auth model.
