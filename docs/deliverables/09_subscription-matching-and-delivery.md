# Epic 9: Subscription matching and delivery

## Summary

When entity data changes, the system determines which active subscriptions’ result sets are affected and delivers updates only to those subscribers. Matching and delivery are designed to scale and not block the write path.

## Functionality

- **Matching:** When entity data changes, the system determines which active subscriptions’ result sets are affected by that change.
- **Targeted delivery:** Only those subscribers receive an update (or an invalidation hint); others do not.
- **Non-blocking:** Matching and delivery are designed so that the write path (persisting the change) is not blocked by subscription logic.
- **Scalability:** The mechanism scales with the number of subscriptions and the rate of changes (e.g. via separate processing or dedicated components, without prescribing technology).

## Dependencies

- Epic 8 (Subscriptions to queries) — subscriptions must exist to be matched and delivered.

## Succeeds to

- Epic 10 (End-to-end live, authorized, scoped entities).
