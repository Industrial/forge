# Epic 1: Scoped session and profile selection

## Summary

Authenticated users with more than one org/role can choose a single “session scope” (org + role). That choice is persisted (e.g. across reloads) and is the only scope used for all subsequent requests until the user changes it.

## Functionality

- **Profile selection requirement:** After login, if the user has multiple org/role memberships, they are required to choose a scope before using the rest of the app (e.g. a dedicated profile/scope selection step).
- **Scope persistence:** The chosen scope is persisted (e.g. across reloads) so that the user does not have to select again until they explicitly switch or log out.
- **Request context:** Every API request is made in the context of that scope (no implicit or default scope for multi-org users).
- **UI reflection:** The UI always reflects the current scope (e.g. who is acting as whom) and allows switching scope; after a switch, all subsequent requests use the new scope and the UI updates accordingly.

## Dependencies

- None (first deliverable).

## Succeeds to

- Epic 2 (Entity-based permissions) and all later epics rely on a stable, persisted session scope.
