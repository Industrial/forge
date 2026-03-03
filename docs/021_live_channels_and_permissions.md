# Live channels and permission-based subscription (framework contract)

This document defines how **channel names** and **permissions** map for the forge-live real-time system. The server derives WebSocket subscriptions from the session; the client does not send a subscription list.

## 1. Channel naming

- **One channel per permission scope**: A user receives live updates only for data they are allowed to view. Channels are named so that having a given permission implies subscription to exactly one channel (or a fixed set) for that scope.

- **Per org and per resource type** (org-scoped resources):  
  Channel format: `org:{org_id}:{resource_type}`  
  Examples: `org:550e8400-e29b-41d4-a716-446655440000:users`, `org:...:roles`, `org:...:role_permissions`.

- **Global channels** (not scoped to an organization):  
  Use a single literal name.  
  Examples: `tasks`, `audit-log`, `organizations`.

| Resource / feature   | Channel name pattern              | Scope   |
|----------------------|-----------------------------------|---------|
| Users in an org      | `org:{org_id}:users`              | Org     |
| Roles in an org      | `org:{org_id}:roles`              | Org     |
| Role permissions     | `org:{org_id}:role_permissions`   | Org     |
| Tasks (demo)         | `tasks`                           | Global  |
| Audit log            | `audit-log`                       | Global  |
| Organizations list   | `organizations`                   | Global  |

## 2. Permission → channel mapping

- The server loads the **current session** (user id, `current_org_id`, and the set of permissions for that org, or equivalent).
- For each **permission that grants view access** to a resource type in that org, the server subscribes the WebSocket connection to the corresponding channel:
  - Permission to view **users** in org O → subscribe to `org:{O}:users`
  - Permission to view **roles** in org O → subscribe to `org:{O}:roles`
  - Permission to view **role_permissions** in org O → subscribe to `org:{O}:role_permissions`
- For **global** resources, if the user has permission to view them, subscribe to the global channel:
  - Permission to view **tasks** → subscribe to `tasks`
  - Permission to view **audit log** → subscribe to `audit-log`
  - Permission to view **organizations** (e.g. list orgs) → subscribe to `organizations`
- If the user has **no current org** (e.g. `current_org_id` is null), only global channels they are allowed to see are subscribed.

## 3. Broadcast (server) contract

- When a **mutation** occurs (create/update/delete), the handler broadcasts to the **channel that matches the resource and scope**:
  - Mutate users in org O → broadcast to `org:{O}:users`
  - Mutate roles in org O → broadcast to `org:{O}:roles`
  - Mutate role_permissions in org O → broadcast to `org:{O}:role_permissions`
  - Mutate organizations (create/update/delete org) → broadcast to `organizations`
  - Tasks tick / audit log entry → broadcast to `tasks` or `audit-log` respectively.
- Payload format is defined by the forge-live `LiveEvent` enum (e.g. `UsersUpdated`, `ResourceChanged`). At least-once delivery is acceptable.

## 4. Forge-live API alignment

- Use `Channel::org(org_id)` only when the app intentionally uses a **single org channel** for all org-scoped events (legacy or simplified model).
- For **per–resource-type** channels, use a channel name consistent with this contract:  
  `org:{org_id}:{resource_type}` can be built via `Channel::raw(format!("org:{}:{}", org_id, resource_type))` or a helper such as `Channel::org_resource(org_id, resource_type)` if added to forge-live.

## 5. Summary

| Concern              | Rule |
|----------------------|------|
| Channel naming       | `org:{org_id}:{resource_type}` for org-scoped; literal for global (`tasks`, `audit-log`, `organizations`). |
| Who subscribes       | Server, from session (current org + permissions). |
| Who sends subscribe  | No client subscribe list; server subscribes the connection on connect. |
| Broadcast target    | One channel per (org, resource_type) or per global resource. |

This contract ensures that **any user with permission to view a resource automatically receives live updates** for that resource when any other user (or the system) mutates it, without per-page or per-component subscription logic on the client.

---

## 6. WebSocket handshake and server-side subscription

- **On connect**: The WebSocket handler must **authenticate** the request (e.g. session cookie or token). If the user is not authenticated, reject the upgrade (e.g. 401/403) so the WebSocket is never established.
- **After upgrade**: Using the authenticated session, the server loads the user’s **current org** and **permissions** (or equivalent). It then subscribes the connection to exactly the channels implied by §2 (permission → channel). The client **does not** send a list of channels to subscribe to; the server derives subscriptions from the session.
- **Client role**: The client opens a single WebSocket and receives all messages the server pushes for the channels it was subscribed to. Optional: client may send heartbeat/ping; the client does not send `subscribe` / `unsubscribe` messages for channel management.
- **Reconnect**: If the client reconnects (e.g. after network drop), the server again authenticates and re-applies the same subscription logic so the new connection gets the same channel set.
