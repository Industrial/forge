# Technical choices: Epic 10 — End-to-end live, authorized, scoped entities

This document records technical decisions for the tenth (final) deliverable: end-to-end integration of live, authorized, scoped entities via REST, RPC, and subscriptions.

**Epic reference:** [10_end-to-end-live-authorized-scoped-entities](../deliverables/10_end-to-end-live-authorized-scoped-entities.md)

---

## Scope of this epic

Epic 10 does **not** introduce new subsystems or new technical mechanisms. All behaviour is defined by Epics 1–9:

- **Epic 1:** Scoped session and profile selection (client scope, zero-trust, forced profile selection for multi-org).
- **Epic 2:** Entity-based permissions (`<entity>.<action>`, scope-aware).
- **Epic 3:** Entity registry (code-defined, metadata, filter/sort from model).
- **Epic 4:** Unified query specification (filter, sort, pagination; scope separate).
- **Epic 5:** Single generic REST handler (entity + action + query spec; authz per request).
- **Epic 6:** No embedded relations (IDs only; expand/include → 400).
- **Epic 7:** RPC parity with REST (HTTP/2, per-request identity/scope, same transport for subscribe).
- **Epic 8:** Subscriptions to queries (subscribe/unsubscribe RPC; invalidation over HTTP/2 stream).
- **Epic 9:** Subscription matching and delivery (worker + cache; scope-aware; in-process delivery for now).

Epic 10 is the **integration and validation** epic: one consistent model, authorization on every path, no leakage, and the full path working together.

---

## Integration contract (from deliverable)

- **Unified model:** Entities are readable/writable via REST and RPC, and subscribable via the same query specification, with one consistent model of scope and entity permissions.
- **Authorization everywhere:** Every operation (REST, RPC, subscription) is authorized using entity-based permissions and the current session scope.
- **No leakage:** A subscriber receives invalidation only for data they are permitted to read in that scope.
- **Complete path:** Scoped session → entity registry → entity permissions → unified query spec → generic REST → no embedded relations → RPC parity → query subscriptions → matching/delivery, all working together.

No additional technical choices are prescribed here; implementation follows the prior epic docs.

---

## Integration decisions

1. **Acceptance / testing:** Use the **existing end-to-end test suite** (Playwright). Extend it to cover the full path (login → scope selection → REST list → subscribe → write → receive invalidation) with multiple scopes and permissions where needed. No separate E2E framework.
2. **Rollout / feature flags:** **No feature flags.** This is the core product; the new stack (generic REST, RPC, subscriptions) is the default behaviour.
3. **Documentation:** **Skip** a dedicated “Live entities overview” doc for now.
