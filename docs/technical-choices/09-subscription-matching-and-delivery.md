# Technical choices: Epic 9 — Subscription matching and delivery

This document records technical decisions for the ninth deliverable (subscription matching and delivery). It fixes how change events are processed, where subscriptions are stored, and how the write path stays non-blocking.

**Epic reference:** [09_subscription-matching-and-delivery](../deliverables/09_subscription-matching-and-delivery.md)

**Implementation:** After create/update/delete, the generic handler calls `SubscriptionStore::publish_change(ChangeEvent { model_id, resource_id, action, organization_id })` (fire-and-forget). A **change worker** is spawned at startup (`SubscriptionStore::spawn_change_worker()`) and receives change events; for each event it calls `notify_affected_by(event)`, which uses **subscriptions_affected_by(event)** (scope-aware: same model_id, and org match or platform-level) then **send_invalidation** for each affected subscription id. Wired in `build_router_for_test` and in `server/main.rs`.

---

## 1. Use existing worker / Task system

- **Choice:** Matching and delivery use the **existing worker / Task system**. The same system is used for other background work; its backend and how it scales (e.g. in-process, out-of-process, distributed) depend on **configuration**. Subscriptions are built on the **cache layer**: the cache layer is the abstraction that holds subscription state and that the worker (or matching logic) uses. How that cache is backed (in-memory vs Redis, etc.) is configuration-dependent, same as elsewhere in the system.
- **Implication:** No new queue or worker framework for this epic. The write path publishes a change event (or enqueues a task); the worker runs matching and delivery. Configuration determines whether the worker runs in the same process or in an independent process and whether the subscription store is in-memory or shared (e.g. Redis).

---

## 2. Subscription store: cache layer (in-memory or Redis per config)

- **Choice:** The **subscription store** (which subscriptions exist and their query + scope + stream/connection reference) lives in the **cache layer**. The cache layer is in-memory by default but may use Redis (or similar) depending on configuration. A worker in an **independent process** can then read from that cache when processing change events, so matching can run out-of-process when configured that way.
- **Implication:** Subscribe/unsubscribe update the cache layer. Matching reads active subscriptions from the cache (filtered by entity and scope as needed). The same cache abstraction used elsewhere is reused for subscriptions so that configuration (in-memory vs Redis) applies consistently.

---

## 3. Write path: publish event, async matching and delivery

- **Choice:** The write path (e.g. generic handler after create/update/delete) **publishes a change event** (e.g. entity type, row id, operation, and scope-relevant data such as org_id) and **returns** without waiting for subscription logic. Matching and delivery run **asynchronously** (via the worker/Task system). The write path does not block on matching or on pushing invalidation to streams.
- **Implication:** After persisting the change, the handler enqueues a task or publishes an event. The worker picks it up, runs matching (which subscriptions are affected, scope-aware), and performs delivery. Response to the client is not delayed by subscription work.

---

## 4. Delivery: in-process for now; document future options

- **Choice:** **Right now delivery is in-process**: the component that holds the client’s HTTP/2 stream and the component that runs matching/delivery are in the same process. Invocations to “push invalidation to this stream” are in-process. The design **leaves open** moving to cross-process delivery later (e.g. worker publishes “invalidate subscription X”; the process that owns the stream subscribes and pushes to the stream, or routing by connection/stream id to the correct process). This is **not** prescribed in this epic; it is called out so that we can make the right decision later (e.g. when scaling to multiple nodes).
- **Implication:** Implementation keeps matching and delivery in-process. Comments and this document note that delivery may later be refactored to support cross-process (e.g. when the stream holder is in a different process than the worker). No change to the external contract (invalidation hint, one per subscription).

---

## 5. Non-blocking write path (confirmed)

- **Choice:** The write path is **not** blocked by subscription logic. The handler responds to the client after persisting the change and enqueuing the change event (or task). Matching and delivery run asynchronously.
- **Implication:** No synchronous call from the write handler into matching or delivery. Confirmed as above.

---

## 6. Scope-aware matching

- **Choice:** Matching is **scope-aware**. When determining “which subscriptions are affected by this change,” we only include subscriptions whose **scope** allows them to see the changed row (e.g. the row’s org_id is in the subscription’s scope). We never send an invalidation to a client whose scope cannot see the changed data. The change event includes scope-relevant fields (e.g. org_id); the subscription store stores each subscription’s scope; the matching logic compares them.
- **Implication:** No leakage: a subscriber only receives invalidation hints for data they are permitted to read in their current scope. Authorization is enforced at subscribe time (they must have `<entity>.read` in scope) and again at matching time (the changed row must be in that scope).

---

## Summary table

| Topic | Choice |
|-------|--------|
| Worker | Use existing worker/Task system; backend and scaling by configuration. |
| Subscription store | Cache layer (in-memory by default, may use Redis); worker can run in separate process and read from cache. |
| Write path | Publish change event and return; matching and delivery async via worker. |
| Delivery | In-process for now; document that cross-process delivery may be added later. |
| Non-blocking | Confirmed; write path does not wait for matching or delivery. |
| Matching | Scope-aware; only invalidate subscriptions whose scope can see the changed row. |

---

## 7. Organization entity: platform-level invalidation, scope-dependent data

- **Choice:** For the **organization** entity, the list is one logical query (same entity + params). **Invalidation** is **platform-level**: any create/update/delete of an organization publishes a change event with `organization_id: None`, so all subscriptions watching `"organization"` are invalidated and every subscriber refetches. **Data** returned on list/get is **scope-dependent**: the read path filters so that org-scoped requests only see the organization(s) in their scope (e.g. list returns only `scope.organization_id`, get returns 404 for other org ids).
- **Implication:** Subscribers always receive invalidation when any org changes; what they see after refetch depends on their request scope. Multi-tenancy is enforced on the read path, not by restricting who gets invalidated.
