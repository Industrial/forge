# Forge-live migration plan: 100% default template

## Summary
Make the default template use forge-live for all real-time features: tasks, audit-log, and dashboard mutations (users, roles, organizations, role-permissions). Wire format for tasks and audit-log stays the same so existing frontend keeps working.

## File-by-file changes

### 1. `crates/forge/src/app.rs` (framework)
- **Change:** Add `pub fn with_live_query_using(mut self, backend: Arc<forge_live::InMemoryLiveBackend>) -> Self` that sets `self.live_backend = Some(backend)`.
- **Why:** Template needs to create the backend in main and pass it to both the app and the tick loop.

### 2. `crates/forge-cli/templates/default/crates/app/src/main.rs`
- **Change:** Create `let live_backend = Arc::new(forge_live::InMemoryLiveBackend::new())` before building the app. Call `app.with_live_query_using(live_backend.clone())` instead of (or in addition to) no live query. In the tick spawn, pass `live_backend.clone()` and call `task_state.tick(&live_backend).await` (or equivalent).
- **Why:** Backend is shared between the app (for extensions) and the tick loop (for broadcasting task updates).

### 3. `crates/forge-cli/templates/default/crates/app/src/tasks.rs`
- **Change:** Remove `broadcast` and `audit_broadcast` from `TaskState`. Keep `store`. Add `pub async fn tick(&self, live_backend: &Arc<forge::InMemoryLiveBackend>)`: update store as today, then serialize tasks as `{ "type": "tasks", "tasks": tasks }` and call `forge::broadcast_to_channel` with `Channel::raw("tasks")` and the bytes. Update tests to pass a backend or mock.
- **Why:** All task updates go through forge-live; no tokio broadcast channels.

### 4. `crates/forge-cli/templates/default/crates/app/src/handlers/ws.rs`
- **Change:** Handler takes `Extension(Option<Arc<forge::InMemoryLiveBackend>>)` and `Extension(Arc<TaskState>)`. If `live_backend` is None, return 503 or refuse upgrade. On upgrade: register connection with `live_backend.register_connection(tx)`, spawn task that receives from the mpsc and sends to the WebSocket. On client message `{"type":"subscribe","channel":"tasks"}`: subscribe to `Channel::raw("tasks")`, then send initial snapshot from `task_state.store.read().await`. On `"audit-log"`: subscribe to `Channel::raw("audit-log")`. On `"org:<uuid>"`: parse uuid and subscribe to `Channel::org(uuid)`. Remove all use of `task_state.broadcast` and `task_state.audit_broadcast`.
- **Why:** WebSocket connections and subscriptions are managed by forge-live; TaskState only used for reading initial tasks.

### 5. `crates/forge-cli/templates/default/crates/app/src/handlers/auth.rs`
- **Change:** Add `Extension(Option<Arc<forge::InMemoryLiveBackend>>)` to login and logout. Replace `broadcast_audit_entry(&task_state, &event, &result)` with an async `broadcast_audit_entry(live_backend, &event, &result).await` that, if `live_backend` is Some, serializes the entry as `{ "type": "audit_log", "entry": entry }` and calls `forge::broadcast_to_channel(backend, &Channel::raw("audit-log"), &bytes).await`.
- **Why:** Audit log live updates go through forge-live.

### 6. `crates/forge-cli/templates/default/crates/app/src/handlers/dashboard.rs`
- **Change:** Add `Extension(Option<Arc<forge::InMemoryLiveBackend>>)` to every mutation handler: create_role, update_role, delete_role, create_organization, update_organization, delete_organization, create_user, update_user, delete_user, add_role_permission, delete_role_permission. After successful mutation, if `live_backend` is Some, call `forge::broadcast_to_org(backend, org_id, &LiveEvent::...)` or for organizations (no single org) use `broadcast_to_channel(backend, &Channel::raw("organizations"), &event)`. Use `LiveEvent::UsersUpdated`, `LiveEvent::ResourceChanged` as appropriate.
- **Why:** So other browsers can receive live updates when data changes (frontend will subscribe in a follow-up).

### 7. Frontend: optional follow-up
- **Change:** On Users, Roles, Organizations, Role-permissions pages, open WebSocket and send `{"type":"subscribe","channel":"org:<current_org_id>"}` (and for organizations list, `"channel":"organizations"`). On message, if type is users_updated / resource_changed, refetch or merge state. This can be separate bd tasks if desired.

## Wire format (unchanged for tasks and audit-log)
- Tasks: `{ "type": "tasks", "tasks": [...] }` (same as today).
- Audit log: `{ "type": "audit_log", "entry": { ... } }` (same as today).
- Dashboard events: `LiveEvent` JSON (`users_updated`, `resource_changed`, etc.) for pages that subscribe to org channel.

## Dependency
- Template app already depends on `forge`; forge re-exports `broadcast_to_channel`, `broadcast_to_org`, `Channel`, `LiveEvent`, `InMemoryLiveBackend`. No new deps.
