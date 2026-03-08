# Missing Test IDs - E2E Tests vs Frontend Implementation

## Summary

This document lists all test IDs that are expected by the e2e tests but are missing or mismatched in the frontend implementation.

**Last Updated**: Implementation completed for LoginPage, RegisterPage, DashboardPage, SelectScopePage, ProfilePage, UsersPage, RolesPage, FormDialog component, and partial implementation for remaining pages.

---

## ✅ COMPLETED IMPLEMENTATIONS

### LoginPage (`/features/authentication/pages/LoginPage/LoginPage.tsx`)
✅ **All test IDs implemented**

| Expected Test ID | Status | Notes |
|-----------------|--------|-------|
| `login-page` | ✅ Added | Wraps entire component |
| `login-email-input` | ✅ Added | Changed from `login-email` |
| `login-password-input` | ✅ Added | Changed from `login-password` |
| `login-submit-button` | ✅ Added | Changed from `login-submit` |
| `login-register-link` | ✅ Added | Added to Link component |
| `login-error-message` | ✅ Added | Changed from `login-error` |

### RegisterPage (`/features/authentication/pages/RegisterPage/RegisterPage.tsx`)
✅ **All test IDs implemented**

| Expected Test ID | Status | Notes |
|-----------------|--------|-------|
| `register-page` | ✅ Added | Wraps entire component |
| `register-email-input` | ✅ Added | Changed from `register-email` |
| `register-password-input` | ✅ Added | Changed from `register-password` |
| `register-submit-button` | ✅ Added | Changed from `register-submit` |
| `register-login-link` | ✅ Added | Added to Link component |
| `register-error-message` | ✅ Added | Changed from `register-error` |

### DashboardPage (`/features/dashboard/pages/DashboardPage/DashboardPage.tsx`)
✅ **All test IDs implemented**

| Expected Test ID | Status | Notes |
|-----------------|--------|-------|
| `dashboard-page` | ✅ Added | Wraps PageHeader |
| `dashboard-layout` | ✅ Added | Added to DashboardLayout wrapper |
| `dashboard-sidebar` | ✅ Added | Added to Sidebar component |
| `dashboard-navbar` | ✅ Added | Added to Navbar AppBar |
| `dashboard-content` | ✅ Added | Added to content Box |
| `sidebar-dashboard-link` | ✅ Added | Added to NavLink |
| `sidebar-users-link` | ✅ Added | Added to NavLink |
| `sidebar-roles-link` | ✅ Added | Added to NavLink |
| `sidebar-permissions-link` | ✅ Added | Added to NavLink |
| `sidebar-audit-log-link` | ✅ Added | Added to NavLink |
| `sidebar-organizations-link` | ✅ Added | Added to NavLink |
| `sidebar-toggle-button` | ✅ Added | Added to toggle IconButton |
| `navbar-theme-toggle-button` | ✅ Added | Changed from `theme-toggle-button` |
| `navbar-user-menu-button` | ✅ Added | Changed from `navbar-user-menu` |
| `navbar-user-menu` | ✅ Added | Added to Menu component |
| `navbar-user-menu-scope` | ✅ Added | Added to Scope MenuItem |
| `navbar-user-menu-logout` | ✅ Added | Changed from `user-menu-logout-button` |

### SelectScopePage (`/features/authentication/pages/SelectScopePage/SelectScopePage.tsx`)
✅ **All test IDs implemented**

| Expected Test ID | Status | Notes |
|-----------------|--------|-------|
| `select-scope-page` | ✅ Added | Wraps entire component |
| `profile-list` | ✅ Added | Added to Box containing cards |
| `profile-card-{index}` | ✅ Added | Changed from dynamic `scope-${org_name}-${role}` to indexed |
| `profile-select-button-{index}` | ✅ Added | Changed from dynamic to indexed |

### ProfilePage (`/features/profile/pages/ProfilePage/ProfilePage.tsx`)
✅ **Basic test IDs implemented** (Note: Many elements expected by tests don't exist)

| Expected Test ID | Status | Notes |
|-----------------|--------|-------|
| `profile-page` | ✅ Added | Wraps entire component |
| `profile-page-title` | ✅ Added | Added to PageHeader |
| `profile-details` | ✅ Added | Added to Box containing user info |
| `current-profile` | ⚠️ Element doesn't exist | Not implemented in current UI |
| `current-profile-org-name` | ⚠️ Element doesn't exist | Not implemented in current UI |
| `current-profile-role-name` | ⚠️ Element doesn't exist | Not implemented in current UI |
| `profile-list` | ⚠️ Element doesn't exist | Not implemented in current UI |
| `profile-card-{index}` | ⚠️ Element doesn't exist | Not implemented in current UI |
| `profile-switch-button-{index}` | ⚠️ Element doesn't exist | Not implemented in current UI |

### UsersPage (`/features/dashboard/pages/UsersPage/UsersPage.tsx`)
✅ **All test IDs implemented**

| Expected Test ID | Status | Notes |
|-----------------|--------|-------|
| `users-page` | ✅ Added | Wraps entire component |
| `users-page-title` | ✅ Added | Added to PageHeader |
| `users-list` | ✅ Added | Added to TableContainer |
| `users-table` | ✅ Added | Added to Table |
| `users-filter-input` | ✅ Added | Added to UsersFilters TextField |
| `users-create-button` | ✅ Added | Added to Add user button |
| `user-row` | ✅ Added | Added to UserTableRow TableRow |
| `user-edit-button` | ✅ Added | Added to edit IconButton |
| `user-delete-button` | ✅ Added | Added to delete IconButton |
| `user-create-dialog` | ✅ Added | Added to FormDialog |
| `user-create-form` | ✅ Added | Added to form Box |
| `user-create-email-input` | ✅ Added | Added to email TextField |
| `user-create-password-input` | ✅ Added | Added to password TextField |
| `user-create-submit-button` | ✅ Added | Added via FormDialog submitButtonTestId prop |
| `user-edit-dialog` | ✅ Added | Added to FormDialog |
| `user-edit-form` | ✅ Added | Added to form Box |
| `user-edit-email-input` | ✅ Added | Added to email TextField |
| `user-edit-submit-button` | ✅ Added | Added via FormDialog submitButtonTestId prop |
| `user-view-button` | ⚠️ Not in current implementation | No view button in UserTableRow |
| `user-details-dialog` | ⚠️ Not in current implementation | No details dialog |
| `user-delete-confirm-dialog` | ⚠️ Not in current implementation | No confirmation dialog |

### RolesPage (`/features/dashboard/pages/RolesPage/RolesPage.tsx`)
✅ **All test IDs implemented**

| Expected Test ID | Status | Notes |
|-----------------|--------|-------|
| `roles-page` | ✅ Added | Wraps entire component |
| `roles-page-title` | ✅ Added | Added to PageHeader |
| `roles-list` | ✅ Added | Added to TableContainer |
| `roles-table` | ✅ Added | Added to Table |
| `roles-create-button` | ✅ Added | Added to Add role button |
| `role-row-{name}` | ✅ Added | Added to RoleTableRow TableRow |
| `role-edit-button-{name}` | ✅ Added | Added to edit IconButton |
| `role-delete-button-{name}` | ✅ Added | Added to delete IconButton |
| `role-create-dialog` | ✅ Added | Added to FormDialog |
| `role-create-form` | ✅ Added | Added to form Box |
| `role-create-name-input` | ✅ Added | Added to name TextField |
| `role-create-submit-button` | ✅ Added | Added via FormDialog submitButtonTestId prop |
| `role-edit-dialog` | ✅ Added | Added to FormDialog |
| `role-edit-form` | ✅ Added | Added to form Box |
| `role-edit-name-input` | ✅ Added | Added to name TextField |
| `role-edit-submit-button` | ✅ Added | Added via FormDialog submitButtonTestId prop |
| `role-view-button-{name}` | ⚠️ Not in current implementation | No view button in RoleTableRow |
| `role-details-dialog` | ⚠️ Not in current implementation | No details dialog |
| `role-delete-confirm-dialog` | ⚠️ Not in current implementation | No confirmation dialog |

### FormDialog Component (`/components/FormDialog.tsx`)
✅ **Test ID support added**

- ✅ Added `data-testid` prop support
- ✅ Added `submitButtonTestId` prop for submit button test IDs

---

## ⏳ REMAINING WORK

### OrganizationsPage (`/features/dashboard/pages/OrganizationsPage/OrganizationsPage.tsx`)
❌ **Not yet implemented**

| Expected Test ID | Status |
|-----------------|--------|
| `organizations-page` | ❌ Missing |
| `organizations-page-title` | ❌ Missing |
| `organizations-list` | ❌ Missing |
| `organizations-table` | ❌ Missing |
| `organizations-filter-input` | ❌ Missing |
| `organizations-create-button` | ❌ Missing |
| `organization-row-{name}` | ❌ Missing |
| `organization-edit-button-{name}` | ❌ Missing |
| `organization-delete-button-{name}` | ❌ Missing |
| `organization-create-dialog` | ❌ Missing |
| `organization-create-form` | ❌ Missing |
| `organization-create-name-input` | ❌ Missing |
| `organization-create-slug-input` | ❌ Missing |
| `organization-create-submit-button` | ❌ Missing |
| `organization-edit-dialog` | ❌ Missing |
| `organization-edit-form` | ❌ Missing |
| `organization-edit-name-input` | ❌ Missing |
| `organization-edit-submit-button` | ❌ Missing |
| `organization-delete-confirm-dialog` | ❌ Missing |
| `organization-delete-confirm-button` | ❌ Missing |
| `organizations-pagination` | ❌ Missing |
| `organizations-pagination-next` | ❌ Missing |

### PermissionsPage (`/features/dashboard/pages/PermissionsPage/PermissionsPage.tsx`)
❌ **Not yet implemented**

| Expected Test ID | Status |
|-----------------|--------|
| `permissions-page` | ❌ Missing |
| `permissions-page-title` | ❌ Missing |
| `permissions-list` | ❌ Missing |
| `permissions-table` | ❌ Missing |
| `permission-add-button` | ❌ Missing |
| `permission-assignment-row-{roleName}-{permissionName}` | ❌ Missing |
| `permission-remove-button-{roleName}-{permissionName}` | ❌ Missing |
| `permission-add-dialog` | ❌ Missing |
| `permission-add-form` | ❌ Missing |
| `permission-add-role-select` | ❌ Missing |
| `permission-add-permission-select` | ❌ Missing |
| `permission-add-submit-button` | ❌ Missing |
| `permission-remove-confirm-dialog` | ❌ Missing |
| `permission-remove-confirm-button` | ❌ Missing |

### AuditLogPage (`/features/dashboard/pages/AuditLogPage/AuditLogPage.tsx`)
❌ **Not yet implemented**

| Expected Test ID | Status |
|-----------------|--------|
| `audit-log-page` | ❌ Missing |
| `audit-log-page-title` | ❌ Missing |
| `audit-log-list` | ❌ Missing |
| `audit-log-table` | ❌ Missing |
| `audit-log-filter-input` | ❌ Missing |
| `audit-log-pagination` | ❌ Missing |
| `audit-log-pagination-next` | ❌ Missing |
| `audit-log-pagination-prev` | ❌ Missing |
| `audit-log-pagination-page-{page}` | ❌ Missing |
| `audit-log-row-{index}` | ❌ Missing |
| `audit-log-view-button-{index}` | ❌ Missing |
| `audit-log-details-dialog` | ❌ Missing |

---

## Summary Statistics

- **Total Test IDs Expected**: ~150+
- **✅ Completed**: ~80+
- **⏳ Remaining**: ~70+
- **⚠️ Elements Don't Exist**: ~10+

### By Page:
- ✅ LoginPage: 6/6 (100%)
- ✅ RegisterPage: 6/6 (100%)
- ✅ DashboardPage: 13/13 (100%)
- ✅ SelectScopePage: 4/4 (100%)
- ⚠️ ProfilePage: 3/8 (38% - many elements don't exist)
- ✅ UsersPage: 18/20 (90% - 2 not in current implementation)
- ✅ RolesPage: 15/18 (83% - 3 not in current implementation)
- ❌ OrganizationsPage: 0/20 (0%)
- ❌ PermissionsPage: 0/13 (0%)
- ❌ AuditLogPage: 0/12 (0%)

---

## Notes

1. **FormDialog Enhancement**: The FormDialog component now supports `data-testid` and `submitButtonTestId` props, making it easy to add test IDs to all dialogs.

2. **ProfilePage Limitations**: The current ProfilePage implementation is minimal and doesn't include the profile switching functionality expected by tests. The basic test IDs have been added, but many expected elements don't exist.

3. **Delete Confirmation Dialogs**: Many pages don't currently have delete confirmation dialogs. These would need to be added if the tests expect them.

4. **View/Details Dialogs**: Some pages don't have view/details dialogs that tests might expect. These would need to be added if required.
