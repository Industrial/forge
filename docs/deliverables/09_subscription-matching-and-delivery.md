# Epic 9: Subscription matching and delivery

## Summary

When model data changes, the handler publishes a change event; a background worker runs scope-aware matching and delivers invalidation hints only to affected subscribers. The write path does not block on matching or delivery.

## Functionality

- **Matching:** When model data changes, the generic handler publishes a change event. A background worker runs scope-aware matching (subscriptions_affected_by) to determine which active subscriptions are affected.
- **Targeted delivery:** For each affected subscription id, the worker sends an invalidation hint; only those subscribers receive an update.
- **Non-blocking:** The write path publishes the change event and returns; matching and delivery run in a spawned worker (spawn_change_worker), so the write path is not blocked.
- **Scalability:** In-process broadcast and worker today; subscription store and delivery can be moved to a cache layer and out-of-process worker per technical-choices.

## Dependencies

- Epic 8 (Subscriptions to queries) — subscriptions must exist to be matched and delivered.

## Succeeds to

- Epic 10 (End-to-end live, authorized, scoped entities).
