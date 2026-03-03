# REST API resource map

Basic REST structure for **users**, **organizations**, and **roles**. Plural collection names; nested resources where ownership is clear. No scope or permission checks—endpoints are unprotected for now.

---

## 1. Top-level resources

| Method | Path | Description |
|--------|------|-------------|
| **Users** | | |
| GET | `/api/users` | List users. |
| POST | `/api/users` | Create user (body: email, password; optional: create personal org or add to org). |
| GET | `/api/users/:id` | Get one user. |
| PATCH | `/api/users/:id` | Update user (email, is_active, etc.). |
| DELETE | `/api/users/:id` | Delete user and their memberships. |
| **Organizations** | | |
| GET | `/api/organizations` | List organizations. |
| POST | `/api/organizations` | Create organization (name, slug). |
| GET | `/api/organizations/:id` | Get one organization. |
| PATCH | `/api/organizations/:id` | Update organization. |
| DELETE | `/api/organizations/:id` | Delete organization. |

---

## 2. Nested under organizations

### 2.1 Organization → users (members in this org)

| Method | Path | Description |
|--------|------|-------------|
| GET | `/api/organizations/:id/users` | List users in this org (members). |
| POST | `/api/organizations/:id/users` | Add user to org: create membership + role(s). Body: `user_id` (existing) or `email`+`password` (create and add), plus `role_ids`. |
| GET | `/api/organizations/:id/users/:userId` | Get user’s membership/roles in this org (or 404). |
| PATCH | `/api/organizations/:id/users/:userId` | Update user’s role(s) in this org. |
| DELETE | `/api/organizations/:id/users/:userId` | Remove user from org (delete membership and user_org_role rows). |

### 2.2 Organization → roles

| Method | Path | Description |
|--------|------|-------------|
| GET | `/api/organizations/:id/roles` | List roles for this org. |
| POST | `/api/organizations/:id/roles` | Create role (name, display_name). |
| GET | `/api/organizations/:id/roles/:roleId` | Get one role. |
| PATCH | `/api/organizations/:id/roles/:roleId` | Update role. |
| DELETE | `/api/organizations/:id/roles/:roleId` | Delete role (fail if any user has it). |

---

## 3. Nested under users

| Method | Path | Description |
|--------|------|-------------|
| GET | `/api/users/:id/organizations` | List organizations this user belongs to (memberships with org + role info). |
| or | `/api/users/:id/memberships` | Same: list memberships (org_id, role_ids/names). |

Use one of these for “user’s orgs” so callers don’t need to hit each org’s `/users` list.

---

## 4. Audit and session

| Method | Path | Description |
|--------|------|-------------|
| GET | `/api/audit-log` | List audit log entries (query: org_id, filters). |
| GET | `/api/auth/session` | Current session (user, profiles). |
| GET | `/api/users/me` | Current user (alias or redirect). |

---

## 5. Complete URL tree (summary)

```
/api
├── users
│   ├── GET, POST                    /api/users
│   ├── GET, PATCH, DELETE           /api/users/:id
│   └── GET                          /api/users/:id/organizations  (or /api/users/:id/memberships)
│
├── organizations
│   ├── GET, POST                    /api/organizations
│   ├── GET, PATCH, DELETE           /api/organizations/:id
│   └── :id/
│       ├── users
│       │   ├── GET, POST            /api/organizations/:id/users
│       │   └── GET, PATCH, DELETE   /api/organizations/:id/users/:userId
│       └── roles
│           ├── GET, POST            /api/organizations/:id/roles
│           └── GET, PATCH, DELETE   /api/organizations/:id/roles/:roleId
│
├── audit-log
│   └── GET                          /api/audit-log
│
└── auth/
    ├── session                      GET  (and login, logout, register, set-profile, etc.)
    └── …
```
