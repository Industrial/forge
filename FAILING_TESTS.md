# E2E Test Fixes

## Root Cause Identified and Fixed

**Issue:** All authenticated E2E tests were timing out waiting for the dashboard page to render after login.

**Root Cause:** The DashboardPage route in `App.tsx` had a `PermissionGuard` checking for `'dashboard'` permission:

```typescript
<PermissionGuard permissions={['dashboard']}>
  <DashboardPage />
</PermissionGuard>
```

However, the viewer role (and other roles) don't have a `'dashboard'` permission. The PermissionGuard was showing a loading spinner forever when permissions were empty `[]`, preventing the dashboard from rendering.

**Fix:** Removed the `PermissionGuard` from the DashboardPage route since it's just a welcome page that all authenticated users should access.

## Test Results

### ✅ Fixed Tests
- `tests/02-viewer.spec.ts:43` - "single profile redirects to dashboard" - **NOW PASSING**

### Status
- **1 test verified as fixed**
- **147 tests likely fixed** by the same root cause fix
- Running full E2E suite to verify (in progress)

## Changed Files
- `frontend/src/App.tsx` - Removed PermissionGuard from DashboardPage route (line 120-127)

---

## Original Failing Tests List

[chromium] › tests/02-viewer.spec.ts:43:5 › Viewer Role › 2.1 Authentication & Profile Selection › single profile redirects to dashboard 
[chromium] › tests/02-viewer.spec.ts:65:5 › Viewer Role › 2.1 Authentication & Profile Selection › multi-profile redirects to scope selection then dashboard 
[chromium] › tests/02-viewer.spec.ts:116:5 › Viewer Role › 2.2 Dashboard Access › should display dashboard page 
[chromium] › tests/02-viewer.spec.ts:138:5 › Viewer Role › 2.2 Dashboard Access › should show navigation links 
[chromium] › tests/02-viewer.spec.ts:175:5 › Viewer Role › 2.2 Dashboard Access › theme toggle works 
[chromium] › tests/02-viewer.spec.ts:224:5 › Viewer Role › 2.3 Users Page › should display users list 
[chromium] › tests/02-viewer.spec.ts:242:5 › Viewer Role › 2.3 Users Page › should filter users 
[chromium] › tests/02-viewer.spec.ts:257:5 › Viewer Role › 2.3 Users Page › should view user details (read-only) 
[chromium] › tests/02-viewer.spec.ts:281:5 › Viewer Role › 2.3 Users Page › should not show create button 
[chromium] › tests/02-viewer.spec.ts:319:5 › Viewer Role › 2.4 Roles Page › should display roles list 
[chromium] › tests/02-viewer.spec.ts:337:5 › Viewer Role › 2.4 Roles Page › should view role details (read-only) 
[chromium] › tests/02-viewer.spec.ts:359:5 › Viewer Role › 2.4 Roles Page › should not show create button 
[chromium] › tests/02-viewer.spec.ts:397:5 › Viewer Role › 2.5 Permissions Page › should display permissions list 
[chromium] › tests/02-viewer.spec.ts:415:5 › Viewer Role › 2.5 Permissions Page › should not show add button 
[chromium] › tests/02-viewer.spec.ts:453:5 › Viewer Role › 2.6 Audit Log Page › should display audit log list 
[chromium] › tests/02-viewer.spec.ts:471:5 › Viewer Role › 2.6 Audit Log Page › should filter audit log 
[chromium] › tests/02-viewer.spec.ts:486:5 › Viewer Role › 2.6 Audit Log Page › should paginate audit log 
[chromium] › tests/02-viewer.spec.ts:503:5 › Viewer Role › 2.6 Audit Log Page › should view audit entry details 
[chromium] › tests/02-viewer.spec.ts:522:5 › Viewer Role › 2.7 Organizations Page (Blocked) › should show 403 when accessing organizations page 
[chromium] › tests/02-viewer.spec.ts:564:5 › Viewer Role › 2.8 Profile Management › should display current profile 
[chromium] › tests/02-viewer.spec.ts:601:5 › Viewer Role › 2.9 API Token Management › POST /api/auth/tokens returns 201 Created 
[chromium] › tests/02-viewer.spec.ts:622:5 › Viewer Role › 2.9 API Token Management › use token to GET /api/auth/me returns 200 OK 
[chromium] › tests/02-viewer.spec.ts:663:5 › Viewer Role › 2.10 Generic Entity API (Read-only) › GET /api/entities/user returns 200 OK 
[chromium] › tests/02-viewer.spec.ts:670:5 › Viewer Role › 2.10 Generic Entity API (Read-only) › GET /api/entities/role returns 200 OK 
[chromium] › tests/02-viewer.spec.ts:677:5 › Viewer Role › 2.10 Generic Entity API (Read-only) › GET /api/entities/permission returns 200 OK 
[chromium] › tests/02-viewer.spec.ts:684:5 › Viewer Role › 2.10 Generic Entity API (Read-only) › GET /api/entities/audit returns 200 OK 
[chromium] › tests/02-viewer.spec.ts:691:5 › Viewer Role › 2.10 Generic Entity API (Read-only) › GET /api/entities/organization returns 403 Forbidden 
[chromium] › tests/02-viewer.spec.ts:700:5 › Viewer Role › 2.10 Generic Entity API (Read-only) › POST /api/entities/organization returns 403 Forbidden 
[chromium] › tests/02-viewer.spec.ts:728:5 › Viewer Role › 2.11 RPC API (Read-only) › POST /api/rpc entity.list returns 200 OK 
[chromium] › tests/02-viewer.spec.ts:735:5 › Viewer Role › 2.11 RPC API (Read-only) › POST /api/rpc entity.get returns 200 OK 
[chromium] › tests/02-viewer.spec.ts:753:5 › Viewer Role › 2.11 RPC API (Read-only) › POST /api/rpc entity.create returns 403 Forbidden 
[chromium] › tests/02-viewer.spec.ts:764:5 › Viewer Role › 2.12 Subscription Stream › GET /api/subscriptions/stream establishes SSE connection 
[chromium] › tests/02-viewer.spec.ts:790:5 › Viewer Role › 2.13 Admin Endpoint (Blocked) › GET /api/auth/admin returns 403 Forbidden 
[chromium] › tests/02-viewer.spec.ts:808:5 › Viewer Role › 2.14 Logout › logout redirects to login 
[chromium] › tests/02-viewer.spec.ts:838:5 › Viewer Role › 2.14 Logout › after logout protected routes redirect to login 
[chromium] › tests/03-editor.spec.ts:39:5 › Editor Role › 3.1 Authentication & Profile Selection › single profile redirects to dashboard 
[chromium] › tests/03-editor.spec.ts:78:5 › Editor Role › 3.2 Dashboard Access › should show navigation links 
[chromium] › tests/03-editor.spec.ts:134:5 › Editor Role › 3.3 Users Page (Create/Update) › should show create button 
[chromium] › tests/03-editor.spec.ts:147:5 › Editor Role › 3.3 Users Page (Create/Update) › should create user 
[chromium] › tests/03-editor.spec.ts:164:5 › Editor Role › 3.3 Users Page (Create/Update) › should update user 
[chromium] › tests/03-editor.spec.ts:186:5 › Editor Role › 3.3 Users Page (Create/Update) › should filter users 
[chromium] › tests/03-editor.spec.ts:201:5 › Editor Role › 3.3 Users Page (Create/Update) › should view user details with edit button 
[chromium] › tests/03-editor.spec.ts:241:5 › Editor Role › 3.4 Roles Page (Create/Update) › should create role 
[chromium] › tests/03-editor.spec.ts:258:5 › Editor Role › 3.4 Roles Page (Create/Update) › should update role 
[chromium] › tests/03-editor.spec.ts:277:5 › Editor Role › 3.4 Roles Page (Create/Update) › should view role details with edit button 
[chromium] › tests/03-editor.spec.ts:317:5 › Editor Role › 3.5 Permissions Page (Full CRUD) › should show add button 
[chromium] › tests/03-editor.spec.ts:330:5 › Editor Role › 3.5 Permissions Page (Full CRUD) › should add permission to role 
[chromium] › tests/03-editor.spec.ts:352:5 › Editor Role › 3.6 Audit Log Page (Blocked) › should show 403 when accessing audit log page 
[chromium] › tests/03-editor.spec.ts:376:5 › Editor Role › 3.7 Organizations Page (Blocked) › should show 403 when accessing organizations page 
[chromium] › tests/03-editor.spec.ts:402:5 › Editor Role › 3.8 Profile Management › should display current profile 
[chromium] › tests/03-editor.spec.ts:433:5 › Editor Role › 3.9 Generic Entity API › POST /api/entities/user returns 201 Created 
[chromium] › tests/03-editor.spec.ts:455:5 › Editor Role › 3.9 Generic Entity API › PATCH /api/entities/user/{id} returns 200 OK 
[chromium] › tests/03-editor.spec.ts:487:5 › Editor Role › 3.9 Generic Entity API › GET /api/entities/organization returns 403 Forbidden 
[chromium] › tests/03-editor.spec.ts:509:5 › Editor Role › 3.10 RPC API (Create/Update) › POST /api/rpc entity.create returns 200 OK 
[chromium] › tests/03-editor.spec.ts:534:5 › Editor Role › 3.10 RPC API (Create/Update) › POST /api/rpc entity.delete returns 403 Forbidden 
[chromium] › tests/03-editor.spec.ts:559:5 › Editor Role › 3.11 Subscription Stream › GET /api/subscriptions/stream establishes SSE connection 
[chromium] › tests/03-editor.spec.ts:584:5 › Editor Role › 3.12 Admin Endpoint (Blocked) › GET /api/auth/admin returns 403 Forbidden 
[chromium] › tests/03-editor.spec.ts:602:5 › Editor Role › 3.13 Logout › logout redirects to login 
[chromium] › tests/04-org-owner.spec.ts:84:5 › Org Owner Role › 4.2 Dashboard Access (Full) › should show all navigation links 
[chromium] › tests/04-org-owner.spec.ts:126:5 › Org Owner Role › 4.3 Organizations Page (Full CRUD) › should create organization 
[chromium] › tests/04-org-owner.spec.ts:146:5 › Org Owner Role › 4.3 Organizations Page (Full CRUD) › should update organization 
[chromium] › tests/04-org-owner.spec.ts:171:5 › Org Owner Role › 4.3 Organizations Page (Full CRUD) › should delete organization 
[chromium] › tests/04-org-owner.spec.ts:192:5 › Org Owner Role › 4.3 Organizations Page (Full CRUD) › should filter organizations 
[chromium] › tests/04-org-owner.spec.ts:224:5 › Org Owner Role › 4.4 Users Page (Full CRUD) › should create user 
[chromium] › tests/04-org-owner.spec.ts:241:5 › Org Owner Role › 4.4 Users Page (Full CRUD) › should update user 
[chromium] › tests/04-org-owner.spec.ts:260:5 › Org Owner Role › 4.4 Users Page (Full CRUD) › should delete user 
[chromium] › tests/04-org-owner.spec.ts:295:5 › Org Owner Role › 4.5 Roles Page (Full CRUD) › should create role 
[chromium] › tests/04-org-owner.spec.ts:312:5 › Org Owner Role › 4.5 Roles Page (Full CRUD) › should delete role 
[chromium] › tests/04-org-owner.spec.ts:347:5 › Org Owner Role › 4.6 Permissions Page (Full CRUD) › should add permission to role 
[chromium] › tests/04-org-owner.spec.ts:365:5 › Org Owner Role › 4.6 Permissions Page (Full CRUD) › should remove permission from role 
[chromium] › tests/04-org-owner.spec.ts:404:5 › Org Owner Role › 4.7 Audit Log Page › should display audit log list 
[chromium] › tests/04-org-owner.spec.ts:422:5 › Org Owner Role › 4.8 Profile Management › should display current profile 
[chromium] › tests/04-org-owner.spec.ts:453:5 › Org Owner Role › 4.9 Generic Entity API (Full CRUD) › POST /api/entities/organization returns 201 Created 
[chromium] › tests/04-org-owner.spec.ts:477:5 › Org Owner Role › 4.9 Generic Entity API (Full CRUD) › DELETE /api/entities/organization/{id} returns 200 OK 
[chromium] › tests/04-org-owner.spec.ts:508:5 › Org Owner Role › 4.10 RPC API (Full CRUD) › POST /api/rpc entity.delete returns 200 OK 
[chromium] › tests/04-org-owner.spec.ts:549:5 › Org Owner Role › 4.11 Subscription Stream › GET /api/subscriptions/stream establishes SSE connection 
[chromium] › tests/04-org-owner.spec.ts:574:5 › Org Owner Role › 4.12 Admin Endpoint (Blocked) › GET /api/auth/admin returns 403 Forbidden 
[chromium] › tests/04-org-owner.spec.ts:592:5 › Org Owner Role › 4.13 Scope Switching › should switch profile and see updated data 
[chromium] › tests/04-org-owner.spec.ts:630:5 › Org Owner Role › 4.14 Logout › logout redirects to login 
[chromium] › tests/05-org-admin.spec.ts:57:5 › Org Admin Role › 5.2 Dashboard Access (Full) › should show all navigation links 
[chromium] › tests/05-org-admin.spec.ts:95:5 › Org Admin Role › 5.3 Organizations Page (Full CRUD) › should create organization 
[chromium] › tests/05-org-admin.spec.ts:129:5 › Org Admin Role › 5.4 Users Page (Full CRUD) › should create user 
[chromium] › tests/05-org-admin.spec.ts:160:5 › Org Admin Role › 5.5 Roles Page (Full CRUD) › should create role 
[chromium] › tests/05-org-admin.spec.ts:191:5 › Org Admin Role › 5.6 Permissions Page (Full CRUD) › should add permission to role 
[chromium] › tests/05-org-admin.spec.ts:224:5 › Org Admin Role › 5.7 Audit Log Page › should display audit log list 
[chromium] › tests/05-org-admin.spec.ts:255:5 › Org Admin Role › 5.8 Profile Management › should display current profile 
[chromium] › tests/05-org-admin.spec.ts:286:5 › Org Admin Role › 5.9 Generic Entity API (Full CRUD) › POST /api/entities/organization returns 201 Created 
[chromium] › tests/05-org-admin.spec.ts:312:5 › Org Admin Role › 5.10 RPC API (Full CRUD) › POST /api/rpc entity.delete returns 200 OK 
[chromium] › tests/05-org-admin.spec.ts:353:5 › Org Admin Role › 5.11 Subscription Stream › GET /api/subscriptions/stream establishes SSE connection 
[chromium] › tests/05-org-admin.spec.ts:378:5 › Org Admin Role › 5.12 Admin Endpoint (Blocked) › GET /api/auth/admin returns 403 Forbidden 
[chromium] › tests/05-org-admin.spec.ts:396:5 › Org Admin Role › 5.13 Scope Switching › should switch profile and see updated data 
[chromium] › tests/05-org-admin.spec.ts:431:5 › Org Admin Role › 5.14 Logout › logout redirects to login 
[chromium] › tests/06-app-admin.spec.ts:110:5 › App Admin Role › 6.2 Dashboard Access (Full - Global Scope) › can access all organizations (global-scope) 
[chromium] › tests/06-app-admin.spec.ts:126:5 › App Admin Role › 6.2 Dashboard Access (Full - Global Scope) › can see users from all orgs (global-scope) 
[chromium] › tests/06-app-admin.spec.ts:144:5 › App Admin Role › 6.3 Organizations Page (Full CRUD - Global Scope) › should create organization 
[chromium] › tests/06-app-admin.spec.ts:178:5 › App Admin Role › 6.4 Users Page (Full CRUD - Global Scope) › should create user 
[chromium] › tests/06-app-admin.spec.ts:209:5 › App Admin Role › 6.5 Roles Page (Full CRUD - Global Scope) › should create role 
[chromium] › tests/06-app-admin.spec.ts:240:5 › App Admin Role › 6.6 Permissions Page (Full CRUD - Global Scope) › should add permission to role 
[chromium] › tests/06-app-admin.spec.ts:273:5 › App Admin Role › 6.7 Audit Log Page (Global Scope) › should display audit log list 
[chromium] › tests/06-app-admin.spec.ts:304:5 › App Admin Role › 6.8 Profile Management › should display current profile 
[chromium] › tests/06-app-admin.spec.ts:335:5 › App Admin Role › 6.9 Admin Endpoint (Access Granted) › GET /api/auth/admin returns 200 OK 
[chromium] › tests/06-app-admin.spec.ts:356:5 › App Admin Role › 6.10 Generic Entity API (Full CRUD - Global Scope) › GET /api/entities/organization returns 200 OK (all orgs) 
[chromium] › tests/06-app-admin.spec.ts:376:5 › App Admin Role › 6.10 Generic Entity API (Full CRUD - Global Scope) › POST /api/entities/organization returns 201 Created 
[chromium] › tests/06-app-admin.spec.ts:400:5 › App Admin Role › 6.10 Generic Entity API (Full CRUD - Global Scope) › GET /api/entities/user returns 200 OK (all users, all orgs) 
[chromium] › tests/06-app-admin.spec.ts:422:5 › App Admin Role › 6.11 RPC API (Full CRUD - Global Scope) › POST /api/rpc entity.list returns 200 OK (all entities, all orgs) 
[chromium] › tests/06-app-admin.spec.ts:444:5 › App Admin Role › 6.12 Subscription Stream (Global Scope) › GET /api/subscriptions/stream establishes SSE connection 
[chromium] › tests/06-app-admin.spec.ts:469:5 › App Admin Role › 6.13 Global Role Management › should manage global roles 
[chromium] › tests/06-app-admin.spec.ts:497:5 › App Admin Role › 6.14 Scope Switching (Global Admin) › should switch profile and see updated data 
[chromium] › tests/06-app-admin.spec.ts:532:5 › App Admin Role › 6.15 API Token Management › POST /api/auth/tokens returns 201 Created 
[chromium] › tests/06-app-admin.spec.ts:550:5 › App Admin Role › 6.15 API Token Management › use token to GET /api/auth/admin returns 200 OK 
[chromium] › tests/06-app-admin.spec.ts:580:5 › App Admin Role › 6.16 Logout › logout redirects to login 
[chromium] › tests/07-security.spec.ts:24:5 › Cross-Role Scenarios › 7.1 Multi-Organization User Flow › should switch between organizations 
[chromium] › tests/07-security.spec.ts:99:7 › Cross-Role Scenarios › 7.2 Permission Escalation Prevention › Editor permissions › Editor cannot access org-admin-only features 
[chromium] › tests/07-security.spec.ts:123:7 › Cross-Role Scenarios › 7.2 Permission Escalation Prevention › Org Admin permissions › Org Admin cannot access global-admin-only features 
[chromium] › tests/07-security.spec.ts:145:7 › Cross-Role Scenarios › 7.2 Permission Escalation Prevention › App Admin permissions › App Admin can access global-admin features 
[chromium] › tests/07-security.spec.ts:166:5 › Cross-Role Scenarios › 7.3 Concurrent Operations › subscription stream receives invalidation events 
[chromium] › tests/07-security.spec.ts:226:5 › Cross-Role Scenarios › 7.4 Error Handling › invalid API request returns 400 Bad Request 
[chromium] › tests/07-security.spec.ts:242:7 › Cross-Role Scenarios › 7.4 Error Handling › with Viewer auth › forbidden request returns 403 Forbidden 
[chromium] › tests/07-security.spec.ts:263:7 › Cross-Role Scenarios › 7.4 Error Handling › with Viewer auth › not found returns 404 Not Found 
[chromium] › tests/07-security.spec.ts:284:5 › Cross-Role Scenarios › 7.5 Session Management › logout invalidates token 
[chromium] › tests/07-security.spec.ts:314:5 › Cross-Role Scenarios › 7.6 Data Consistency › created entity appears in list 
[chromium] › tests/08-edge-cases.spec.ts:33:5 › Edge Cases & Boundary Conditions › 8.1 Empty States › organizations page shows empty state 
[chromium] › tests/08-edge-cases.spec.ts:69:5 › Edge Cases & Boundary Conditions › 8.1 Empty States › users page shows empty state 
[chromium] › tests/08-edge-cases.spec.ts:91:5 › Edge Cases & Boundary Conditions › 8.1 Empty States › roles page shows empty state 
[chromium] › tests/08-edge-cases.spec.ts:113:5 › Edge Cases & Boundary Conditions › 8.1 Empty States › audit log page shows empty state 
[chromium] › tests/08-edge-cases.spec.ts:137:5 › Edge Cases & Boundary Conditions › 8.2 Large Datasets › large user list pagination works correctly 
[chromium] › tests/08-edge-cases.spec.ts:181:5 › Edge Cases & Boundary Conditions › 8.2 Large Datasets › large organization list pagination works correctly 
[chromium] › tests/08-edge-cases.spec.ts:217:5 › Edge Cases & Boundary Conditions › 8.2 Large Datasets › large audit log pagination works correctly 
[chromium] › tests/08-edge-cases.spec.ts:249:5 › Edge Cases & Boundary Conditions › 8.2 Large Datasets › cursor pagination works correctly 
[chromium] › tests/08-edge-cases.spec.ts:281:5 › Edge Cases & Boundary Conditions › 8.3 Invalid Inputs › invalid email format shows validation error 
[chromium] › tests/08-edge-cases.spec.ts:330:5 › Edge Cases & Boundary Conditions › 8.3 Invalid Inputs › short password shows validation error 
[chromium] › tests/08-edge-cases.spec.ts:368:5 › Edge Cases & Boundary Conditions › 8.3 Invalid Inputs › invalid UUID returns 400 Bad Request 
[chromium] › tests/08-edge-cases.spec.ts:386:5 › Edge Cases & Boundary Conditions › 8.3 Invalid Inputs › invalid filter JSON returns 400 Bad Request 
[chromium] › tests/08-edge-cases.spec.ts:404:5 › Edge Cases & Boundary Conditions › 8.3 Invalid Inputs › invalid sort field returns 400 Bad Request 
[chromium] › tests/08-edge-cases.spec.ts:424:5 › Edge Cases & Boundary Conditions › 8.4 Network Conditions › slow network shows loading states 
[chromium] › tests/08-edge-cases.spec.ts:471:5 › Edge Cases & Boundary Conditions › 8.4 Network Conditions › network error shows error message 
[chromium] › tests/08-edge-cases.spec.ts:486:5 › Edge Cases & Boundary Conditions › 8.4 Network Conditions › timeout shows error message 
[chromium] › tests/08-edge-cases.spec.ts:504:5 › Edge Cases & Boundary Conditions › 8.5 Browser Compatibility › mobile viewport responsive layout works 
[chromium] › tests/08-edge-cases.spec.ts:543:5 › Edge Cases & Boundary Conditions › 8.5 Browser Compatibility › dark/light theme toggle works 
[chromium] › tests/08-edge-cases.spec.ts:584:5 › Edge Cases & Boundary Conditions › 8.5 Browser Compatibility › browser back/forward navigation works 
[chromium] › tests/09-security.spec.ts:47:5 › Security Scenarios › 9.1 Authentication Security › SQL injection in login is sanitized/rejected 
[chromium] › tests/09-security.spec.ts:75:5 › Security Scenarios › 9.1 Authentication Security › XSS in login is sanitized/rejected 
[chromium] › tests/09-security.spec.ts:109:7 › Security Scenarios › 9.2 Authorization Security › Viewer auth › Viewer cannot create entity 
[chromium] › tests/09-security.spec.ts:169:7 › Security Scenarios › 9.2 Authorization Security › Org Admin auth › Org Admin cannot access other org data 
[chromium] › tests/09-security.spec.ts:195:7 › Security Scenarios › 9.2 Authorization Security › App Admin auth › App Admin can access any org data 
[chromium] › tests/09-security.spec.ts:220:5 › Security Scenarios › 9.3 CSRF Protection › form submission includes CSRF token 
[chromium] › tests/09-security.spec.ts:258:5 › Security Scenarios › 9.3 CSRF Protection › missing CSRF token request is rejected 
[chromium] › tests/09-security.spec.ts:319:5 › Security Scenarios › 9.5 Security Headers › security headers are present 
