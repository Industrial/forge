#!/usr/bin/env python3
"""Create bd tasks for all failing e2e tests."""
import re
import subprocess
import sys

# Test failures list (from user message)
TEST_LINES = """[chromium] › tests/01-guest.spec.ts:162:7 › Guest/Unauthenticated User › 1.2 Registration Flow › Invalid Input › invalid email shows error 
[chromium] › tests/01-guest.spec.ts:187:7 › Guest/Unauthenticated User › 1.2 Registration Flow › Invalid Input › short password shows error 
[chromium] › tests/01-guest.spec.ts:237:7 › Guest/Unauthenticated User › 1.2 Registration Flow › Valid Registration › can login with new credentials 
[chromium] › tests/01-guest.spec.ts:316:7 › Guest/Unauthenticated User › 1.3 Login Flow › Valid Credentials › single profile redirects to dashboard 
[chromium] › tests/01-guest.spec.ts:335:7 › Guest/Unauthenticated User › 1.3 Login Flow › Valid Credentials › multi-profile redirects to scope selection 
[chromium] › tests/02-viewer.spec.ts:46:5 › Viewer Role › 2.1 Authentication & Profile Selection › single profile redirects to dashboard 
[chromium] › tests/02-viewer.spec.ts:68:5 › Viewer Role › 2.1 Authentication & Profile Selection › multi-profile redirects to scope selection then dashboard 
[chromium] › tests/02-viewer.spec.ts:108:5 › Viewer Role › 2.2 Dashboard Access › should display dashboard page 
[chromium] › tests/02-viewer.spec.ts:130:5 › Viewer Role › 2.2 Dashboard Access › should show navigation links 
[chromium] › tests/02-viewer.spec.ts:152:5 › Viewer Role › 2.2 Dashboard Access › should not show organizations link 
[chromium] › tests/02-viewer.spec.ts:167:5 › Viewer Role › 2.2 Dashboard Access › theme toggle works 
[chromium] › tests/02-viewer.spec.ts:205:5 › Viewer Role › 2.3 Users Page › should display users list 
[chromium] › tests/02-viewer.spec.ts:223:5 › Viewer Role › 2.3 Users Page › should filter users 
[chromium] › tests/02-viewer.spec.ts:238:5 › Viewer Role › 2.3 Users Page › should view user details (read-only) 
[chromium] › tests/02-viewer.spec.ts:262:5 › Viewer Role › 2.3 Users Page › should not show create button 
[chromium] › tests/02-viewer.spec.ts:289:5 › Viewer Role › 2.4 Roles Page › should display roles list 
[chromium] › tests/02-viewer.spec.ts:307:5 › Viewer Role › 2.4 Roles Page › should view role details (read-only) 
[chromium] › tests/02-viewer.spec.ts:329:5 › Viewer Role › 2.4 Roles Page › should not show create button 
[chromium] › tests/02-viewer.spec.ts:356:5 › Viewer Role › 2.5 Permissions Page › should display permissions list 
[chromium] › tests/02-viewer.spec.ts:374:5 › Viewer Role › 2.5 Permissions Page › should not show add button 
[chromium] › tests/02-viewer.spec.ts:401:5 › Viewer Role › 2.6 Audit Log Page › should display audit log list 
[chromium] › tests/02-viewer.spec.ts:419:5 › Viewer Role › 2.6 Audit Log Page › should filter audit log 
[chromium] › tests/02-viewer.spec.ts:434:5 › Viewer Role › 2.6 Audit Log Page › should paginate audit log 
[chromium] › tests/02-viewer.spec.ts:451:5 › Viewer Role › 2.6 Audit Log Page › should view audit entry details 
[chromium] › tests/02-viewer.spec.ts:470:5 › Viewer Role › 2.7 Organizations Page (Blocked) › should show 403 when accessing organizations page 
[chromium] › tests/02-viewer.spec.ts:490:5 › Viewer Role › 2.8 Profile Management › should display current profile 
[chromium] › tests/02-viewer.spec.ts:512:5 › Viewer Role › 2.8 Profile Management › should view profile details (read-only) 
[chromium] › tests/02-viewer.spec.ts:527:5 › Viewer Role › 2.9 API Token Management › POST /api/auth/tokens returns 201 Created 
[chromium] › tests/02-viewer.spec.ts:535:5 › Viewer Role › 2.9 API Token Management › use token to GET /api/auth/me returns 200 OK 
[chromium] › tests/02-viewer.spec.ts:553:5 › Viewer Role › 2.10 Generic Entity API (Read-only) › GET /api/entities/user returns 200 OK 
[chromium] › tests/02-viewer.spec.ts:558:5 › Viewer Role › 2.10 Generic Entity API (Read-only) › GET /api/entities/role returns 200 OK 
[chromium] › tests/02-viewer.spec.ts:563:5 › Viewer Role › 2.10 Generic Entity API (Read-only) › GET /api/entities/permission returns 200 OK 
[chromium] › tests/02-viewer.spec.ts:570:5 › Viewer Role › 2.10 Generic Entity API (Read-only) › GET /api/entities/audit returns 200 OK 
[chromium] › tests/02-viewer.spec.ts:575:5 › Viewer Role › 2.10 Generic Entity API (Read-only) › GET /api/entities/organization returns 403 Forbidden 
[chromium] › tests/02-viewer.spec.ts:584:5 › Viewer Role › 2.10 Generic Entity API (Read-only) › POST /api/entities/organization returns 403 Forbidden 
[chromium] › tests/02-viewer.spec.ts:598:5 › Viewer Role › 2.11 RPC API (Read-only) › POST /api/rpc entity.list returns 200 OK 
[chromium] › tests/02-viewer.spec.ts:605:5 › Viewer Role › 2.11 RPC API (Read-only) › POST /api/rpc entity.get returns 200 OK 
[chromium] › tests/02-viewer.spec.ts:623:5 › Viewer Role › 2.11 RPC API (Read-only) › POST /api/rpc entity.create returns 403 Forbidden 
[chromium] › tests/02-viewer.spec.ts:634:5 › Viewer Role › 2.12 Subscription Stream › GET /api/subscriptions/stream establishes SSE connection 
[chromium] › tests/02-viewer.spec.ts:649:5 › Viewer Role › 2.13 Admin Endpoint (Blocked) › GET /api/auth/admin returns 403 Forbidden 
[chromium] › tests/02-viewer.spec.ts:656:5 › Viewer Role › 2.14 Logout › logout redirects to login 
[chromium] › tests/02-viewer.spec.ts:675:5 › Viewer Role › 2.14 Logout › after logout protected routes redirect to login 
[chromium] › tests/03-editor.spec.ts:41:5 › Editor Role › 3.1 Authentication & Profile Selection › single profile redirects to dashboard 
[chromium] › tests/03-editor.spec.ts:69:5 › Editor Role › 3.2 Dashboard Access › should show navigation links 
[chromium] › tests/03-editor.spec.ts:89:5 › Editor Role › 3.2 Dashboard Access › should not show organizations or audit log links 
[chromium] › tests/03-editor.spec.ts:114:5 › Editor Role › 3.3 Users Page (Create/Update) › should show create button 
[chromium] › tests/03-editor.spec.ts:127:5 › Editor Role › 3.3 Users Page (Create/Update) › should create user 
[chromium] › tests/03-editor.spec.ts:144:5 › Editor Role › 3.3 Users Page (Create/Update) › should update user 
[chromium] › tests/03-editor.spec.ts:166:5 › Editor Role › 3.3 Users Page (Create/Update) › should filter users 
[chromium] › tests/03-editor.spec.ts:181:5 › Editor Role › 3.3 Users Page (Create/Update) › should view user details with edit button 
[chromium] › tests/03-editor.spec.ts:210:5 › Editor Role › 3.4 Roles Page (Create/Update) › should create role 
[chromium] › tests/03-editor.spec.ts:227:5 › Editor Role › 3.4 Roles Page (Create/Update) › should update role 
[chromium] › tests/03-editor.spec.ts:246:5 › Editor Role › 3.4 Roles Page (Create/Update) › should view role details with edit button 
[chromium] › tests/03-editor.spec.ts:275:5 › Editor Role › 3.5 Permissions Page (Full CRUD) › should show add button 
[chromium] › tests/03-editor.spec.ts:288:5 › Editor Role › 3.5 Permissions Page (Full CRUD) › should add permission to role 
[chromium] › tests/03-editor.spec.ts:310:5 › Editor Role › 3.6 Audit Log Page (Blocked) › should show 403 when accessing audit log page 
[chromium] › tests/03-editor.spec.ts:323:5 › Editor Role › 3.7 Organizations Page (Blocked) › should show 403 when accessing organizations page 
[chromium] › tests/03-editor.spec.ts:338:5 › Editor Role › 3.8 Profile Management › should display current profile 
[chromium] › tests/03-editor.spec.ts:358:5 › Editor Role › 3.9 Generic Entity API › POST /api/entities/user returns 201 Created 
[chromium] › tests/03-editor.spec.ts:366:5 › Editor Role › 3.9 Generic Entity API › PATCH /api/entities/user/{id} returns 200 OK 
[chromium] › tests/03-editor.spec.ts:389:5 › Editor Role › 3.9 Generic Entity API › GET /api/entities/organization returns 403 Forbidden 
[chromium] › tests/03-editor.spec.ts:400:5 › Editor Role › 3.10 RPC API (Create/Update) › POST /api/rpc entity.create returns 200 OK 
[chromium] › tests/03-editor.spec.ts:414:5 › Editor Role › 3.10 RPC API (Create/Update) › POST /api/rpc entity.delete returns 403 Forbidden 
[chromium] › tests/03-editor.spec.ts:428:5 › Editor Role › 3.11 Subscription Stream › GET /api/subscriptions/stream establishes SSE connection 
[chromium] › tests/03-editor.spec.ts:442:5 › Editor Role › 3.12 Admin Endpoint (Blocked) › GET /api/auth/admin returns 403 Forbidden 
[chromium] › tests/03-editor.spec.ts:449:5 › Editor Role › 3.13 Logout › logout redirects to login 
[chromium] › tests/04-org-owner.spec.ts:47:5 › Org Owner Role › 4.1 Authentication & Profile Selection › single profile redirects to dashboard 
[chromium] › tests/04-org-owner.spec.ts:75:5 › Org Owner Role › 4.2 Dashboard Access (Full) › should show all navigation links 
[chromium] › tests/04-org-owner.spec.ts:106:5 › Org Owner Role › 4.3 Organizations Page (Full CRUD) › should create organization 
[chromium] › tests/04-org-owner.spec.ts:126:5 › Org Owner Role › 4.3 Organizations Page (Full CRUD) › should update organization 
[chromium] › tests/04-org-owner.spec.ts:151:5 › Org Owner Role › 4.3 Organizations Page (Full CRUD) › should delete organization 
[chromium] › tests/04-org-owner.spec.ts:172:5 › Org Owner Role › 4.3 Organizations Page (Full CRUD) › should filter organizations 
[chromium] › tests/04-org-owner.spec.ts:193:5 › Org Owner Role › 4.4 Users Page (Full CRUD) › should create user 
[chromium] › tests/04-org-owner.spec.ts:210:5 › Org Owner Role › 4.4 Users Page (Full CRUD) › should update user 
[chromium] › tests/04-org-owner.spec.ts:229:5 › Org Owner Role › 4.4 Users Page (Full CRUD) › should delete user 
[chromium] › tests/04-org-owner.spec.ts:253:5 › Org Owner Role › 4.5 Roles Page (Full CRUD) › should create role 
[chromium] › tests/04-org-owner.spec.ts:270:5 › Org Owner Role › 4.5 Roles Page (Full CRUD) › should delete role 
[chromium] › tests/04-org-owner.spec.ts:294:5 › Org Owner Role › 4.6 Permissions Page (Full CRUD) › should add permission to role 
[chromium] › tests/04-org-owner.spec.ts:312:5 › Org Owner Role › 4.6 Permissions Page (Full CRUD) › should remove permission from role 
[chromium] › tests/04-org-owner.spec.ts:340:5 › Org Owner Role › 4.7 Audit Log Page › should display audit log list 
[chromium] › tests/04-org-owner.spec.ts:358:5 › Org Owner Role › 4.8 Profile Management › should display current profile 
[chromium] › tests/04-org-owner.spec.ts:378:5 › Org Owner Role › 4.9 Generic Entity API (Full CRUD) › POST /api/entities/organization returns 201 Created 
[chromium] › tests/04-org-owner.spec.ts:391:5 › Org Owner Role › 4.9 Generic Entity API (Full CRUD) › DELETE /api/entities/organization/{id} returns 200 OK 
[chromium] › tests/04-org-owner.spec.ts:411:5 › Org Owner Role › 4.10 RPC API (Full CRUD) › POST /api/rpc entity.delete returns 200 OK 
[chromium] › tests/04-org-owner.spec.ts:435:5 › Org Owner Role › 4.11 Subscription Stream › GET /api/subscriptions/stream establishes SSE connection 
[chromium] › tests/04-org-owner.spec.ts:449:5 › Org Owner Role › 4.12 Admin Endpoint (Blocked) › GET /api/auth/admin returns 403 Forbidden 
[chromium] › tests/04-org-owner.spec.ts:456:5 › Org Owner Role › 4.13 Scope Switching › should switch profile and see updated data 
[chromium] › tests/04-org-owner.spec.ts:483:5 › Org Owner Role › 4.14 Logout › logout redirects to login 
[chromium] › tests/05-org-admin.spec.ts:35:5 › Org Admin Role › 5.1 Authentication & Profile Selection › single profile redirects to dashboard 
[chromium] › tests/05-org-admin.spec.ts:59:5 › Org Admin Role › 5.2 Dashboard Access (Full) › should show all navigation links 
[chromium] › tests/05-org-admin.spec.ts:86:5 › Org Admin Role › 5.3 Organizations Page (Full CRUD) › should create organization 
[chromium] › tests/05-org-admin.spec.ts:109:5 › Org Admin Role › 5.4 Users Page (Full CRUD) › should create user 
[chromium] › tests/05-org-admin.spec.ts:129:5 › Org Admin Role › 5.5 Roles Page (Full CRUD) › should create role 
[chromium] › tests/05-org-admin.spec.ts:149:5 › Org Admin Role › 5.6 Permissions Page (Full CRUD) › should add permission to role 
[chromium] › tests/05-org-admin.spec.ts:171:5 › Org Admin Role › 5.7 Audit Log Page › should display audit log list 
[chromium] › tests/05-org-admin.spec.ts:191:5 › Org Admin Role › 5.8 Profile Management › should display current profile 
[chromium] › tests/05-org-admin.spec.ts:211:5 › Org Admin Role › 5.9 Generic Entity API (Full CRUD) › POST /api/entities/organization returns 201 Created 
[chromium] › tests/05-org-admin.spec.ts:226:5 › Org Admin Role › 5.10 RPC API (Full CRUD) › POST /api/rpc entity.delete returns 200 OK 
[chromium] › tests/05-org-admin.spec.ts:250:5 › Org Admin Role › 5.11 Subscription Stream › GET /api/subscriptions/stream establishes SSE connection 
[chromium] › tests/05-org-admin.spec.ts:264:5 › Org Admin Role › 5.12 Admin Endpoint (Blocked) › GET /api/auth/admin returns 403 Forbidden 
[chromium] › tests/05-org-admin.spec.ts:271:5 › Org Admin Role › 5.13 Scope Switching › should switch profile and see updated data 
[chromium] › tests/05-org-admin.spec.ts:295:5 › Org Admin Role › 5.14 Logout › logout redirects to login 
[chromium] › tests/06-app-admin.spec.ts:50:5 › App Admin Role › 6.1 Authentication & Profile Selection › single profile redirects to dashboard 
[chromium] › tests/06-app-admin.spec.ts:78:5 › App Admin Role › 6.2 Dashboard Access (Full - Global Scope) › should show all navigation links 
[chromium] › tests/06-app-admin.spec.ts:101:5 › App Admin Role › 6.2 Dashboard Access (Full - Global Scope) › can access all organizations (global-scope) 
[chromium] › tests/06-app-admin.spec.ts:117:5 › App Admin Role › 6.2 Dashboard Access (Full - Global Scope) › can see users from all orgs (global-scope) 
[chromium] › tests/06-app-admin.spec.ts:135:5 › App Admin Role › 6.3 Organizations Page (Full CRUD - Global Scope) › should create organization 
[chromium] › tests/06-app-admin.spec.ts:158:5 › App Admin Role › 6.4 Users Page (Full CRUD - Global Scope) › should create user 
[chromium] › tests/06-app-admin.spec.ts:178:5 › App Admin Role › 6.5 Roles Page (Full CRUD - Global Scope) › should create role 
[chromium] › tests/06-app-admin.spec.ts:198:5 › App Admin Role › 6.6 Permissions Page (Full CRUD - Global Scope) › should add permission to role 
[chromium] › tests/06-app-admin.spec.ts:220:5 › App Admin Role › 6.7 Audit Log Page (Global Scope) › should display audit log list 
[chromium] › tests/06-app-admin.spec.ts:240:5 › App Admin Role › 6.8 Profile Management › should display current profile 
[chromium] › tests/06-app-admin.spec.ts:260:5 › App Admin Role › 6.9 Admin Endpoint (Access Granted) › GET /api/auth/admin returns 200 OK 
[chromium] › tests/06-app-admin.spec.ts:270:5 › App Admin Role › 6.10 Generic Entity API (Full CRUD - Global Scope) › GET /api/entities/organization returns 200 OK (all orgs) 
[chromium] › tests/06-app-admin.spec.ts:279:5 › App Admin Role › 6.10 Generic Entity API (Full CRUD - Global Scope) › POST /api/entities/organization returns 201 Created 
[chromium] › tests/06-app-admin.spec.ts:292:5 › App Admin Role › 6.10 Generic Entity API (Full CRUD - Global Scope) › GET /api/entities/user returns 200 OK (all users, all orgs) 
[chromium] › tests/06-app-admin.spec.ts:301:5 › App Admin Role › 6.11 RPC API (Full CRUD - Global Scope) › POST /api/rpc entity.list returns 200 OK (all entities, all orgs) 
[chromium] › tests/06-app-admin.spec.ts:312:5 › App Admin Role › 6.12 Subscription Stream (Global Scope) › GET /api/subscriptions/stream establishes SSE connection 
[chromium] › tests/06-app-admin.spec.ts:326:5 › App Admin Role › 6.13 Global Role Management › should manage global roles 
[chromium] › tests/06-app-admin.spec.ts:343:5 › App Admin Role › 6.14 Scope Switching (Global Admin) › should switch profile and see updated data 
[chromium] › tests/06-app-admin.spec.ts:367:5 › App Admin Role › 6.15 API Token Management › POST /api/auth/tokens returns 201 Created 
[chromium] › tests/06-app-admin.spec.ts:372:5 › App Admin Role › 6.15 API Token Management › use token to GET /api/auth/admin returns 200 OK 
[chromium] › tests/06-app-admin.spec.ts:391:5 › App Admin Role › 6.16 Logout › logout redirects to login 
[chromium] › tests/07-security.spec.ts:24:5 › Cross-Role Scenarios › 7.1 Multi-Organization User Flow › should switch between organizations 
[chromium] › tests/07-security.spec.ts:73:7 › Cross-Role Scenarios › 7.2 Permission Escalation Prevention › Viewer permissions › Viewer cannot access editor-only features 
[chromium] › tests/07-security.spec.ts:92:7 › Cross-Role Scenarios › 7.2 Permission Escalation Prevention › Editor permissions › Editor cannot access org-admin-only features 
[chromium] › tests/07-security.spec.ts:107:7 › Cross-Role Scenarios › 7.2 Permission Escalation Prevention › Org Admin permissions › Org Admin cannot access global-admin-only features 
[chromium] › tests/07-security.spec.ts:118:7 › Cross-Role Scenarios › 7.2 Permission Escalation Prevention › App Admin permissions › App Admin can access global-admin features 
[chromium] › tests/07-security.spec.ts:130:5 › Cross-Role Scenarios › 7.3 Concurrent Operations › subscription stream receives invalidation events 
[chromium] › tests/07-security.spec.ts:168:5 › Cross-Role Scenarios › 7.4 Error Handling › invalid API request returns 400 Bad Request 
[chromium] › tests/07-security.spec.ts:186:7 › Cross-Role Scenarios › 7.4 Error Handling › with Viewer auth › forbidden request returns 403 Forbidden 
[chromium] › tests/07-security.spec.ts:196:7 › Cross-Role Scenarios › 7.4 Error Handling › with Viewer auth › not found returns 404 Not Found 
[chromium] › tests/07-security.spec.ts:208:5 › Cross-Role Scenarios › 7.5 Session Management › logout invalidates token 
[chromium] › tests/07-security.spec.ts:229:5 › Cross-Role Scenarios › 7.6 Data Consistency › created entity appears in list 
[chromium] › tests/08-edge-cases.spec.ts:33:5 › Edge Cases & Boundary Conditions › 8.1 Empty States › organizations page shows empty state 
[chromium] › tests/08-edge-cases.spec.ts:58:5 › Edge Cases & Boundary Conditions › 8.1 Empty States › users page shows empty state 
[chromium] › tests/08-edge-cases.spec.ts:80:5 › Edge Cases & Boundary Conditions › 8.1 Empty States › roles page shows empty state 
[chromium] › tests/08-edge-cases.spec.ts:102:5 › Edge Cases & Boundary Conditions › 8.1 Empty States › audit log page shows empty state 
[chromium] › tests/08-edge-cases.spec.ts:128:5 › Edge Cases & Boundary Conditions › 8.2 Large Datasets › large user list pagination works correctly 
[chromium] › tests/08-edge-cases.spec.ts:161:5 › Edge Cases & Boundary Conditions › 8.2 Large Datasets › large organization list pagination works correctly 
[chromium] › tests/08-edge-cases.spec.ts:197:5 › Edge Cases & Boundary Conditions › 8.2 Large Datasets › large audit log pagination works correctly 
[chromium] › tests/08-edge-cases.spec.ts:229:5 › Edge Cases & Boundary Conditions › 8.2 Large Datasets › cursor pagination works correctly 
[chromium] › tests/08-edge-cases.spec.ts:252:5 › Edge Cases & Boundary Conditions › 8.3 Invalid Inputs › invalid email format shows validation error 
[chromium] › tests/08-edge-cases.spec.ts:290:5 › Edge Cases & Boundary Conditions › 8.3 Invalid Inputs › short password shows validation error 
[chromium] › tests/08-edge-cases.spec.ts:328:5 › Edge Cases & Boundary Conditions › 8.3 Invalid Inputs › invalid UUID returns 400 Bad Request 
[chromium] › tests/08-edge-cases.spec.ts:335:5 › Edge Cases & Boundary Conditions › 8.3 Invalid Inputs › invalid filter JSON returns 400 Bad Request 
[chromium] › tests/08-edge-cases.spec.ts:342:5 › Edge Cases & Boundary Conditions › 8.3 Invalid Inputs › invalid sort field returns 400 Bad Request 
[chromium] › tests/08-edge-cases.spec.ts:353:5 › Edge Cases & Boundary Conditions › 8.4 Network Conditions › slow network shows loading states 
[chromium] › tests/08-edge-cases.spec.ts:389:5 › Edge Cases & Boundary Conditions › 8.4 Network Conditions › network error shows error message 
[chromium] › tests/08-edge-cases.spec.ts:404:5 › Edge Cases & Boundary Conditions › 8.4 Network Conditions › timeout shows error message 
[chromium] › tests/08-edge-cases.spec.ts:424:5 › Edge Cases & Boundary Conditions › 8.5 Browser Compatibility › mobile viewport responsive layout works 
[chromium] › tests/08-edge-cases.spec.ts:452:5 › Edge Cases & Boundary Conditions › 8.5 Browser Compatibility › dark/light theme toggle works 
[chromium] › tests/08-edge-cases.spec.ts:493:5 › Edge Cases & Boundary Conditions › 8.5 Browser Compatibility › browser back/forward navigation works 
[chromium] › tests/09-security.spec.ts:46:5 › Security Scenarios › 9.1 Authentication Security › SQL injection in login is sanitized/rejected 
[chromium] › tests/09-security.spec.ts:74:5 › Security Scenarios › 9.1 Authentication Security › XSS in login is sanitized/rejected 
[chromium] › tests/09-security.spec.ts:110:7 › Security Scenarios › 9.2 Authorization Security › Viewer auth › Viewer cannot create entity 
[chromium] › tests/09-security.spec.ts:124:7 › Security Scenarios › 9.2 Authorization Security › Editor auth › Editor cannot delete entity if no delete permission 
[chromium] › tests/09-security.spec.ts:152:7 › Security Scenarios › 9.2 Authorization Security › Org Admin auth › Org Admin cannot access other org data 
[chromium] › tests/09-security.spec.ts:169:7 › Security Scenarios › 9.2 Authorization Security › App Admin auth › App Admin can access any org data 
[chromium] › tests/09-security.spec.ts:185:5 › Security Scenarios › 9.3 CSRF Protection › form submission includes CSRF token 
[chromium] › tests/09-security.spec.ts:212:5 › Security Scenarios › 9.3 CSRF Protection › missing CSRF token request is rejected 
[chromium] › tests/09-security.spec.ts:259:5 › Security Scenarios › 9.5 Security Headers › security headers are present"""


def parse_test_line(line):
    """Parse a test failure line into components."""
    if not line.strip():
        return None
    
    # Parse: [chromium] › tests/01-guest.spec.ts:162:7 › Guest/Unauthenticated User › ...
    parts = line.split(' › ', 2)
    if len(parts) < 3:
        return None
    
    test_file_line = parts[1]  # tests/01-guest.spec.ts:162:7
    test_path = parts[2].strip()  # Guest/Unauthenticated User › ...
    
    # Extract file and line
    file_line_match = re.match(r'(tests/[^:]+):(\d+)', test_file_line)
    if not file_line_match:
        return None
    
    test_file = file_line_match.group(1)
    test_line = file_line_match.group(2)
    
    # Get short title (last part of test path)
    title_parts = test_path.split(' › ')
    short_title = title_parts[-1] if title_parts else test_path
    
    return {
        'file': test_file,
        'line': test_line,
        'path': test_path,
        'title': short_title,
        'spec': f"{test_file}:{test_line}"
    }


def create_bd_task(test_info):
    """Create a bd task for a test."""
    title = f"Fix e2e test: {test_info['title']}"
    description = f"""Fix failing e2e test: {test_info['path']}

Test location: {test_info['file']}:{test_info['line']}

To run this test:
  bin/test-e2e-single '{test_info['spec']}'

Fix the application code (not the test code) to make this test pass."""
    
    cmd = [
        'devenv', 'shell', '--',
        'bd', 'create', title,
        '-t', 'bug',
        '-p', '2',
        '-d', description,
        '--json'
    ]
    
    try:
        result = subprocess.run(cmd, capture_output=True, text=True, check=False)
        # Extract JSON from output (it's usually at the end)
        output = result.stdout + result.stderr
        import json
        import re
        
        # Find JSON object in output (look for {...})
        json_match = re.search(r'\{[^{}]*"id"[^{}]*\}', output, re.DOTALL)
        if json_match:
            task_json = json.loads(json_match.group(0))
            return task_json.get('id', 'unknown')
        else:
            # Try parsing entire output as JSON (might be just JSON)
            try:
                task_json = json.loads(output.strip())
                return task_json.get('id', 'unknown')
            except:
                # If no JSON found, check if command succeeded
                if result.returncode == 0:
                    # Command succeeded but no JSON - might have been created anyway
                    return 'created'
                return None
    except Exception as e:
        print(f"Error creating task for {test_info['spec']}: {e}", file=sys.stderr)
        return None


def main():
    """Main entry point."""
    tests = []
    for line in TEST_LINES.strip().split('\n'):
        test_info = parse_test_line(line)
        if test_info:
            tests.append(test_info)
    
    print(f"Creating bd tasks for {len(tests)} failing e2e tests...", file=sys.stderr)
    
    created = 0
    failed = 0
    
    for i, test_info in enumerate(tests, 1):
        print(f"[{i}/{len(tests)}] Creating task for {test_info['spec']}...", file=sys.stderr)
        task_id = create_bd_task(test_info)
        if task_id:
            created += 1
            print(f"  Created: {task_id}", file=sys.stderr)
        else:
            failed += 1
    
    print(f"\nDone! Created {created} tasks, {failed} failed.", file=sys.stderr)


if __name__ == '__main__':
    main()
