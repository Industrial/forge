# Functionality and Tests Overview

This document provides a comprehensive overview of all backend functionality, frontend functionality, integration tests, and end-to-end tests in the Forge project.

---

## 1. Backend Functionality

### 1.1 Authentication & Authorization (`handlers/auth.rs`)

#### Endpoints:
- **POST `/api/auth/register`** - User registration
  - Validates email and password
  - Creates new user account
  - Returns 201 Created on success, 422 on validation errors

- **POST `/api/auth/login`** - User login
  - Authenticates user with email/password
  - Returns Bearer token on success
  - Returns 401 Unauthorized for invalid credentials

- **GET `/api/auth/logout`** - User logout
  - Works for both authenticated and anonymous users
  - Returns 200 OK

- **GET `/api/auth/me`** - Get current user info
  - Requires authentication (Bearer token)
  - Returns 401 Unauthorized without token
  - Returns user information for authenticated users

- **GET `/api/auth/profiles`** - List user profiles
  - Returns available organization/role profiles for the user
  - Used for scope selection

- **POST `/api/auth/tokens`** - Create API token
  - Creates a new API token for the authenticated user

- **GET `/api/auth/admin`** - Admin-only endpoint
  - Requires global admin permissions
  - Returns 403 Forbidden for non-admin users
  - Returns "access granted" for admin users

#### Core Functions:
- `resolve_permissions()` - Resolves user permissions from global and org-scoped roles
- `has_global_scope()` - Checks if user has global-scope permission
- `require_entity_permission()` - Authorization helper for entity-based permissions
- Token authentication via `RequireAuth` and `OptionalRequireAuth`
- Scope extraction from headers (`X-Organization-Id`, `X-Role-Id`)

### 1.2 Generic Entity Handler (`handlers/generic_entity.rs`)

#### Endpoints:
- **GET `/api/entities/{entity_id}`** - List entities
  - Supports filter, sort, pagination via query params (offset+limit or cursor+limit; not both)
  - Uses unified query specification; cursor responses include `next_cursor` when more items exist
  - Returns 404 for unknown entity_id
  - Requires entity.read permission

- **POST `/api/entities/{entity_id}`** - Create entity
  - Creates new entity instance
  - Requires entity.create permission
  - Publishes change event after creation

- **GET `/api/entities/{entity_id}/{id}`** - Get entity by ID
  - Returns single entity by ID
  - Requires entity.read permission

- **PATCH `/api/entities/{entity_id}/{id}`** - Update entity
  - Updates existing entity
  - Requires entity.update permission
  - Publishes change event after update

- **DELETE `/api/entities/{entity_id}/{id}`** - Delete entity
  - Deletes entity by ID
  - Requires entity.delete permission
  - Publishes change event after deletion

#### Supported Entities:
- `organization` - Full CRUD support
- `user` - Read-only (list/get)
- `role` - Read-only (list/get)
- `permission` - Read-only (list/get)
- `audit` - Read-only (list/get)

#### Features:
- Unified query specification (filter, sort, pagination)
- Entity-based authorization
- Scope-aware queries (org-scoped or global)
- Rejects expand/include parameters (Epic 6)
- Change event publishing for subscriptions

### 1.3 Dashboard Handlers (`handlers/dashboard.rs`)

#### Endpoints:
- **GET `/api/dashboard/users`** - List users
- **POST `/api/dashboard/users`** - Create user
- **PATCH `/api/dashboard/users`** - Update user
- **DELETE `/api/dashboard/users`** - Delete user

- **GET `/api/organizations/{org_id}/roles`** - List organization roles
- **GET `/api/organizations/{org_id}/roles/{role_id}/permissions`** - Get role permissions
- **POST `/api/organizations/{org_id}/roles/{role_id}/permissions`** - Add permission to role
- **DELETE `/api/organizations/{org_id}/roles/{role_id}/permissions`** - Remove permission from role

#### Features:
- Scope-aware (requires `X-Organization-Id` and `X-Role-Id` headers)
- Permission-based authorization
- Dashboard-specific permissions (`dashboard.users.read`, `dashboard.users.write`, etc.)

### 1.4 RPC Handler (`handlers/rpc.rs`)

#### Endpoint:
- **POST `/api/rpc`** - RPC endpoint

#### Methods:
- `entity.list` - List entities (same as GET `/api/entities/{entity_id}`)
- `entity.get` - Get entity by ID (same as GET `/api/entities/{entity_id}/{id}`)
- `entity.create` - Create entity (same as POST `/api/entities/{entity_id}`)
- `entity.update` - Update entity (same as PATCH `/api/entities/{entity_id}/{id}`)
- `entity.delete` - Delete entity (same as DELETE `/api/entities/{entity_id}/{id}`)
- `subscribe` - Subscribe to entity changes
- `unsubscribe` - Unsubscribe from entity changes

#### Request Format:
```json
{
  "method": "entity.list",
  "entity_id": "organization",
  "params": {
    "filter": "...",
    "sort": "...",
    "offset": 0,
    "limit": 10
  },
  "id": "correlation-id"
}
```

#### Response Format:
```json
{
  "result": {...},
  "id": "correlation-id"  // Echoed if provided in request
}
```

#### Features:
- Same auth/scope as REST (Bearer token + scope headers)
- Supports correlation IDs for request/response matching
- Error responses: `{"error": {"code": 404, "message": "..."}}`
- Parity with REST endpoints

### 1.5 Subscription Stream Handler (`handlers/subscription_stream.rs`)

#### Endpoint:
- **GET `/api/subscriptions/stream`** - Server-Sent Events (SSE) stream

#### Features:
- Long-lived HTTP/2 stream
- SSE format (`data: {...}\n\n`)
- Sends "ready" message on connection
- Pushes invalidation events when subscribed entities change
- Requires authentication
- Used for live query updates (Epic 8)

### 1.6 Health Check Endpoints

#### Endpoints:
- **GET `/healthz`** - Health check
- **GET `/livez`** - Liveness probe
- **GET `/readyz`** - Readiness probe

All health endpoints:
- Excluded from IP-based rate limiting
- Return "ok" status
- Used for Kubernetes health checks

### 1.7 Internationalization (`handlers/i18n.rs`)

- Internationalization support for multi-language content

---

## 2. Frontend Functionality

### 2.1 Authentication Feature (`features/authentication/`)

#### Pages:
- **LoginPage** (`/authentication/login`)
  - Email/password login form
  - Error message display
  - Redirects to dashboard on success
  - Redirects to profile selection if user has multiple profiles

- **RegisterPage** (`/authentication/register`)
  - User registration form
  - Email and password validation
  - Redirects to login after successful registration

- **SelectScopePage** (`/authentication/select-profile`)
  - Profile selection for multi-org users
  - Shows available organization/role profiles
  - Redirects to dashboard after selection

#### Components:
- **AuthenticationLayout** - Layout wrapper for auth pages
- **SelectScopeOnlyGuard** - Route guard that redirects if profile not selected

#### Services:
- **Authentication** - Authentication service interface
- **AuthenticationLive** - Live implementation (API calls)
- **AuthenticationMock** - Mock implementation for testing

#### Domain:
- **AuthenticationUser** - User domain model
- **Profile** - Profile/scope domain model
- **Scope** - Scope domain model
- **Flash** - Flash message domain model
- **AuthenticationStateSnapshot** - State snapshot for reactive store

#### Stores:
- **AuthenticationStateReactiveStore** - Reactive store for auth state

### 2.2 Dashboard Feature (`features/dashboard/`)

#### Pages:
- **DashboardPage** (`/dashboard`)
  - Main dashboard landing page
  - Requires `dashboard` permission
  - Shows overview and navigation

- **OrganizationsPage** (`/dashboard/organizations`)
  - List and manage organizations
  - Requires `organization.read` and `organization.create` permissions
  - Filtering and pagination support

- **UsersPage** (`/dashboard/users`)
  - List and manage users
  - Requires `user.read` and `user.create` permissions
  - Scope-aware (shows users based on current scope)
  - Filtering support

- **RolesPage** (`/dashboard/roles`)
  - List and manage roles
  - Requires `role.read` and `role.create` permissions

- **PermissionsPage** (`/dashboard/roles-and-permissions`)
  - Manage role-permission assignments
  - Requires `permission.read` and `permission.create` permissions
  - Add/remove permissions from roles

- **AuditLogPage** (`/dashboard/audit-log`)
  - View audit log entries
  - Requires `audit.read` permission
  - Filtering and pagination support

#### Components:
- **DashboardLayout** - Main dashboard layout with sidebar and navbar
- **Sidebar** - Navigation sidebar with permission-based menu items
- **DashboardScopeGuard** - Ensures scope is set before accessing dashboard
- **OrganizationsFilters** - Filter component for organizations
- **UsersFilters** - Filter component for users
- **AuditLogFilters** - Filter component for audit log
- **OrganizationCard** - Card component for organization display
- **OrganizationTableRow** - Table row for organization
- **UserTableRow** - Table row for user
- **RoleTableRow** - Table row for role
- **AssignmentTableRow** - Table row for role-permission assignment
- **AuditLogTableRow** - Table row for audit log entry
- **PermissionsAddBar** - Component for adding permissions to roles

#### Services:
- **Dashboard** - Dashboard service interface
- **DashboardLive** - Live implementation
- **DashboardMock** - Mock implementation
- **Users** - Users service interface
- **UsersLive** - Live implementation
- **UsersMock** - Mock implementation
- **Roles** - Roles service interface
- **RolesLive** - Live implementation
- **RolesMock** - Mock implementation
- **Permissions** - Permissions service interface
- **PermissionsLive** - Live implementation
- **PermissionsMock** - Mock implementation
- **AuditLog** - Audit log service interface
- **AuditLogLive** - Live implementation
- **AuditLogMock** - Mock implementation

#### Domain:
- **Organization** - Organization domain model
- **User** - User domain model
- **Role** - Role domain model
- **DashboardRole** - Dashboard role domain model
- **Assignment** - Role-permission assignment domain model
- **AuditLogEntry** - Audit log entry domain model

#### Utils:
- **formatDate** - Date formatting utility
- **membershipsSummary** - Membership summary utility

#### Hooks:
- **useOrganizationsFilter** - Hook for organization filtering

### 2.3 Profile Feature (`features/profile/`)

#### Pages:
- **ProfilePage** (`/scope`)
  - Profile/scope management page
  - Switch between available profiles
  - View current scope information

### 2.4 Home Feature (`features/home/`)

#### Pages:
- **HomePage** (`/`)
  - Home page
  - Protected route (requires authentication)
  - Basic landing page

### 2.5 Shared Components

#### Components:
- **ProtectedRoute** - Route guard for authenticated routes
- **GuestRoute** - Route guard for guest-only routes (login/register)
- **PermissionGuard** - Component that shows/hides content based on permissions
- **Navbar** - Top navigation bar with user menu and theme toggle
- **SubscriptionStreamRunner** - Component that manages subscription stream connection

#### Providers:
- **Providers** - Root provider component (ThemeProvider, etc.)

#### Hooks:
- **useColorScheme** - Hook for managing color scheme (light/dark mode)

### 2.6 Routing

#### Routes:
- `/authentication/login` - Login page (GuestRoute)
- `/authentication/register` - Register page (GuestRoute)
- `/authentication/select-scope` - Profile selection (ProtectedRoute + SelectScopeOnlyGuard)
- `/` - Home page (ProtectedRoute)
- `/dashboard` - Dashboard layout (ProtectedRoute + DashboardScopeGuard)
  - `/dashboard` - Dashboard page (PermissionGuard: `dashboard`)
  - `/dashboard/organizations` - Organizations page (PermissionGuard: `organization.read`, `organization.create`)
  - `/dashboard/users` - Users page (PermissionGuard: `user.read`, `user.create`)
  - `/dashboard/roles` - Roles page (PermissionGuard: `role.read`, `role.create`)
  - `/dashboard/roles-and-permissions` - Permissions page (PermissionGuard: `permission.read`, `permission.create`)
  - `/dashboard/audit-log` - Audit log page (PermissionGuard: `audit.read`)
- `/scope` - Profile page (ProtectedRoute)

### 2.7 Features

- **Theme Management**: Light/dark mode toggle (persisted in localStorage)
- **Permission-based UI**: Components show/hide based on user permissions
- **Scope Management**: Organization and role scope selection and switching
- **Live Updates**: Subscription stream for real-time data updates
- **Responsive Design**: Material UI components with responsive layouts

---

## 3. Integration Tests

All integration tests are located in `crates/forge-cli/templates/default/crates/app/tests/` and use the `test_client()` or `test_client_with_migrations()` helper functions.

### 3.1 Authentication Tests

#### `auth_register.rs`
**Tests:** POST `/api/auth/register`
- ✅ Should return 201 for valid registration
- ✅ Should return 422 for invalid email format
- ✅ Should return 422 for short password

#### `auth_login.rs`
**Tests:** POST `/api/auth/login` and GET `/api/auth/me`
- ✅ Should return 200 for profile after successful login
- ✅ Should return 401 for login with invalid credentials
- ✅ Should return 401 for profile without token

#### `auth_rest.rs`
**Tests:** Auth REST endpoints (logout, me, profiles, tokens, admin)
- ✅ Logout behavior:
  - Should allow logout for anonymous users (200 OK)
  - Should allow logout for authenticated users (200 OK)
- ✅ Me endpoint behavior:
  - Should require authentication (401 Unauthorized)
  - Should return user info for authenticated users (200 OK)
  - Should reject invalid or expired tokens (401 Unauthorized)
- ✅ Profiles endpoint behavior:
  - Should require authentication (401 Unauthorized)
  - Should return profiles for authenticated users (200 OK)
- ✅ Tokens endpoint behavior:
  - Should require authentication (401 Unauthorized)
  - Should create token for authenticated users (201 Created)
- ✅ Admin endpoint behavior:
  - Should require authentication (401 Unauthorized)
  - Should return 403 for non-admin users
  - Should return 200 for global admin users

#### `auth_token_query.rs`
**Tests:** Token-based authentication and query functionality
- Token authentication and query parameter handling

#### `auth_profile_session.rs`
**Tests:** Profile and session management
- Profile selection and session scope handling

### 3.2 Dashboard Tests

#### `dashboard_organizations.rs`
**Tests:** GET/POST `/api/dashboard/organizations`
- ✅ Should return 401 when anonymous request
- ✅ Should return 403 when viewer without dashboard.organizations.read
- ✅ Should return 200 with organizations list when global admin

#### `dashboard_permissions.rs`
**Tests:** Permission-based access control
- Permission checking and authorization

#### `dashboard_audit_log.rs`
**Tests:** GET `/api/entities/audit`
- ✅ Should return 401 for anonymous requests
- ✅ Should return 200 for viewer (has audit.read permission)
- ✅ Should return 200 for admin
- ✅ Should return 401 for get by ID without auth
- ✅ Should return 200 for get by ID with proper permissions

### 3.3 Entity API Tests

#### `api_users.rs`
**Tests:** GET/POST `/api/dashboard/users` and organization user endpoints
- ✅ Authentication behavior:
  - Should return 401 for GET /api/users when unauthenticated
  - Should return 403 for GET /api/users when viewer without scope
- ✅ Get API users behavior:
  - Should return 200 with users array when viewer has scope
  - Scope-aware user listing

#### `api_roles.rs`
**Tests:** GET/POST `/api/organizations/{id}/roles` and role management
- ✅ Should require authentication for getting org roles (401)
- ✅ Should allow viewer to list org roles (200)
- ✅ Should require authentication for creating org roles (401)
- Role CRUD operations with proper authorization

#### `api_role_permissions.rs`
**Tests:** Role-permission assignment endpoints
- GET/POST/DELETE `/api/organizations/{org_id}/roles/{role_id}/permissions`
- Role-permission assignment and removal

#### `api_generic_entity.rs`
**Tests:** Generic entity REST handler GET/POST `/api/entities/{entity_id}` and GET/PATCH/DELETE `/api/entities/{entity_id}/{id}`
- ✅ List: 401 for all entities when unauthenticated; 200 with scope for organization, user, role, permission, audit
- ✅ List: filter (eq, ne, in), sort, **offset/limit** pagination, **cursor** pagination (first page, second page via next_cursor, invalid cursor 400, cursor+offset mutually exclusive 400)
- ✅ List errors: unknown entity 404, invalid filter JSON/operator/field 400, invalid sort field 400, expand/include 400
- ✅ Get by ID: 401, 400 invalid UUID, 404, 200
- ✅ Create/update/delete: organization only (201/200/200); CUD not supported for user/role/permission/audit (4xx/5xx)

### 3.4 Health Check Tests

#### `healthz.rs`
**Tests:** Health check endpoints
- ✅ GET `/healthz` returns 200 OK
- ✅ GET `/livez` returns 200 OK
- ✅ GET `/readyz` returns 200 OK

---

## 4. End-to-End Tests

All E2E tests are located in `test/e2e/tests/` and use Playwright for browser automation. Tests run against a prebuilt project spawned by `bin/test-e2e`.

### 4.1 CLI Tests (`001_cli.test.ts`)
**Tests:** CLI functionality
- Forge CLI project generation and setup

### 4.2 Config Tests (`002_config.test.ts`)
**Tests:** Configuration file structure
- Project configuration validation

### 4.3 Database Tests (`003_database.test.ts`)
**Tests:** Database setup and configuration
- Database connection and configuration

### 4.4 Migrations Tests (`004_migrations.test.ts`)
**Tests:** Database migrations
- Migration execution and seed data

### 4.5 Authentication Tests (`005_auth.test.ts`)
**Tests:** Authentication flow
- ✅ Auth layout: project has auth, org, membership, user AuthzContext, admin route
- ✅ Unauthed `/dashboard` redirects to login
- ✅ Login fails with wrong password and shows error
- ✅ Login with seed user then dashboard visible
- ✅ Logout then `/dashboard` redirects to login
- ✅ Global admin can access `/api/auth/admin`
- ✅ Non-admin cannot access `/api/auth/admin` (Forbidden)
- ✅ Register then login then dashboard (full flow)

### 4.6 Authorization Tests (`006_authz.test.ts`)
**Tests:** Authorization and protected routes
- ✅ Authz layout and protected route: unauthed redirect, register+login then dashboard
- Protected route behavior
- Authorization context setup

### 4.7 Permission Dashboard Tests (`007_permission_dashboard.test.ts`)
**Tests:** Permission-based dashboard access
- ✅ Admin sees all nav items and can open Organizations, Users, Permissions
- ✅ Org admin has no Organizations nav and is redirected from `/dashboard/organizations`
- ✅ Viewer has Dashboard, Users, and Permissions (read) nav; no Organizations
- ✅ Admin can load Permissions page and see assignments
- ✅ Admin sees users from all orgs on Users page (global-scope)

### 4.8 Health Tests (`008_health.test.ts`)
**Tests:** Health check endpoints
- ✅ Prebuilt layout and healthz, livez, readyz return ok
- Health endpoint responses

### 4.9 Rate Limiting Tests (`009_rate_limiting.test.ts`)
**Tests:** Rate limiting functionality
- ✅ App root loads 5 times
- Basic rate limiting behavior

### 4.10 Profile Select Tests (`010_profile_select.test.ts`)
**Tests:** Profile selection and scope management
- ✅ Multi-org user is redirected to profile-select after login
- ✅ Single-profile user goes straight to dashboard after login
- ✅ Profile-select: choose profile then land on dashboard
- ✅ Dashboard without profile redirects to profile-select
- ✅ Viewer sees only Default org users on Users page
- ✅ Multi@email.com: three profiles (Personal, Default, Other) and nav per profile
- ✅ Switch profile in navbar updates dashboard scope (users list)
- ✅ Register then login lands in dashboard with profile set

### 4.11 Security Tests (`013_security.test.ts`)
**Tests:** Security headers and browser security
- ✅ OWASP-aligned headers on `/healthz`
  - `X-Content-Type-Options: nosniff`
  - `X-Frame-Options: DENY`
  - `Referrer-Policy: strict-origin-when-cross-origin`
  - `Content-Security-Policy` with `frame-ancestors`
  - `Permissions-Policy: geolocation=()`
  - `Cross-Origin-Resource-Policy: same-site`
- ✅ Browser loads app root

---

## Summary

### Backend Functionality Summary:
- **7 handler modules** covering authentication, entities, dashboard, RPC, subscriptions, i18n, and health checks
- **24+ API endpoints** supporting CRUD operations, authentication, authorization, and real-time features
- **Entity-based permissions** with scope-aware queries
- **RPC parity** with REST endpoints
- **Live subscriptions** via Server-Sent Events
- **Comprehensive authorization** with global and org-scoped permissions

### Frontend Functionality Summary:
- **4 main features**: Authentication, Dashboard, Profile, Home
- **7 dashboard pages**: Dashboard, Organizations, Users, Roles, Permissions, Audit Log
- **Permission-based UI** with dynamic navigation and content visibility
- **Scope management** with profile selection and switching
- **Live updates** via subscription stream
- **Theme management** with light/dark mode

### Integration Tests Summary:
- **12 test files** covering authentication, dashboard, entities, and health checks
- **50+ test cases** verifying API endpoints, authorization, and business logic
- Tests use BDD-style organization with descriptive names
- Tests support both in-process router and external server (via E2E_API_URL)

### End-to-End Tests Summary:
- **11 test files** covering CLI, config, database, migrations, auth, authorization, permissions, health, rate limiting, profile selection, and security
- **28+ test cases** verifying full user flows from browser perspective
- Tests use Playwright for browser automation
- Tests verify UI behavior, navigation, permissions, and security headers

---

*Last updated: Generated from codebase analysis*
