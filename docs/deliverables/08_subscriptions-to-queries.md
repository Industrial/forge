# Epic 8: Subscriptions to queries

## Summary

Clients can subscribe to a query (entity + filter + sort + pagination) and receive updates when the result set of that query changes. Subscriptions are authorized and scoped.

## Functionality

- **Subscribe to a query:** A client can subscribe to a “query”: an entity plus the same kind of specification used for list (filter, sort, pagination).
- **Updates:** After subscribing, the client receives updates when the result set of that query changes (e.g. rows added, removed, or changed that fall within that query).
- **Multiple subscriptions:** The system supports multiple concurrent subscriptions per client and allows subscribe/unsubscribe.
- **Authorization:** Subscriptions are subject to the same authorization and scope as one-shot reads: a client can only subscribe to queries they are allowed to read under the current scope.

## Dependencies

- Epic 1 (Scoped session), Epic 2 (Entity-based permissions), Epic 4 (Unified query specification).

## Succeeds to

- Epic 9 (Subscription matching and delivery), Epic 10 (End-to-end live).
