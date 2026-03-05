# Epic 4: Unified query specification (filter, sort, pagination)

## Summary

Every list operation for an entity accepts one and the same kind of query specification. The same specification is used wherever list-style data is requested (one-shot list and later subscriptions).

## Functionality

- **Single contract:** Every “list” operation for an entity accepts one and the same kind of query specification.
- **Components:** The specification includes: which rows to return (filter), in what order (sort), and which slice to return (pagination).
- **Consistency:** Filter, sort, and pagination are defined in a consistent way for all entities; per-entity configuration only restricts which fields may be used for filter/sort.
- **Reuse:** The same query specification is used everywhere list-style data is requested (one-shot list, and later subscriptions).
- **Bounded:** The specification is explicit and bounded (no arbitrary graph traversal or unbounded depth).

## Dependencies

- Epic 3 (Entity registry) — entities define which fields are allowed for filter/sort.

## Succeeds to

- Epic 5 (Generic REST handler), Epic 8 (Subscriptions to queries), Epic 9 (Subscription matching and delivery).
