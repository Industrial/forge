# Template: route × user_class → expected outcome (source of truth)

Used to derive integration tests: for each (method, path) and user class, the expected HTTP status.

## User classes (seed users)

| Class            | Email               | Org    | Permissions / role |
|------------------|---------------------|--------|---------------------|
| anon             | (none)              | —      | —                   |
| global_admin     | admin@admin.com     | Default| all + is_admin      |
| viewer_default   | viewer@default.org  | Default| users.read, roles.read, permissions.read, audit.read |
| editor_default   | editor@default.org  | Default| + write             |
| admin_default    | orgadmin@default.org| Default| admin               |
| owner_default    | owner@default.org   | Default| owner               |
| viewer_other     | viewer@other.org    | Other  | viewer              |
| owner_other      | owner@other.org     | Other  | owner               |

## Outcome legend

- **200/201/204** = success (method-dependent)
- **401** = unauthenticated
- **403** = forbidden (no permission or wrong scope)
- **404** = not found
- **422** = validation error

## Auth routes

| Method | Path                    | anon | global_admin | viewer_default | … |
|--------|-------------------------|------|--------------|----------------|---|
| POST   | /api/auth/register      | 201/422 | 201/422   | 201/422        | public |
| POST   | /api/auth/login         | 200/401 | 200       | 200            | public |
| GET    | /api/auth/logout        | 200   | 200        | 200            | auth |
| GET    | /api/auth/profile       | 401   | 200        | 200            | auth |
| GET    | /api/auth/profiles      | 401   | 200        | 200            | auth |
| POST   | /api/auth/switch-profile| 401   | 200        | 200            | auth |
| GET    | /api/auth/session       | 200   | 200        | 200            | auth (session shape differs) |
| POST   | /api/auth/tokens        | 401   | 200        | 200            | auth |
| GET    | /api/auth/admin         | 401   | 200        | 403 (non-admin) | admin-only |

## Dashboard routes (permission + scope)

| Method | Path                              | anon | viewer_default | editor_default | admin_default | global_admin |
|--------|-----------------------------------|------|----------------|----------------|---------------|--------------|
| GET    | /api/dashboard/permissions        | 401  | 200            | 200            | 200           | 200          |
| GET    | /api/dashboard/role-permissions   | 401  | 200            | 200            | 200           | 200          |
| POST   | /api/dashboard/role-permissions   | 401  | 403            | 200            | 200           | 200          |
| DELETE | /api/dashboard/role-permissions   | 401  | 403            | 200            | 200           | 200          |
| GET    | /api/dashboard/tasks              | 401  | 200            | 200            | 200           | 200          |
| GET    | /api/dashboard/audit-log         | 401  | 200            | 200            | 200           | 200          |
| GET    | /api/dashboard/organizations      | 401  | 403            | 403            | 403           | 200          |
| POST   | /api/dashboard/organizations      | 401  | 403            | 403            | 403           | 200          |
| GET    | /api/dashboard/users              | 401  | 200 (scoped)   | 200            | 200           | 200          |
| POST   | /api/dashboard/users              | 401  | 403            | 200            | 200           | 200 (org_id=current) |
| PATCH  | /api/dashboard/users/:id          | 401  | 403            | 200            | 200           | 200          |
| DELETE | /api/dashboard/users/:id          | 401  | 403            | 200            | 200           | 200          |
| GET    | /api/dashboard/roles              | 401  | 200            | 200            | 200           | 200          |
| POST   | /api/dashboard/roles              | 401  | 403            | 200            | 200           | 200          |
| PATCH  | /api/dashboard/roles/:id          | 401  | 403            | 200            | 200           | 200          |
| DELETE | /api/dashboard/roles/:id          | 401  | 403            | 200            | 200           | 200          |

## Scope tests (separate)

- viewer@default.org GET /api/dashboard/users → 200, only Default org users (no Other in memberships).
- viewer@other.org GET /api/dashboard/users → 200, only Other org users.
- viewer@default.org POST /api/dashboard/users with org_id=Other → 403.
