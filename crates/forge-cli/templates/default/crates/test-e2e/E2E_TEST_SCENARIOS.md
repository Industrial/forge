# End-to-End Test Scenarios Tree

This document defines a comprehensive Directed Acyclic Graph (DAG) of all critical E2E test scenarios organized by user role. Each scenario represents a critical path through the application, avoiding loops while covering all possible user interactions.

## Data-TestID Conventions

Each scenario includes:
- **Interactions**: User actions with specific `data-testid` attributes for elements being clicked, typed into, or otherwise interacted with
- **Verifications**: Expected elements with `data-testid` attributes that should be visible, contain specific content, or have certain states

**Note**: These `data-testid` attributes are planned for test implementation and may not exist in the frontend code yet. They should be added during frontend development to support E2E testing.

### Common TestID Patterns:
- Pages: `[data-testid="{page-name}-page"]` (e.g., `login-page`, `dashboard-page`)
- Forms: `[data-testid="{entity}-{action}-form"]` (e.g., `user-create-form`, `role-edit-form`)
- Buttons: `[data-testid="{entity}-{action}-button"]` (e.g., `user-create-button`, `role-edit-button`)
- Lists/Tables: `[data-testid="{entity}-list"]` or `[data-testid="{entity}-table"]`
- Rows/Cards: `[data-testid="{entity}-row"]` or `[data-testid="{entity}-card"]`
- Inputs: `[data-testid="{entity}-{field}-input"]` (e.g., `user-email-input`, `role-name-input`)
- Dialogs: `[data-testid="{entity}-{action}-dialog"]` (e.g., `user-create-dialog`)
- Error messages: `[data-testid="{entity}-error-message"]` or `[data-testid="error-{code}"]`
- Navigation: `[data-testid="sidebar-{page}-link"]` (e.g., `sidebar-users-link`)

---

## 1. Guest/Unauthenticated User

### 1.1 Initial Access
```
[Guest] → Visit `/` → Redirect to `/authentication/login`
  Verification: URL is `/authentication/login`
  Verification: `[data-testid="login-page"]` is visible
  Verification: `[data-testid="login-form"]` is visible

[Guest] → Visit `/dashboard` → Redirect to `/authentication/login`
  Verification: URL is `/authentication/login`
  Verification: `[data-testid="login-page"]` is visible

[Guest] → Visit `/authentication/login` → Shows login form
  Verification: `[data-testid="login-page"]` is visible
  Verification: `[data-testid="login-form"]` is visible
  Verification: `[data-testid="login-email-input"]` is visible
  Verification: `[data-testid="login-password-input"]` is visible
  Verification: `[data-testid="login-submit-button"]` is visible
  Verification: `[data-testid="login-register-link"]` is visible

[Guest] → Visit `/authentication/register` → Shows registration form
  Verification: `[data-testid="register-page"]` is visible
  Verification: `[data-testid="register-form"]` is visible
  Verification: `[data-testid="register-email-input"]` is visible
  Verification: `[data-testid="register-password-input"]` is visible
  Verification: `[data-testid="register-submit-button"]` is visible
  Verification: `[data-testid="register-login-link"]` is visible

[Guest] → `/authentication/login` → Click "Register" link → Navigate to `/authentication/register`
  Interaction: Click `[data-testid="login-register-link"]`
  Verification: URL is `/authentication/register`
  Verification: `[data-testid="register-page"]` is visible

[Guest] → `/authentication/register` → Click "Login" link → Navigate to `/authentication/login`
  Interaction: Click `[data-testid="register-login-link"]`
  Verification: URL is `/authentication/login`
  Verification: `[data-testid="login-page"]` is visible
```

### 1.2 Registration Flow
```
[Guest] → `/authentication/register`
  Verification: `[data-testid="register-page"]` is visible
  Verification: `[data-testid="register-form"]` is visible

  ├─→ Submit invalid email → Error message
    Interaction: Type invalid email in `[data-testid="register-email-input"]`
    Interaction: Type password in `[data-testid="register-password-input"]`
    Interaction: Click `[data-testid="register-submit-button"]`
    Verification: `[data-testid="register-error-message"]` is visible
    Verification: `[data-testid="register-error-message"]` contains error text
    Verification: URL is still `/authentication/register`

  ├─→ Submit short password → Error message
    Interaction: Type valid email in `[data-testid="register-email-input"]`
    Interaction: Type short password in `[data-testid="register-password-input"]`
    Interaction: Click `[data-testid="register-submit-button"]`
    Verification: `[data-testid="register-error-message"]` is visible
    Verification: `[data-testid="register-error-message"]` contains error text
    Verification: URL is still `/authentication/register`

  ├─→ Submit valid registration → Redirect to `/authentication/login`
    Interaction: Type valid email in `[data-testid="register-email-input"]`
    Interaction: Type valid password in `[data-testid="register-password-input"]`
    Interaction: Click `[data-testid="register-submit-button"]`
    Verification: URL is `/authentication/login`
    Verification: `[data-testid="login-page"]` is visible
    Verification: `[data-testid="register-error-message"]` is not visible

  ├─→ Click "Login" link → Navigate to `/authentication/login`
    Interaction: Click `[data-testid="register-login-link"]`
    Verification: URL is `/authentication/login`
    Verification: `[data-testid="login-page"]` is visible

  └─→ Login with new credentials → Success → [Authenticated Flow]
    Interaction: Type email in `[data-testid="login-email-input"]`
    Interaction: Type password in `[data-testid="login-password-input"]`
    Interaction: Click `[data-testid="login-submit-button"]`
    Verification: URL is `/dashboard` or `/authentication/select-scope`
    Verification: `[data-testid="login-error-message"]` is not visible
```

### 1.3 Login Flow
```
[Guest] → `/authentication/login`
  Verification: `[data-testid="login-page"]` is visible
  Verification: `[data-testid="login-form"]` is visible

  ├─→ Submit invalid credentials → Error message
    Interaction: Type invalid email in `[data-testid="login-email-input"]`
    Interaction: Type password in `[data-testid="login-password-input"]`
    Interaction: Click `[data-testid="login-submit-button"]`
    Verification: `[data-testid="login-error-message"]` is visible
    Verification: `[data-testid="login-error-message"]` contains error text
    Verification: URL is still `/authentication/login`

  ├─→ Submit valid credentials (single profile) → Redirect to `/dashboard`
    Interaction: Type valid email in `[data-testid="login-email-input"]`
    Interaction: Type valid password in `[data-testid="login-password-input"]`
    Interaction: Click `[data-testid="login-submit-button"]`
    Verification: URL is `/dashboard`
    Verification: `[data-testid="dashboard-page"]` is visible
    Verification: `[data-testid="login-error-message"]` is not visible

  ├─→ Submit valid credentials (multi-profile) → Redirect to `/authentication/select-scope`
    Interaction: Type multi-profile email in `[data-testid="login-email-input"]`
    Interaction: Type password in `[data-testid="login-password-input"]`
    Interaction: Click `[data-testid="login-submit-button"]`
    Verification: URL is `/authentication/select-scope`
    Verification: `[data-testid="select-scope-page"]` is visible
    Verification: `[data-testid="profile-list"]` is visible

  └─→ Click "Register" link → Navigate to `/authentication/register` → Can navigate back to login
    Interaction: Click `[data-testid="login-register-link"]`
    Verification: URL is `/authentication/register`
    Verification: `[data-testid="register-page"]` is visible
    Interaction: Click `[data-testid="register-login-link"]`
    Verification: URL is `/authentication/login`
    Verification: `[data-testid="login-page"]` is visible
```

### 1.4 Public Endpoints
```
[Guest] → GET `/healthz` → 200 OK
[Guest] → GET `/livez` → 200 OK
[Guest] → GET `/readyz` → 200 OK
[Guest] → GET `/api/auth/logout` → 200 OK (works for anonymous)
```

### 1.5 Protected Endpoint Access (Unauthenticated)
```
[Guest] → GET `/api/auth/me` → 401 Unauthorized
[Guest] → GET `/api/auth/scopes` → 401 Unauthorized
[Guest] → POST `/api/auth/tokens` → 401 Unauthorized
[Guest] → GET `/api/auth/admin` → 401 Unauthorized
[Guest] → GET `/api/entities/organization` → 401 Unauthorized
[Guest] → GET `/api/dashboard/users` → 401 Unauthorized
```

---

## 2. Viewer Role

**Permissions:** `dashboard`, `user.read`, `audit.read`, `permission.read`, `role.read`

### 2.1 Authentication & Profile Selection
```
[Viewer] → Login → Single profile → `/dashboard`
  Interaction: Type email in `[data-testid="login-email-input"]`
  Interaction: Type password in `[data-testid="login-password-input"]`
  Interaction: Click `[data-testid="login-submit-button"]`
  Verification: URL is `/dashboard`
  Verification: `[data-testid="dashboard-page"]` is visible

[Viewer] → Login → Multiple profiles → `/authentication/select-scope` → Select profile → `/dashboard`
  Interaction: Type multi-profile email in `[data-testid="login-email-input"]`
  Interaction: Type password in `[data-testid="login-password-input"]`
  Interaction: Click `[data-testid="login-submit-button"]`
  Verification: URL is `/authentication/select-scope`
  Verification: `[data-testid="select-scope-page"]` is visible
  Verification: `[data-testid="profile-list"]` is visible
  Interaction: Click first `[data-testid="profile-card"]` or `[data-testid="profile-select-button"]`
  Verification: URL is `/dashboard`
  Verification: `[data-testid="dashboard-page"]` is visible
```

### 2.2 Dashboard Access
```
[Viewer] → `/dashboard`
  Verification: `[data-testid="dashboard-page"]` is visible
  Verification: `[data-testid="dashboard-layout"]` is visible

  ├─→ Dashboard page loads → Shows navigation
    Verification: `[data-testid="dashboard-sidebar"]` is visible
    Verification: `[data-testid="dashboard-navbar"]` is visible
    Verification: `[data-testid="dashboard-content"]` is visible

  ├─→ Sidebar shows: Dashboard, Users, Roles, Permissions, Audit Log
    Verification: `[data-testid="sidebar-dashboard-link"]` is visible
    Verification: `[data-testid="sidebar-users-link"]` is visible
    Verification: `[data-testid="sidebar-roles-link"]` is visible
    Verification: `[data-testid="sidebar-permissions-link"]` is visible
    Verification: `[data-testid="sidebar-audit-log-link"]` is visible

  ├─→ Sidebar does NOT show: Organizations
    Verification: `[data-testid="sidebar-organizations-link"]` is not visible

  └─→ Theme toggle works
    Verification: `[data-testid="theme-toggle-button"]` is visible
    Interaction: Click `[data-testid="theme-toggle-button"]`
    Verification: Theme changes (check body class or data attribute)
    Interaction: Click `[data-testid="theme-toggle-button"]` again
    Verification: Theme changes back
```

### 2.3 Users Page
```
[Viewer] → `/dashboard/users`
  Verification: `[data-testid="users-page"]` is visible
  Verification: `[data-testid="users-list"]` or `[data-testid="users-table"]` is visible

  ├─→ Page loads → Shows users list (scope-aware)
    Verification: `[data-testid="users-page-title"]` is visible
    Verification: `[data-testid="users-list"]` or `[data-testid="users-table"]` is visible
    Verification: At least one `[data-testid="user-row"]` or `[data-testid="user-card"]` is visible

  ├─→ Filter users → Results update
    Interaction: Type in `[data-testid="users-filter-input"]`
    Verification: `[data-testid="users-list"]` updates with filtered results
    Verification: Filtered `[data-testid="user-row"]` elements match filter criteria

  ├─→ View user details → Read-only
    Interaction: Click first `[data-testid="user-row"]` or `[data-testid="user-view-button"]`
    Verification: `[data-testid="user-details-dialog"]` or `[data-testid="user-details-page"]` is visible
    Verification: `[data-testid="user-edit-button"]` is not visible
    Verification: `[data-testid="user-delete-button"]` is not visible

  ├─→ Attempt create user → 403 Forbidden (no user.create)
    Verification: `[data-testid="users-create-button"]` is not visible
    (If button exists, clicking it should show 403 error)
    Interaction: Navigate directly to create endpoint → Verify 403 response

  └─→ Attempt edit user → 403 Forbidden (no user.update)
    Verification: `[data-testid="user-edit-button"]` is not visible in user details
    (If button exists, clicking it should show 403 error)
    Interaction: Navigate directly to edit endpoint → Verify 403 response
```

### 2.4 Roles Page
```
[Viewer] → `/dashboard/roles`
  Verification: `[data-testid="roles-page"]` is visible
  Verification: `[data-testid="roles-list"]` or `[data-testid="roles-table"]` is visible

  ├─→ Page loads → Shows roles list
    Verification: `[data-testid="roles-page-title"]` is visible
    Verification: `[data-testid="roles-list"]` or `[data-testid="roles-table"]` is visible
    Verification: At least one `[data-testid="role-row"]` or `[data-testid="role-card"]` is visible

  ├─→ View role details → Read-only
    Interaction: Click first `[data-testid="role-row"]` or `[data-testid="role-view-button"]`
    Verification: `[data-testid="role-details-dialog"]` or `[data-testid="role-details-page"]` is visible
    Verification: `[data-testid="role-edit-button"]` is not visible
    Verification: `[data-testid="role-delete-button"]` is not visible

  ├─→ Attempt create role → 403 Forbidden (no role.create)
    Verification: `[data-testid="roles-create-button"]` is not visible
    (If button exists, clicking it should show 403 error)
    Interaction: Navigate directly to create endpoint → Verify 403 response

  └─→ Attempt edit role → 403 Forbidden (no role.update)
    Verification: `[data-testid="role-edit-button"]` is not visible in role details
    (If button exists, clicking it should show 403 error)
    Interaction: Navigate directly to edit endpoint → Verify 403 response
```

### 2.5 Permissions Page
```
[Viewer] → `/dashboard/roles-and-permissions`
  Verification: `[data-testid="permissions-page"]` is visible
  Verification: `[data-testid="permissions-list"]` or `[data-testid="permissions-table"]` is visible

  ├─→ Page loads → Shows role-permission assignments
    Verification: `[data-testid="permissions-page-title"]` is visible
    Verification: `[data-testid="permissions-list"]` or `[data-testid="permissions-table"]` is visible
    Verification: At least one `[data-testid="permission-assignment-row"]` is visible

  ├─→ View assignments → Read-only
    Verification: `[data-testid="permission-add-button"]` is not visible
    Verification: `[data-testid="permission-remove-button"]` is not visible in assignment rows

  ├─→ Attempt add permission → 403 Forbidden (no permission.create)
    Verification: `[data-testid="permission-add-button"]` is not visible
    (If button exists, clicking it should show 403 error)
    Interaction: Navigate directly to add permission endpoint → Verify 403 response

  └─→ Attempt remove permission → 403 Forbidden (no permission.delete)
    Verification: `[data-testid="permission-remove-button"]` is not visible in assignment rows
    (If button exists, clicking it should show 403 error)
    Interaction: Navigate directly to remove permission endpoint → Verify 403 response
```

### 2.6 Audit Log Page
```
[Viewer] → `/dashboard/audit-log`
  Verification: `[data-testid="audit-log-page"]` is visible
  Verification: `[data-testid="audit-log-list"]` or `[data-testid="audit-log-table"]` is visible

  ├─→ Page loads → Shows audit entries
    Verification: `[data-testid="audit-log-page-title"]` is visible
    Verification: `[data-testid="audit-log-list"]` or `[data-testid="audit-log-table"]` is visible
    Verification: At least one `[data-testid="audit-log-row"]` is visible

  ├─→ Filter audit log → Results update
    Interaction: Type in `[data-testid="audit-log-filter-input"]`
    Verification: `[data-testid="audit-log-list"]` updates with filtered results
    Verification: Filtered `[data-testid="audit-log-row"]` elements match filter criteria

  ├─→ Paginate audit log → Next page loads
    Verification: `[data-testid="audit-log-pagination"]` is visible
    Interaction: Click `[data-testid="audit-log-pagination-next"]` or `[data-testid="audit-log-pagination-page-2"]`
    Verification: URL updates with pagination params
    Verification: `[data-testid="audit-log-list"]` shows different entries
    Verification: `[data-testid="audit-log-pagination-prev"]` is visible (if not on first page)

  └─→ View audit entry details → Read-only
    Interaction: Click first `[data-testid="audit-log-row"]` or `[data-testid="audit-log-view-button"]`
    Verification: `[data-testid="audit-log-details-dialog"]` or `[data-testid="audit-log-details-page"]` is visible
    Verification: Details are read-only (no edit buttons)
```

### 2.7 Organizations Page (Blocked)
```
[Viewer] → `/dashboard/organizations` → 403 Forbidden (no organization.read)
  Interaction: Navigate to `/dashboard/organizations`
  Verification: 403 error is shown or redirect occurs
  Verification: `[data-testid="error-403"]` or error message is visible

[Viewer] → Sidebar → Organizations link → Not visible
  Verification: `[data-testid="sidebar-organizations-link"]` is not visible
  Verification: `[data-testid="dashboard-sidebar"]` does not contain organizations link
```

### 2.8 Profile Management
```
[Viewer] → `/scope`
  Verification: `[data-testid="profile-page"]` is visible

  ├─→ Page loads → Shows current profile
    Verification: `[data-testid="profile-page-title"]` is visible
    Verification: `[data-testid="current-profile"]` is visible
    Verification: `[data-testid="current-profile-org-name"]` is visible
    Verification: `[data-testid="current-profile-role-name"]` is visible

  ├─→ Switch profile (if multiple) → Scope updates → Dashboard refreshes
    Verification: `[data-testid="profile-list"]` is visible (if multiple profiles)
    Interaction: Click different `[data-testid="profile-card"]` or `[data-testid="profile-switch-button"]`
    Verification: `[data-testid="current-profile"]` updates
    Verification: Navigate to `/dashboard` → `[data-testid="dashboard-page"]` shows updated scope
    Verification: `[data-testid="users-list"]` shows users from new scope

  └─→ View profile details → Read-only
    Verification: `[data-testid="profile-details"]` is visible
    Verification: Profile information is displayed (read-only)
```

### 2.9 API Token Management
```
[Viewer] → POST `/api/auth/tokens` → 201 Created → Token returned
[Viewer] → Use token → GET `/api/auth/me` → 200 OK
```

### 2.10 Generic Entity API (Read-only)
```
[Viewer] → GET `/api/entities/user` → 200 OK (has user.read)
[Viewer] → GET `/api/entities/user/{id}` → 200 OK
[Viewer] → GET `/api/entities/role` → 200 OK (has role.read)
[Viewer] → GET `/api/entities/permission` → 200 OK (has permission.read)
[Viewer] → GET `/api/entities/audit` → 200 OK (has audit.read)
[Viewer] → GET `/api/entities/organization` → 403 Forbidden (no organization.read)
[Viewer] → POST `/api/entities/organization` → 403 Forbidden (no organization.create)
[Viewer] → PATCH `/api/entities/organization/{id}` → 403 Forbidden (no organization.update)
[Viewer] → DELETE `/api/entities/organization/{id}` → 403 Forbidden (no organization.delete)
```

### 2.11 RPC API (Read-only)
```
[Viewer] → POST `/api/rpc` → `entity.list` → 200 OK (for entities with read permission)
[Viewer] → POST `/api/rpc` → `entity.get` → 200 OK
[Viewer] → POST `/api/rpc` → `entity.create` → 403 Forbidden
[Viewer] → POST `/api/rpc` → `entity.update` → 403 Forbidden
[Viewer] → POST `/api/rpc` → `entity.delete` → 403 Forbidden
```

### 2.12 Subscription Stream
```
[Viewer] → GET `/api/subscriptions/stream` → SSE connection established
  ├─→ Receives "ready" message
  ├─→ Subscribe to entities → Receives invalidation events
  └─→ Unsubscribe → Stops receiving events
```

### 2.13 Admin Endpoint (Blocked)
```
[Viewer] → GET `/api/auth/admin` → 403 Forbidden (not app admin)
```

### 2.14 Logout
```
[Viewer] → Logout → Redirect to `/authentication/login`
  Interaction: Click `[data-testid="navbar-user-menu"]` or `[data-testid="navbar-menu-button"]`
  Verification: `[data-testid="user-menu-dropdown"]` is visible
  Interaction: Click `[data-testid="user-menu-logout-button"]`
  Verification: URL is `/authentication/login`
  Verification: `[data-testid="login-page"]` is visible

[Viewer] → After logout → Protected routes redirect to login
  Interaction: Navigate to `/dashboard`
  Verification: URL is `/authentication/login`
  Verification: `[data-testid="login-page"]` is visible
```

---

## 3. Editor Role

**Permissions:** `dashboard`, `user.read`, `user.create`, `user.update`, `permission.read`, `permission.create`, `permission.delete`, `role.read`, `role.create`, `role.update`

### 3.1 Authentication & Profile Selection
```
[Editor] → Login → Single profile → `/dashboard`
  Interaction: Type email in `[data-testid="login-email-input"]`
  Interaction: Type password in `[data-testid="login-password-input"]`
  Interaction: Click `[data-testid="login-submit-button"]`
  Verification: URL is `/dashboard`
  Verification: `[data-testid="dashboard-page"]` is visible

[Editor] → Login → Multiple profiles → `/authentication/select-scope` → Select profile → `/dashboard`
  Interaction: Type multi-profile email in `[data-testid="login-email-input"]`
  Interaction: Type password in `[data-testid="login-password-input"]`
  Interaction: Click `[data-testid="login-submit-button"]`
  Verification: URL is `/authentication/select-scope`
  Verification: `[data-testid="select-scope-page"]` is visible
  Interaction: Click first `[data-testid="profile-card"]` or `[data-testid="profile-select-button"]`
  Verification: URL is `/dashboard`
  Verification: `[data-testid="dashboard-page"]` is visible
```

### 3.2 Dashboard Access
```
[Editor] → `/dashboard`
  Verification: `[data-testid="dashboard-page"]` is visible

  ├─→ Dashboard page loads → Shows navigation
    Verification: `[data-testid="dashboard-sidebar"]` is visible
    Verification: `[data-testid="dashboard-navbar"]` is visible

  ├─→ Sidebar shows: Dashboard, Users, Roles, Permissions
    Verification: `[data-testid="sidebar-dashboard-link"]` is visible
    Verification: `[data-testid="sidebar-users-link"]` is visible
    Verification: `[data-testid="sidebar-roles-link"]` is visible
    Verification: `[data-testid="sidebar-permissions-link"]` is visible

  ├─→ Sidebar does NOT show: Organizations, Audit Log
    Verification: `[data-testid="sidebar-organizations-link"]` is not visible
    Verification: `[data-testid="sidebar-audit-log-link"]` is not visible

  └─→ Theme toggle works
    Interaction: Click `[data-testid="theme-toggle-button"]`
    Verification: Theme changes
```

### 3.3 Users Page (Create/Update)
```
[Editor] → `/dashboard/users`
  Verification: `[data-testid="users-page"]` is visible
  Verification: `[data-testid="users-create-button"]` is visible

  ├─→ Page loads → Shows users list
    Verification: `[data-testid="users-list"]` or `[data-testid="users-table"]` is visible

  ├─→ Create user → Form → Submit → User created → List updates
    Interaction: Click `[data-testid="users-create-button"]`
    Verification: `[data-testid="user-create-dialog"]` or `[data-testid="user-create-form"]` is visible
    Interaction: Type email in `[data-testid="user-create-email-input"]`
    Interaction: Type password in `[data-testid="user-create-password-input"]`
    Interaction: Click `[data-testid="user-create-submit-button"]`
    Verification: `[data-testid="user-create-dialog"]` closes or form submits
    Verification: `[data-testid="users-list"]` updates with new user
    Verification: New `[data-testid="user-row"]` is visible

  ├─→ Update user → Form → Submit → User updated → List updates
    Interaction: Click first `[data-testid="user-row"]` or `[data-testid="user-edit-button"]`
    Verification: `[data-testid="user-edit-dialog"]` or `[data-testid="user-edit-form"]` is visible
    Interaction: Update email in `[data-testid="user-edit-email-input"]`
    Interaction: Click `[data-testid="user-edit-submit-button"]`
    Verification: `[data-testid="user-edit-dialog"]` closes or form submits
    Verification: `[data-testid="users-list"]` updates with changes
    Verification: Updated `[data-testid="user-row"]` shows new data

  ├─→ Filter users → Results update
    Interaction: Type in `[data-testid="users-filter-input"]`
    Verification: `[data-testid="users-list"]` updates with filtered results

  └─→ View user details → Can edit
    Interaction: Click first `[data-testid="user-row"]`
    Verification: `[data-testid="user-details-dialog"]` is visible
    Verification: `[data-testid="user-edit-button"]` is visible
    Verification: `[data-testid="user-delete-button"]` is not visible (no delete permission)
```

### 3.4 Roles Page (Create/Update)
```
[Editor] → `/dashboard/roles`
  Verification: `[data-testid="roles-page"]` is visible
  Verification: `[data-testid="roles-create-button"]` is visible

  ├─→ Page loads → Shows roles list
    Verification: `[data-testid="roles-list"]` or `[data-testid="roles-table"]` is visible

  ├─→ Create role → Form → Submit → Role created → List updates
    Interaction: Click `[data-testid="roles-create-button"]`
    Verification: `[data-testid="role-create-dialog"]` or `[data-testid="role-create-form"]` is visible
    Interaction: Type name in `[data-testid="role-create-name-input"]`
    Interaction: Click `[data-testid="role-create-submit-button"]`
    Verification: `[data-testid="role-create-dialog"]` closes
    Verification: `[data-testid="roles-list"]` updates with new role
    Verification: New `[data-testid="role-row"]` is visible

  ├─→ Update role → Form → Submit → Role updated → List updates
    Interaction: Click first `[data-testid="role-row"]` or `[data-testid="role-edit-button"]`
    Verification: `[data-testid="role-edit-dialog"]` or `[data-testid="role-edit-form"]` is visible
    Interaction: Update name in `[data-testid="role-edit-name-input"]`
    Interaction: Click `[data-testid="role-edit-submit-button"]`
    Verification: `[data-testid="role-edit-dialog"]` closes
    Verification: `[data-testid="roles-list"]` updates with changes
    Verification: Updated `[data-testid="role-row"]` shows new data

  └─→ View role details → Can edit
    Interaction: Click first `[data-testid="role-row"]`
    Verification: `[data-testid="role-details-dialog"]` is visible
    Verification: `[data-testid="role-edit-button"]` is visible
    Verification: `[data-testid="role-delete-button"]` is not visible (no delete permission)
```

### 3.5 Permissions Page (Full CRUD)
```
[Editor] → `/dashboard/roles-and-permissions`
  Verification: `[data-testid="permissions-page"]` is visible
  Verification: `[data-testid="permission-add-button"]` is visible

  ├─→ Page loads → Shows role-permission assignments
    Verification: `[data-testid="permissions-list"]` or `[data-testid="permissions-table"]` is visible

  ├─→ Add permission to role → Form → Submit → Assignment created → List updates
    Interaction: Click `[data-testid="permission-add-button"]`
    Verification: `[data-testid="permission-add-dialog"]` or `[data-testid="permission-add-form"]` is visible
    Interaction: Select role from `[data-testid="permission-add-role-select"]`
    Interaction: Select permission from `[data-testid="permission-add-permission-select"]`
    Interaction: Click `[data-testid="permission-add-submit-button"]`
    Verification: `[data-testid="permission-add-dialog"]` closes
    Verification: `[data-testid="permissions-list"]` updates with new assignment
    Verification: New `[data-testid="permission-assignment-row"]` is visible

  ├─→ Remove permission from role → Delete → Assignment removed → List updates
    Interaction: Click `[data-testid="permission-remove-button"]` on an assignment row
    Verification: `[data-testid="permission-remove-confirm-dialog"]` is visible (if confirmation needed)
    Interaction: Confirm deletion in `[data-testid="permission-remove-confirm-button"]`
    Verification: `[data-testid="permissions-list"]` updates (assignment removed)
    Verification: Removed `[data-testid="permission-assignment-row"]` is not visible

  └─→ View assignments → Can modify
    Verification: `[data-testid="permission-add-button"]` is visible
    Verification: `[data-testid="permission-remove-button"]` is visible in assignment rows
```

### 3.6 Audit Log Page (Blocked)
```
[Editor] → `/dashboard/audit-log` → 403 Forbidden (no audit.read)
  Interaction: Navigate to `/dashboard/audit-log`
  Verification: 403 error is shown or redirect occurs
  Verification: `[data-testid="error-403"]` or error message is visible

[Editor] → Sidebar → Audit Log link → Not visible
  Verification: `[data-testid="sidebar-audit-log-link"]` is not visible
```

### 3.7 Organizations Page (Blocked)
```
[Editor] → `/dashboard/organizations` → 403 Forbidden (no organization.read)
  Interaction: Navigate to `/dashboard/organizations`
  Verification: 403 error is shown or redirect occurs
  Verification: `[data-testid="error-403"]` or error message is visible

[Editor] → Sidebar → Organizations link → Not visible
  Verification: `[data-testid="sidebar-organizations-link"]` is not visible
```

### 3.8 Profile Management
```
[Editor] → `/scope`
  Verification: `[data-testid="profile-page"]` is visible

  ├─→ Page loads → Shows current profile
    Verification: `[data-testid="current-profile"]` is visible
    Verification: `[data-testid="current-profile-org-name"]` is visible
    Verification: `[data-testid="current-profile-role-name"]` is visible

  ├─→ Switch profile (if multiple) → Scope updates → Dashboard refreshes
    Verification: `[data-testid="profile-list"]` is visible (if multiple profiles)
    Interaction: Click different `[data-testid="profile-card"]` or `[data-testid="profile-switch-button"]`
    Verification: `[data-testid="current-profile"]` updates
    Verification: Navigate to `/dashboard` → `[data-testid="dashboard-page"]` shows updated scope

  └─→ View profile details → Read-only
    Verification: `[data-testid="profile-details"]` is visible
    Verification: Profile information is displayed (read-only)
```

### 3.9 Generic Entity API (Users, Roles, Permissions)
```
[Editor] → GET `/api/entities/user` → 200 OK
[Editor] → POST `/api/entities/user` → 201 Created (has user.create)
[Editor] → PATCH `/api/entities/user/{id}` → 200 OK (has user.update)
[Editor] → GET `/api/entities/role` → 200 OK
[Editor] → POST `/api/entities/role` → 201 Created (has role.create)
[Editor] → PATCH `/api/entities/role/{id}` → 200 OK (has role.update)
[Editor] → GET `/api/entities/organization` → 403 Forbidden (no organization.read)
[Editor] → POST `/api/entities/organization` → 403 Forbidden (no organization.create)
```

### 3.10 RPC API (Create/Update)
```
[Editor] → POST `/api/rpc` → `entity.create` → 200 OK (for users, roles)
[Editor] → POST `/api/rpc` → `entity.update` → 200 OK
[Editor] → POST `/api/rpc` → `entity.delete` → 403 Forbidden (no delete permissions)
```

### 3.11 Subscription Stream
```
[Editor] → GET `/api/subscriptions/stream` → SSE connection established
  ├─→ Receives "ready" message
  ├─→ Subscribe to user changes → Receives invalidation events on create/update
  └─→ Subscribe to role changes → Receives invalidation events
```

### 3.12 Admin Endpoint (Blocked)
```
[Editor] → GET `/api/auth/admin` → 403 Forbidden (not app admin)
```

### 3.13 Logout
```
[Editor] → Logout → Redirect to `/authentication/login`
  Interaction: Click `[data-testid="navbar-user-menu"]`
  Verification: `[data-testid="user-menu-dropdown"]` is visible
  Interaction: Click `[data-testid="user-menu-logout-button"]`
  Verification: URL is `/authentication/login`
```

---

## 4. Org Owner Role

**Permissions:** `dashboard`, `dashboard.*`, `all.read`, `all.write` (org-scoped)

### 4.1 Authentication & Profile Selection
```
[Org Owner] → Login → Single profile → `/dashboard`
  Interaction: Type email in `[data-testid="login-email-input"]`
  Interaction: Type password in `[data-testid="login-password-input"]`
  Interaction: Click `[data-testid="login-submit-button"]`
  Verification: URL is `/dashboard`
  Verification: `[data-testid="dashboard-page"]` is visible

[Org Owner] → Login → Multiple profiles → `/authentication/select-scope` → Select profile → `/dashboard`
  Interaction: Type multi-profile email in `[data-testid="login-email-input"]`
  Interaction: Type password in `[data-testid="login-password-input"]`
  Interaction: Click `[data-testid="login-submit-button"]`
  Verification: URL is `/authentication/select-scope`
  Interaction: Click first `[data-testid="profile-card"]` or `[data-testid="profile-select-button"]`
  Verification: URL is `/dashboard`
```

### 4.2 Dashboard Access (Full)
```
[Org Owner] → `/dashboard`
  Verification: `[data-testid="dashboard-page"]` is visible

  ├─→ Dashboard page loads → Shows full navigation
    Verification: `[data-testid="dashboard-sidebar"]` is visible
    Verification: `[data-testid="dashboard-navbar"]` is visible

  ├─→ Sidebar shows: Dashboard, Organizations, Users, Roles, Permissions, Audit Log
    Verification: `[data-testid="sidebar-dashboard-link"]` is visible
    Verification: `[data-testid="sidebar-organizations-link"]` is visible
    Verification: `[data-testid="sidebar-users-link"]` is visible
    Verification: `[data-testid="sidebar-roles-link"]` is visible
    Verification: `[data-testid="sidebar-permissions-link"]` is visible
    Verification: `[data-testid="sidebar-audit-log-link"]` is visible

  └─→ All pages accessible
    Interaction: Click each sidebar link
    Verification: Each page loads successfully
```

### 4.3 Organizations Page (Full CRUD)
```
[Org Owner] → `/dashboard/organizations`
  Verification: `[data-testid="organizations-page"]` is visible
  Verification: `[data-testid="organizations-create-button"]` is visible

  ├─→ Page loads → Shows organizations list (org-scoped)
    Verification: `[data-testid="organizations-list"]` or `[data-testid="organizations-table"]` is visible

  ├─→ Create organization → Form → Submit → Organization created → List updates
    Interaction: Click `[data-testid="organizations-create-button"]`
    Verification: `[data-testid="organization-create-dialog"]` is visible
    Interaction: Type name in `[data-testid="organization-create-name-input"]`
    Interaction: Type slug in `[data-testid="organization-create-slug-input"]` (if applicable)
    Interaction: Click `[data-testid="organization-create-submit-button"]`
    Verification: `[data-testid="organization-create-dialog"]` closes
    Verification: `[data-testid="organizations-list"]` updates with new organization

  ├─→ Update organization → Form → Submit → Organization updated → List updates
    Interaction: Click first `[data-testid="organization-row"]` or `[data-testid="organization-edit-button"]`
    Verification: `[data-testid="organization-edit-dialog"]` is visible
    Interaction: Update name in `[data-testid="organization-edit-name-input"]`
    Interaction: Click `[data-testid="organization-edit-submit-button"]`
    Verification: `[data-testid="organizations-list"]` updates

  ├─→ Delete organization → Confirm → Organization deleted → List updates
    Interaction: Click `[data-testid="organization-delete-button"]` on a row
    Verification: `[data-testid="organization-delete-confirm-dialog"]` is visible
    Interaction: Click `[data-testid="organization-delete-confirm-button"]`
    Verification: `[data-testid="organizations-list"]` updates (organization removed)

  ├─→ Filter organizations → Results update
    Interaction: Type in `[data-testid="organizations-filter-input"]`
    Verification: `[data-testid="organizations-list"]` updates with filtered results

  └─→ Paginate organizations → Next page loads
    Interaction: Click `[data-testid="organizations-pagination-next"]`
    Verification: `[data-testid="organizations-list"]` shows different entries
```

### 4.4 Users Page (Full CRUD)
```
[Org Owner] → `/dashboard/users`
  Verification: `[data-testid="users-page"]` is visible
  Verification: `[data-testid="users-create-button"]` is visible

  ├─→ Page loads → Shows users list (org-scoped)
    Verification: `[data-testid="users-list"]` is visible

  ├─→ Create user → Form → Submit → User created → List updates
    Interaction: Click `[data-testid="users-create-button"]`
    Verification: `[data-testid="user-create-dialog"]` is visible
    Interaction: Type email in `[data-testid="user-create-email-input"]`
    Interaction: Type password in `[data-testid="user-create-password-input"]`
    Interaction: Click `[data-testid="user-create-submit-button"]`
    Verification: `[data-testid="users-list"]` updates with new user

  ├─→ Update user → Form → Submit → User updated → List updates
    Interaction: Click `[data-testid="user-edit-button"]` on a row
    Verification: `[data-testid="user-edit-dialog"]` is visible
    Interaction: Update email in `[data-testid="user-edit-email-input"]`
    Interaction: Click `[data-testid="user-edit-submit-button"]`
    Verification: `[data-testid="users-list"]` updates

  ├─→ Delete user → Confirm → User deleted → List updates
    Interaction: Click `[data-testid="user-delete-button"]` on a row
    Verification: `[data-testid="user-delete-confirm-dialog"]` is visible
    Interaction: Click `[data-testid="user-delete-confirm-button"]`
    Verification: `[data-testid="users-list"]` updates (user removed)

  └─→ Filter users → Results update
    Interaction: Type in `[data-testid="users-filter-input"]`
    Verification: `[data-testid="users-list"]` updates with filtered results
```

### 4.5 Roles Page (Full CRUD)
```
[Org Owner] → `/dashboard/roles`
  Verification: `[data-testid="roles-page"]` is visible
  Verification: `[data-testid="roles-create-button"]` is visible

  ├─→ Page loads → Shows roles list
    Verification: `[data-testid="roles-list"]` is visible

  ├─→ Create role → Form → Submit → Role created → List updates
    Interaction: Click `[data-testid="roles-create-button"]`
    Verification: `[data-testid="role-create-dialog"]` is visible
    Interaction: Type name in `[data-testid="role-create-name-input"]`
    Interaction: Click `[data-testid="role-create-submit-button"]`
    Verification: `[data-testid="roles-list"]` updates

  ├─→ Update role → Form → Submit → Role updated → List updates
    Interaction: Click `[data-testid="role-edit-button"]` on a row
    Verification: `[data-testid="role-edit-dialog"]` is visible
    Interaction: Update name in `[data-testid="role-edit-name-input"]`
    Interaction: Click `[data-testid="role-edit-submit-button"]`
    Verification: `[data-testid="roles-list"]` updates

  ├─→ Delete role → Confirm → Role deleted → List updates
    Interaction: Click `[data-testid="role-delete-button"]` on a row
    Verification: `[data-testid="role-delete-confirm-dialog"]` is visible
    Interaction: Click `[data-testid="role-delete-confirm-button"]`
    Verification: `[data-testid="roles-list"]` updates (role removed)

  └─→ View role details → Full access
    Interaction: Click `[data-testid="role-row"]`
    Verification: `[data-testid="role-details-dialog"]` is visible
    Verification: `[data-testid="role-edit-button"]` is visible
    Verification: `[data-testid="role-delete-button"]` is visible
```

### 4.6 Permissions Page (Full CRUD)
```
[Org Owner] → `/dashboard/roles-and-permissions`
  Verification: `[data-testid="permissions-page"]` is visible
  Verification: `[data-testid="permission-add-button"]` is visible

  ├─→ Page loads → Shows role-permission assignments
    Verification: `[data-testid="permissions-list"]` is visible

  ├─→ Add permission to role → Form → Submit → Assignment created
    Interaction: Click `[data-testid="permission-add-button"]`
    Verification: `[data-testid="permission-add-dialog"]` is visible
    Interaction: Select role from `[data-testid="permission-add-role-select"]`
    Interaction: Select permission from `[data-testid="permission-add-permission-select"]`
    Interaction: Click `[data-testid="permission-add-submit-button"]`
    Verification: `[data-testid="permissions-list"]` updates

  ├─→ Remove permission from role → Delete → Assignment removed
    Interaction: Click `[data-testid="permission-remove-button"]` on an assignment row
    Verification: `[data-testid="permission-remove-confirm-dialog"]` is visible
    Interaction: Click `[data-testid="permission-remove-confirm-button"]`
    Verification: `[data-testid="permissions-list"]` updates (assignment removed)

  └─→ View all assignments → Full access
    Verification: `[data-testid="permission-add-button"]` is visible
    Verification: `[data-testid="permission-remove-button"]` is visible in all rows
```

### 4.7 Audit Log Page
```
[Org Owner] → `/dashboard/audit-log`
  Verification: `[data-testid="audit-log-page"]` is visible

  ├─→ Page loads → Shows audit entries (org-scoped)
    Verification: `[data-testid="audit-log-list"]` is visible

  ├─→ Filter audit log → Results update
    Interaction: Type in `[data-testid="audit-log-filter-input"]`
    Verification: `[data-testid="audit-log-list"]` updates

  ├─→ Paginate audit log → Next page loads
    Interaction: Click `[data-testid="audit-log-pagination-next"]`
    Verification: `[data-testid="audit-log-list"]` shows different entries

  └─→ View audit entry details → Read-only
    Interaction: Click `[data-testid="audit-log-row"]`
    Verification: `[data-testid="audit-log-details-dialog"]` is visible
```

### 4.8 Profile Management
```
[Org Owner] → `/scope`
  Verification: `[data-testid="profile-page"]` is visible

  ├─→ Page loads → Shows current profile
    Verification: `[data-testid="current-profile"]` is visible
    Verification: `[data-testid="current-profile-org-name"]` is visible

  ├─→ Switch profile (if multiple) → Scope updates → Dashboard refreshes
    Interaction: Click different `[data-testid="profile-card"]` or `[data-testid="profile-switch-button"]`
    Verification: `[data-testid="current-profile"]` updates
    Verification: Navigate to `/dashboard` → `[data-testid="users-list"]` shows users from new scope

  └─→ View profile details → Read-only
    Verification: `[data-testid="profile-details"]` is visible
```

### 4.9 Generic Entity API (Full CRUD - Org-scoped)
```
[Org Owner] → GET `/api/entities/organization` → 200 OK
[Org Owner] → POST `/api/entities/organization` → 201 Created
[Org Owner] → PATCH `/api/entities/organization/{id}` → 200 OK
[Org Owner] → DELETE `/api/entities/organization/{id}` → 200 OK
[Org Owner] → GET `/api/entities/user` → 200 OK (org-scoped)
[Org Owner] → POST `/api/entities/user` → 201 Created
[Org Owner] → PATCH `/api/entities/user/{id}` → 200 OK
[Org Owner] → DELETE `/api/entities/user/{id}` → 200 OK
[Org Owner] → GET `/api/entities/role` → 200 OK
[Org Owner] → POST `/api/entities/role` → 201 Created
[Org Owner] → PATCH `/api/entities/role/{id}` → 200 OK
[Org Owner] → DELETE `/api/entities/role/{id}` → 200 OK
```

### 4.10 RPC API (Full CRUD)
```
[Org Owner] → POST `/api/rpc` → `entity.list` → 200 OK
[Org Owner] → POST `/api/rpc` → `entity.get` → 200 OK
[Org Owner] → POST `/api/rpc` → `entity.create` → 200 OK
[Org Owner] → POST `/api/rpc` → `entity.update` → 200 OK
[Org Owner] → POST `/api/rpc` → `entity.delete` → 200 OK
```

### 4.11 Subscription Stream
```
[Org Owner] → GET `/api/subscriptions/stream` → SSE connection established
  ├─→ Receives "ready" message
  ├─→ Subscribe to all entities → Receives invalidation events
  └─→ Multiple subscriptions → All work correctly
```

### 4.12 Admin Endpoint (Blocked - Not Global Admin)
```
[Org Owner] → GET `/api/auth/admin` → 403 Forbidden (not global admin)
```

### 4.13 Scope Switching
```
[Org Owner] → Multiple orgs → Switch scope → Dashboard updates
  ├─→ Users list changes (org-scoped)
  ├─→ Organizations list changes
  └─→ Permissions resolve correctly for new scope
```

### 4.14 Logout
```
[Org Owner] → Logout → Redirect to `/authentication/login`
```

---

## 5. Org Admin Role

**Permissions:** Same as Org Owner (`dashboard`, `dashboard.*`, `all.read`, `all.write` org-scoped)

**Note:** Org Admin follows the same test scenarios and `data-testid` patterns as Org Owner (Section 4). All interactions and verifications use the same `data-testid` attributes. The only difference is the user credentials used for testing.

### 5.1 Authentication & Profile Selection
```
[Org Admin] → Login → Single profile → `/dashboard`
  Interaction: Type email in `[data-testid="login-email-input"]`
  Interaction: Type password in `[data-testid="login-password-input"]`
  Interaction: Click `[data-testid="login-submit-button"]`
  Verification: URL is `/dashboard`
  Verification: `[data-testid="dashboard-page"]` is visible

[Org Admin] → Login → Multiple profiles → `/authentication/select-scope` → Select profile → `/dashboard`
  Interaction: Type multi-profile email in `[data-testid="login-email-input"]`
  Interaction: Type password in `[data-testid="login-password-input"]`
  Interaction: Click `[data-testid="login-submit-button"]`
  Verification: URL is `/authentication/select-scope`
  Interaction: Click first `[data-testid="profile-card"]` or `[data-testid="profile-select-button"]`
  Verification: URL is `/dashboard`
```

**Note:** All other Org Admin scenarios (5.2-5.14) follow the same patterns as Org Owner (Section 4.2-4.14) with identical `data-testid` attributes.

### 5.2 Dashboard Access (Full - Same as Owner)
```
[Org Admin] → `/dashboard`
  ├─→ Dashboard page loads → Shows full navigation
  ├─→ Sidebar shows: Dashboard, Organizations, Users, Roles, Permissions, Audit Log
  └─→ All pages accessible
```

### 5.3 Organizations Page (Full CRUD)
```
[Org Admin] → `/dashboard/organizations`
  ├─→ Page loads → Shows organizations list (org-scoped)
  ├─→ Create organization → Form → Submit → Organization created
  ├─→ Update organization → Form → Submit → Organization updated
  ├─→ Delete organization → Confirm → Organization deleted
  └─→ Filter and paginate → Works correctly
```

### 5.4 Users Page (Full CRUD)
```
[Org Admin] → `/dashboard/users`
  ├─→ Page loads → Shows users list (org-scoped)
  ├─→ Create user → Form → Submit → User created
  ├─→ Update user → Form → Submit → User updated
  ├─→ Delete user → Confirm → User deleted
  └─→ Manage user roles → Add/remove roles → Updates correctly
```

### 5.5 Roles Page (Full CRUD)
```
[Org Admin] → `/dashboard/roles`
  ├─→ Page loads → Shows roles list
  ├─→ Create role → Form → Submit → Role created
  ├─→ Update role → Form → Submit → Role updated
  ├─→ Delete role → Confirm → Role deleted
  └─→ View role permissions → Full access
```

### 5.6 Permissions Page (Full CRUD)
```
[Org Admin] → `/dashboard/roles-and-permissions`
  ├─→ Page loads → Shows role-permission assignments
  ├─→ Add permission to role → Form → Submit → Assignment created
  ├─→ Remove permission from role → Delete → Assignment removed
  └─→ Bulk operations → Works correctly
```

### 5.7 Audit Log Page
```
[Org Admin] → `/dashboard/audit-log`
  ├─→ Page loads → Shows audit entries (org-scoped)
  ├─→ Filter audit log → Results update
  └─→ Paginate audit log → Next page loads
```

### 5.8 Profile Management
```
[Org Admin] → `/scope`
  ├─→ Page loads → Shows current profile
  ├─→ Switch profile (if multiple) → Scope updates → Dashboard refreshes
  └─→ View profile details → Read-only
```

### 5.9 Generic Entity API (Full CRUD - Org-scoped)
```
[Org Admin] → All CRUD operations → 200 OK / 201 Created (org-scoped)
  ├─→ Organizations → Full CRUD
  ├─→ Users → Full CRUD
  ├─→ Roles → Full CRUD
  └─→ Permissions → Full CRUD
```

### 5.10 RPC API (Full CRUD)
```
[Org Admin] → POST `/api/rpc` → All entity operations → 200 OK
  ├─→ `entity.list` → Works
  ├─→ `entity.get` → Works
  ├─→ `entity.create` → Works
  ├─→ `entity.update` → Works
  └─→ `entity.delete` → Works
```

### 5.11 Subscription Stream
```
[Org Admin] → GET `/api/subscriptions/stream` → SSE connection established
  ├─→ Receives "ready" message
  ├─→ Subscribe to all entities → Receives invalidation events
  └─→ Multiple subscriptions → All work correctly
```

### 5.12 Admin Endpoint (Blocked - Not Global Admin)
```
[Org Admin] → GET `/api/auth/admin` → 403 Forbidden (not global admin)
```

### 5.13 Scope Switching
```
[Org Admin] → Multiple orgs → Switch scope → Dashboard updates
  ├─→ Users list changes (org-scoped)
  ├─→ Organizations list changes
  └─→ Permissions resolve correctly for new scope
```

### 5.14 Logout
```
[Org Admin] → Logout → Redirect to `/authentication/login`
```

---

## 6. App Admin Role (Global Admin)

**Permissions:** `all.read`, `all.write` (global-scope), `platform_admin` global role

### 6.1 Authentication & Profile Selection
```
[App Admin] → Login → Single profile → `/dashboard`
[App Admin] → Login → Multiple profiles → `/authentication/select-scope` → Select profile → `/dashboard`
```

### 6.2 Dashboard Access (Full - Global Scope)
```
[App Admin] → `/dashboard`
  Verification: `[data-testid="dashboard-page"]` is visible

  ├─→ Dashboard page loads → Shows full navigation
    Verification: `[data-testid="dashboard-sidebar"]` is visible
    Verification: `[data-testid="dashboard-navbar"]` is visible

  ├─→ Sidebar shows: Dashboard, Organizations, Users, Roles, Permissions, Audit Log
    Verification: `[data-testid="sidebar-dashboard-link"]` is visible
    Verification: `[data-testid="sidebar-organizations-link"]` is visible
    Verification: `[data-testid="sidebar-users-link"]` is visible
    Verification: `[data-testid="sidebar-roles-link"]` is visible
    Verification: `[data-testid="sidebar-permissions-link"]` is visible
    Verification: `[data-testid="sidebar-audit-log-link"]` is visible

  ├─→ Can access all organizations (global-scope)
    Interaction: Click `[data-testid="sidebar-organizations-link"]`
    Verification: `[data-testid="organizations-list"]` shows organizations from all orgs

  └─→ Can see users from all orgs (global-scope)
    Interaction: Click `[data-testid="sidebar-users-link"]`
    Verification: `[data-testid="users-list"]` shows users from all orgs
```

**Note:** App Admin scenarios (6.3-6.16) follow similar patterns to Org Owner (Section 4) but with global scope. All `data-testid` attributes are the same, but verifications check for global-scope data (users/orgs from all organizations).

### 6.3 Organizations Page (Full CRUD - Global Scope)
```
[App Admin] → `/dashboard/organizations`
  ├─→ Page loads → Shows ALL organizations (global-scope)
  ├─→ Create organization → Form → Submit → Organization created → List updates
  ├─→ Update organization → Form → Submit → Organization updated → List updates
  ├─→ Delete organization → Confirm → Organization deleted → List updates
  ├─→ Filter organizations → Results update (across all orgs)
  └─→ Paginate organizations → Next page loads
```

### 6.4 Users Page (Full CRUD - Global Scope)
```
[App Admin] → `/dashboard/users`
  ├─→ Page loads → Shows users from ALL organizations (global-scope)
  ├─→ Create user → Form → Submit → User created → List updates
  ├─→ Update user → Form → Submit → User updated → List updates
  ├─→ Delete user → Confirm → User deleted → List updates
  ├─→ Filter users → Results update (across all orgs)
  └─→ Assign user to any org → Works correctly
```

### 6.5 Roles Page (Full CRUD - Global Scope)
```
[App Admin] → `/dashboard/roles`
  ├─→ Page loads → Shows roles from all orgs
  ├─→ Create role → Form → Submit → Role created → List updates
  ├─→ Update role → Form → Submit → Role updated → List updates
  ├─→ Delete role → Confirm → Role deleted → List updates
  └─→ View role permissions → Full access (global + org-scoped)
```

### 6.6 Permissions Page (Full CRUD - Global Scope)
```
[App Admin] → `/dashboard/roles-and-permissions`
  ├─→ Page loads → Shows role-permission assignments (all orgs)
  ├─→ Add permission to role → Form → Submit → Assignment created
  ├─→ Remove permission from role → Delete → Assignment removed
  └─→ Manage global role permissions → Works correctly
```

### 6.7 Audit Log Page (Global Scope)
```
[App Admin] → `/dashboard/audit-log`
  ├─→ Page loads → Shows audit entries from ALL organizations (global-scope)
  ├─→ Filter audit log → Results update (across all orgs)
  ├─→ Paginate audit log → Next page loads
  └─→ View audit entry details → Full access
```

### 6.8 Profile Management
```
[App Admin] → `/scope`
  ├─→ Page loads → Shows current profile
  ├─→ Switch profile (if multiple) → Scope updates → Dashboard refreshes
  ├─→ Switch to global scope → Can see all orgs
  └─→ View profile details → Read-only
```

### 6.9 Admin Endpoint (Access Granted)
```
[App Admin] → GET `/api/auth/admin` → 200 OK → "access granted"
```

### 6.10 Generic Entity API (Full CRUD - Global Scope)
```
[App Admin] → GET `/api/entities/organization` → 200 OK (all orgs)
[App Admin] → POST `/api/entities/organization` → 201 Created
[App Admin] → PATCH `/api/entities/organization/{id}` → 200 OK (any org)
[App Admin] → DELETE `/api/entities/organization/{id}` → 200 OK (any org)
[App Admin] → GET `/api/entities/user` → 200 OK (all users, all orgs)
[App Admin] → POST `/api/entities/user` → 201 Created (any org)
[App Admin] → PATCH `/api/entities/user/{id}` → 200 OK (any user)
[App Admin] → DELETE `/api/entities/user/{id}` → 200 OK (any user)
```

### 6.11 RPC API (Full CRUD - Global Scope)
```
[App Admin] → POST `/api/rpc` → `entity.list` → 200 OK (all entities, all orgs)
[App Admin] → POST `/api/rpc` → `entity.get` → 200 OK (any entity)
[App Admin] → POST `/api/rpc` → `entity.create` → 200 OK (any org)
[App Admin] → POST `/api/rpc` → `entity.update` → 200 OK (any entity)
[App Admin] → POST `/api/rpc` → `entity.delete` → 200 OK (any entity)
```

### 6.12 Subscription Stream (Global Scope)
```
[App Admin] → GET `/api/subscriptions/stream` → SSE connection established
  ├─→ Receives "ready" message
  ├─→ Subscribe to all entities → Receives invalidation events (all orgs)
  ├─→ Subscribe to global-scope entities → Works correctly
  └─→ Multiple subscriptions → All work correctly
```

### 6.13 Global Role Management
```
[App Admin] → Manage global roles → Add/remove global role assignments
  ├─→ Assign global role to user → User gets global permissions
  ├─→ Remove global role from user → User loses global permissions
  └─→ View global role permissions → Full access
```

### 6.14 Scope Switching (Global Admin)
```
[App Admin] → Multiple orgs → Switch scope → Dashboard updates
  ├─→ Users list changes (org-scoped view)
  ├─→ Organizations list changes
  ├─→ Switch to global scope → See all orgs
  └─→ Permissions resolve correctly (global + org-scoped)
```

### 6.15 API Token Management
```
[App Admin] → POST `/api/auth/tokens` → 201 Created → Token returned
[App Admin] → Use token → GET `/api/auth/me` → 200 OK → Shows admin status
[App Admin] → Use token → GET `/api/auth/admin` → 200 OK
```

### 6.16 Logout
```
[App Admin] → Logout → Redirect to `/authentication/login`
```

---

## 7. Cross-Role Scenarios

### 7.1 Multi-Organization User Flow
```
[User with multiple orgs] → Login → `/authentication/select-scope`
  Interaction: Type multi-org email in `[data-testid="login-email-input"]`
  Interaction: Type password in `[data-testid="login-password-input"]`
  Interaction: Click `[data-testid="login-submit-button"]`
  Verification: URL is `/authentication/select-scope`
  Verification: `[data-testid="profile-list"]` shows multiple profiles

  ├─→ Select Org A → `/dashboard` → See Org A data
    Interaction: Click `[data-testid="profile-card"]` for Org A
    Verification: URL is `/dashboard`
    Verification: `[data-testid="users-list"]` shows users from Org A only

  ├─→ Navigate to `/scope` → Switch to Org B → `/dashboard` → See Org B data
    Interaction: Navigate to `/scope`
    Interaction: Click `[data-testid="profile-card"]` for Org B
    Interaction: Navigate to `/dashboard`
    Verification: `[data-testid="users-list"]` shows users from Org B only

  ├─→ Switch back to Org A → `/dashboard` → See Org A data again
    Interaction: Navigate to `/scope`
    Interaction: Click `[data-testid="profile-card"]` for Org A
    Interaction: Navigate to `/dashboard`
    Verification: `[data-testid="users-list"]` shows users from Org A again

  └─→ Each org has different role → Permissions change per scope
    Verification: Sidebar shows different links based on role in selected org
    Verification: `[data-testid="users-create-button"]` visibility changes based on role
```

### 7.2 Permission Escalation Prevention
```
[Viewer] → Attempt to access editor-only features → 403 Forbidden
  Verification: `[data-testid="users-create-button"]` is not visible
  Interaction: Navigate directly to create endpoint → Verify 403 response
  Verification: `[data-testid="error-403"]` is visible

[Editor] → Attempt to access org-admin-only features → 403 Forbidden
  Verification: `[data-testid="sidebar-organizations-link"]` is not visible
  Interaction: Navigate directly to `/dashboard/organizations` → Verify 403 response

[Org Admin] → Attempt to access global-admin-only features → 403 Forbidden
  Interaction: Navigate to `/api/auth/admin`
  Verification: 403 response or `[data-testid="error-403"]` is visible

[App Admin] → Access global-admin features → 200 OK
  Interaction: Navigate to `/api/auth/admin`
  Verification: 200 OK response
  Verification: Response contains "access granted" or success indicator
```

### 7.3 Concurrent Operations
```
[User] → Multiple tabs open → Create entity in Tab 1
  Tab 1: Interaction: Click `[data-testid="users-create-button"]`
  Tab 1: Interaction: Fill `[data-testid="user-create-form"]`
  Tab 1: Interaction: Click `[data-testid="user-create-submit-button"]`
  Tab 1: Verification: `[data-testid="users-list"]` updates

  ├─→ Tab 2 → Subscription stream → Receives invalidation event
    Tab 2: Verification: SSE connection established via `[data-testid="subscription-stream"]`
    Tab 2: Verification: Receives invalidation event for user entity

  ├─→ Tab 2 → List refreshes automatically
    Tab 2: Verification: `[data-testid="users-list"]` updates automatically
    Tab 2: Verification: New `[data-testid="user-row"]` appears

  └─→ Tab 3 → Same entity → Shows updated data
    Tab 3: Interaction: Navigate to `/dashboard/users`
    Tab 3: Verification: `[data-testid="users-list"]` shows the new user
```

### 7.4 Error Handling
```
[Any User] → Invalid API request → 400 Bad Request → Error message displayed
[Any User] → Unauthorized request → 401 Unauthorized → Redirect to login
[Any User] → Forbidden request → 403 Forbidden → Error message displayed
[Any User] → Not found → 404 Not Found → Error message displayed
[Any User] → Server error → 500 Internal Server Error → Error message displayed
```

### 7.5 Session Management
```
[Any User] → Login → Session established → Token stored
  ├─→ Token expires → Next request → 401 Unauthorized → Redirect to login
  ├─→ Logout → Token invalidated → Protected routes redirect to login
  └─→ Multiple devices → Each has separate session
```

### 7.6 Data Consistency
```
[User] → Create entity → Entity appears in list
[User] → Update entity → Changes reflected immediately
[User] → Delete entity → Entity removed from list
[User] → Filter → Results match filter criteria
[User] → Pagination → No duplicates, no missing items
```

---

## 8. Edge Cases & Boundary Conditions

### 8.1 Empty States
```
[Any User] → Empty organization → Organizations page shows empty state
[Any User] → Empty user list → Users page shows empty state
[Any User] → Empty role list → Roles page shows empty state
[Any User] → Empty audit log → Audit log page shows empty state
```

### 8.2 Large Datasets
```
[Any User] → Large user list → Pagination works correctly
[Any User] → Large organization list → Pagination works correctly
[Any User] → Large audit log → Pagination works correctly
[Any User] → Cursor pagination → Next cursor works correctly
```

### 8.3 Invalid Inputs
```
[Any User] → Invalid email format → Validation error
[Any User] → Short password → Validation error
[Any User] → Invalid UUID → 400 Bad Request
[Any User] → Invalid filter JSON → 400 Bad Request
[Any User] → Invalid sort field → 400 Bad Request
```

### 8.4 Network Conditions
```
[Any User] → Slow network → Loading states displayed
[Any User] → Network error → Error message displayed
[Any User] → Timeout → Error message displayed
[Any User] → Offline → Error message displayed
```

### 8.5 Browser Compatibility
```
[Any User] → Different browsers → App works correctly
[Any User] → Mobile viewport → Responsive layout works
[Any User] → Dark/light theme → Theme toggle works
[Any User] → Browser back/forward → Navigation works correctly
```

---

## 9. Security Scenarios

### 9.1 Authentication Security
```
[Guest] → Attempt to access protected route → Redirect to login
[Guest] → Attempt to use expired token → 401 Unauthorized
[Guest] → Attempt to use invalid token → 401 Unauthorized
[Guest] → Attempt SQL injection in login → Sanitized/rejected
[Guest] → Attempt XSS in login → Sanitized/rejected
```

### 9.2 Authorization Security
```
[Viewer] → Attempt to create entity → 403 Forbidden
[Editor] → Attempt to delete entity → 403 Forbidden (if no delete permission)
[Org Admin] → Attempt to access other org's data → 403 Forbidden (org-scoped)
[App Admin] → Access any org's data → 200 OK (global-scope)
```

### 9.3 CSRF Protection
```
[Any User] → Form submission → CSRF token validated
[Any User] → Missing CSRF token → Request rejected
```

### 9.4 Rate Limiting
```
[Guest] → Multiple rapid requests → Rate limited → 429 Too Many Requests
[Any User] → Health endpoints → Not rate limited → Always 200 OK
```

### 9.5 Security Headers
```
[Any User] → Any request → Security headers present
  ├─→ `X-Content-Type-Options: nosniff`
  ├─→ `X-Frame-Options: DENY`
  ├─→ `Referrer-Policy: strict-origin-when-cross-origin`
  ├─→ `Content-Security-Policy` with frame-ancestors
  ├─→ `Permissions-Policy: geolocation=()`
  └─→ `Cross-Origin-Resource-Policy: same-site`
```

---

## 10. Test Execution Order

### Critical Path Priority:
1. **Guest/Unauthenticated** → Authentication flow → Registration → Login
2. **Viewer** → Basic read-only access → Verify permissions
3. **Editor** → Create/update operations → Verify permissions
4. **Org Owner** → Full org-scoped CRUD → Verify scope isolation
5. **Org Admin** → Same as owner → Verify consistency
6. **App Admin** → Global-scope access → Verify global permissions
7. **Cross-Role** → Multi-org users → Scope switching
8. **Edge Cases** → Empty states → Large datasets → Invalid inputs
9. **Security** → Authentication → Authorization → Headers

### DAG Structure Notes:
- Each scenario is a node in the DAG
- Edges represent transitions between scenarios
- No cycles (loops) - each path is acyclic
- Critical paths are highlighted for priority testing
- Scenarios can be executed in parallel where independent

---

## 11. Test Data Requirements

### Seed Data Needed:
- **Guest users**: None (unauthenticated)
- **Viewer**: `viewer@default.org` (password: `password`)
- **Editor**: `editor@default.org` (password: `password`)
- **Org Owner**: `owner@default.org` (password: `password`)
- **Org Admin**: `orgadmin@default.org` (password: `password`)
- **App Admin**: `admin@admin.com` (password: `password`)
- **Multi-org user**: `multi@email.com` (password: `password`) - multiple orgs

### Test Organizations:
- **Default** org (slug: `default`)
- **Other** org (slug: `other`)
- **CoolOrg** org (slug: `coolorg`)
- **Admin** org (slug: `admin`)

### Test Entities:
- Organizations: Default, Other, CoolOrg, Admin
- Users: One per role per org
- Roles: owner, admin, editor, viewer (per org)
- Permissions: Entity-based permissions assigned to roles
- Audit entries: Generated by operations

---

## 12. Data-TestID Implementation Notes

### Implementation Status
All `data-testid` attributes listed in this document are **planned** for test implementation. They should be added to the frontend codebase during development to support E2E testing.

### Pattern Consistency
- **Pages**: Use `{page-name}-page` (e.g., `login-page`, `dashboard-page`)
- **Forms**: Use `{entity}-{action}-form` (e.g., `user-create-form`)
- **Dialogs**: Use `{entity}-{action}-dialog` (e.g., `user-create-dialog`)
- **Buttons**: Use `{entity}-{action}-button` (e.g., `user-create-button`, `user-edit-button`)
- **Lists/Tables**: Use `{entity}-list` or `{entity}-table`
- **Rows**: Use `{entity}-row` (e.g., `user-row`, `organization-row`)
- **Inputs**: Use `{entity}-{field}-input` (e.g., `user-email-input`)
- **Navigation**: Use `sidebar-{page}-link` (e.g., `sidebar-users-link`)
- **Error Messages**: Use `{entity}-error-message` or `error-{code}` (e.g., `error-403`)

### Remaining Sections
Sections that reference API endpoints (e.g., 2.9-2.13, 4.9-4.14, 6.9-6.16) primarily test backend behavior and may not require frontend `data-testid` attributes. However, any UI interactions related to these endpoints should follow the same patterns.

Sections 8 (Edge Cases) and 9 (Security) may require additional `data-testid` attributes as specific UI elements are implemented. Follow the established patterns when adding them.

### Testing Framework Integration
When implementing E2E tests (e.g., with Playwright), use these `data-testid` attributes for:
- **Locating elements**: `page.locator('[data-testid="login-email-input"]')`
- **Assertions**: `expect(page.locator('[data-testid="dashboard-page"]')).toBeVisible()`
- **Interactions**: `page.click('[data-testid="login-submit-button"]')`

---

## 13. Missing Scenarios & Gaps

### ⚠️ Critical Gaps (Must Add Before Production)

#### 13.1 Sorting Functionality
**Status:** API supports sorting, but E2E scenarios are missing
```
[Any User] → Sort users by email → List updates
  Interaction: Click `[data-testid="users-sort-email-header"]` or `[data-testid="users-sort-select"]`
  Verification: `[data-testid="users-list"]` sorts by email ascending
  Interaction: Click again or select desc
  Verification: `[data-testid="users-list"]` sorts by email descending
  Verification: Sort indicator `[data-testid="users-sort-indicator"]` shows direction

[Any User] → Sort by different fields → Each field sorts correctly
  Interaction: Select different sort field from `[data-testid="users-sort-field-select"]`
  Verification: List updates with correct sort order
  Verification: Invalid sort field → Error message displayed

[Any User] → Sort + Filter combination → Both work together
  Interaction: Apply filter via `[data-testid="users-filter-input"]`
  Interaction: Apply sort via `[data-testid="users-sort-select"]`
  Verification: Filtered results are sorted correctly
```

#### 13.2 Loading States
**Status:** Mentioned but not detailed with data-testid
```
[Any User] → Page loads → Loading indicators shown
  Verification: `[data-testid="users-loading-spinner"]` or `[data-testid="users-skeleton"]` is visible during load
  Verification: Loading indicator disappears when data loads
  Verification: `[data-testid="users-list"]` appears after loading completes

[Any User] → Form submission → Loading state shown
  Interaction: Click `[data-testid="user-create-submit-button"]`
  Verification: `[data-testid="user-create-submit-button"]` shows loading state (disabled/spinner)
  Verification: Button re-enables after submission completes
```

#### 13.3 Empty States (Detailed)
**Status:** Mentioned but not detailed with data-testid
```
[Any User] → Empty user list → Empty state displayed
  Verification: `[data-testid="users-empty-state"]` is visible
  Verification: `[data-testid="users-empty-message"]` contains helpful text
  Verification: `[data-testid="users-empty-action-button"]` is visible (if user can create)
  Interaction: Click `[data-testid="users-empty-action-button"]`
  Verification: Create dialog opens

[Any User] → Empty after filter → Empty filter state displayed
  Interaction: Apply filter that returns no results
  Verification: `[data-testid="users-empty-filter-state"]` is visible
  Verification: `[data-testid="users-clear-filter-button"]` is visible
  Interaction: Click `[data-testid="users-clear-filter-button"]`
  Verification: Filter clears and list shows all items
```

#### 13.4 Form Validation (Comprehensive)
**Status:** Basic validation covered, but not comprehensive
```
[Any User] → Create user form → All validation rules tested
  Interaction: Click `[data-testid="users-create-button"]`
  Interaction: Leave email empty → Click submit
  Verification: `[data-testid="user-create-email-error"]` shows "Email is required"
  
  Interaction: Type invalid email format
  Verification: `[data-testid="user-create-email-error"]` shows "Invalid email format"
  
  Interaction: Type valid email → Leave password empty → Click submit
  Verification: `[data-testid="user-create-password-error"]` shows "Password is required"
  
  Interaction: Type password < 8 characters
  Verification: `[data-testid="user-create-password-error"]` shows "Password must be at least 8 characters"
  
  Interaction: Fix all errors → Click submit
  Verification: All error messages disappear
  Verification: Form submits successfully
```

#### 13.5 Session Expiration Handling
**Status:** Mentioned but not detailed
```
[Any User] → Session expires during use → Graceful handling
  Verification: User is working on a form
  Interaction: Wait for token to expire (or manually expire)
  Interaction: Submit form or navigate
  Verification: `[data-testid="session-expired-dialog"]` or toast is visible
  Verification: User is redirected to `/authentication/login`
  Verification: Form data is preserved (if applicable) or user is informed data may be lost
```

#### 13.6 Concurrent Edits
**Status:** Not covered - critical for data integrity
```
[User A] → Edit entity → [User B] → Edit same entity → Conflict handling
  User A: Interaction: Open edit dialog for entity
  User B: Interaction: Open edit dialog for same entity
  User A: Interaction: Submit changes
  User B: Interaction: Submit changes
  Verification: User B sees `[data-testid="entity-conflict-error"]` or conflict resolution dialog
  Verification: User B can see User A's changes via `[data-testid="entity-conflict-diff"]`
  Verification: User B can choose to overwrite or merge changes
```

#### 13.7 Optimistic Updates & Rollback
**Status:** Not covered
```
[Any User] → Update entity → Network fails → Rollback
  Interaction: Update entity via `[data-testid="user-edit-form"]`
  Interaction: Click `[data-testid="user-edit-submit-button"]`
  Verification: UI updates optimistically (before API response)
  Simulation: Network request fails
  Verification: `[data-testid="user-edit-error-message"]` shows error
  Verification: UI rolls back to previous state
  Verification: `[data-testid="users-list"]` shows original data
```

### 🔶 Important Gaps (Should Add)

#### 13.8 Keyboard Navigation & Accessibility
**Status:** Not covered at all
```
[Any User] → Keyboard navigation → All features accessible
  Interaction: Press Tab key
  Verification: Focus moves through interactive elements in logical order
  Interaction: Press Enter on focused button
  Verification: Button action executes
  Interaction: Press Escape on dialog
  Verification: Dialog closes
  Verification: Focus returns to trigger element
  
[Any User] → Screen reader → Content is accessible
  Verification: All images have alt text
  Verification: Form inputs have labels
  Verification: ARIA labels are present for complex components
  Verification: Error messages are announced to screen readers
```

#### 13.9 Pagination Edge Cases
**Status:** Basic pagination covered, but edge cases missing
```
[Any User] → Pagination edge cases → Handled correctly
  Interaction: Navigate to last page
  Verification: `[data-testid="users-pagination-next"]` is disabled
  Interaction: Navigate to first page
  Verification: `[data-testid="users-pagination-prev"]` is disabled
  Interaction: Change page size while on page 5
  Verification: List recalculates and shows appropriate page
  Interaction: Delete all items on current page
  Verification: Automatically navigates to previous page or shows empty state
```

#### 13.10 Filter Persistence
**Status:** Not covered
```
[Any User] → Apply filters → Navigate away → Return → Filters persist
  Interaction: Apply filter via `[data-testid="users-filter-input"]`
  Interaction: Navigate to different page
  Interaction: Navigate back to users page
  Verification: Filter is still applied
  Verification: `[data-testid="users-filter-input"]` shows previous value
  Verification: List shows filtered results
```

#### 13.11 Bulk Operations
**Status:** Not covered - may not exist in current implementation
```
[Admin] → Select multiple entities → Bulk actions
  Interaction: Check multiple `[data-testid="user-checkbox"]` or `[data-testid="user-select-checkbox"]`
  Verification: `[data-testid="users-bulk-actions-bar"]` appears
  Verification: Bulk action buttons are visible
  Interaction: Click `[data-testid="users-bulk-delete-button"]`
  Verification: Confirmation dialog shows count
  Interaction: Confirm
  Verification: Selected entities are deleted
  Verification: `[data-testid="users-list"]` updates
```

#### 13.12 Export Functionality
**Status:** Not covered - may not exist
```
[Any User] → Export data → File downloads
  Interaction: Click `[data-testid="users-export-button"]`
  Verification: Export options dialog appears
  Interaction: Select format (CSV/JSON) via `[data-testid="export-format-select"]`
  Interaction: Click `[data-testid="export-confirm-button"]`
  Verification: File download starts
  Verification: File contains correct data
```

#### 13.13 Search vs Filter
**Status:** Filtering covered, but search might be different
```
[Any User] → Search functionality → Results update
  Interaction: Type in `[data-testid="users-search-input"]` (if different from filter)
  Verification: Search results update in real-time or on submit
  Verification: `[data-testid="users-search-clear-button"]` appears
  Interaction: Click clear
  Verification: Search clears and full list shows
```

#### 13.14 User Account Self-Management
**Status:** Not covered
```
[Any User] → Edit own profile → Changes apply
  Interaction: Navigate to own profile page
  Verification: `[data-testid="profile-edit-button"]` is visible
  Interaction: Click edit
  Verification: `[data-testid="profile-edit-form"]` is visible
  Interaction: Update email or other fields
  Interaction: Click `[data-testid="profile-save-button"]`
  Verification: Changes saved
  Verification: User info updates in navbar `[data-testid="navbar-user-email"]`
```

#### 13.15 Error Recovery
**Status:** Basic error handling covered, but recovery not detailed
```
[Any User] → Error occurs → User can recover
  Simulation: Network error during form submission
  Verification: `[data-testid="form-error-message"]` shows error
  Verification: `[data-testid="form-retry-button"]` is visible
  Interaction: Click retry
  Verification: Form resubmits
  Verification: Success on retry
```

### 🔷 Nice-to-Have Gaps

#### 13.16 Performance Testing
**Status:** Not covered
```
[Any User] → Large dataset → Performance acceptable
  Verification: Page load time < 2 seconds
  Verification: List renders < 100ms for 100 items
  Verification: Filter/search responds < 300ms
  Verification: No janky scrolling or animations
```

#### 13.17 Mobile/Responsive Testing
**Status:** Mentioned but not detailed
```
[Any User] → Mobile viewport → UI adapts
  Verification: Sidebar collapses to hamburger menu
  Verification: Tables become cards or scrollable
  Verification: Forms are usable on small screens
  Verification: Touch targets are at least 44x44px
```

#### 13.18 Browser-Specific Testing
**Status:** Mentioned but not detailed
```
[Any User] → Different browsers → Consistent behavior
  Test on: Chrome, Firefox, Safari, Edge
  Verification: All features work identically
  Verification: No browser-specific bugs
```

#### 13.19 Internationalization
**Status:** i18n exists but not tested
```
[Any User] → Change language → UI updates
  Interaction: Select language from `[data-testid="language-selector"]`
  Verification: All text updates to selected language
  Verification: Date formats update
  Verification: RTL languages display correctly (if supported)
```

#### 13.20 Undo/Redo
**Status:** Not covered - may not exist
```
[Any User] → Delete entity → Undo available
  Interaction: Delete entity
  Verification: `[data-testid="undo-toast"]` appears
  Interaction: Click `[data-testid="undo-button"]`
  Verification: Entity is restored
  Verification: `[data-testid="users-list"]` shows entity again
```

---

## 14. Production Readiness Checklist

### ✅ Covered
- [x] Basic authentication flows (login, register, logout)
- [x] Permission-based access control
- [x] CRUD operations for all entities
- [x] Scope switching (multi-org)
- [x] Basic error handling (401, 403, 404, 500)
- [x] Filtering functionality
- [x] Basic pagination
- [x] Theme toggle
- [x] Security headers

### ⚠️ Partially Covered (Needs Enhancement)
- [ ] Sorting (API exists, UI scenarios missing)
- [ ] Loading states (mentioned, needs data-testid)
- [ ] Empty states (mentioned, needs data-testid)
- [ ] Form validation (basic covered, needs comprehensive)
- [ ] Session expiration (mentioned, needs detailed flow)

### ❌ Not Covered (Critical for Production)
- [ ] Concurrent edits conflict resolution
- [ ] Optimistic updates & rollback
- [ ] Keyboard navigation & accessibility
- [ ] Error recovery flows
- [ ] User self-management (edit own profile)

### 📋 Recommended Before Production
1. **Add all Critical Gaps (13.1-13.7)** - These are essential for data integrity and user experience
2. **Add Important Gaps (13.8-13.15)** - These prevent user frustration and support accessibility
3. **Consider Nice-to-Have (13.16-13.20)** - Based on user requirements and priorities

---

*Last updated: Generated from codebase analysis and FUNCTIONALITY_AND_TESTS.md*
