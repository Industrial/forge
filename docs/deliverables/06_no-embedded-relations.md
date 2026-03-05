# Epic 6: No embedded relations in API responses

## Summary

List and get responses return only the requested entity’s data. Relationships are represented by identifiers. Clients obtain related data via separate requests.

## Functionality

- **No nesting:** List and get responses return only data for the requested entity (one “table”/resource); they do not embed or expand related entities.
- **Identifiers for relations:** Relationships are represented by identifiers (e.g. foreign keys) in the response.
- **Client composition:** To obtain related data, the client issues separate requests (e.g. list or get) for the related entity, using those identifiers in the query (e.g. filter).
- **Contract:** This contract applies to all entities and is documented as the API design; no endpoint returns nested graphs of related entities.

## Dependencies

- Epic 5 (Single generic REST handler) — the handler and API contract enforce this behavior.

## Succeeds to

- Epic 7–10 (RPC and subscriptions use the same contract).
