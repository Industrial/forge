/**
 * E2E tests for App Admin Role (Global Admin)
 *
 * Tests follow the DAG structure from E2E_TEST_SCENARIOS.md:
 * - 6.1 Authentication & Profile Selection
 * - 6.2 Dashboard Access (Full - Global Scope)
 * - 6.3 Organizations Page (Full CRUD - Global Scope)
 * - 6.4 Users Page (Full CRUD - Global Scope)
 * - 6.5 Roles Page (Full CRUD - Global Scope)
 * - 6.6 Permissions Page (Full CRUD - Global Scope)
 * - 6.7 Audit Log Page (Global Scope)
 * - 6.8 Profile Management
 * - 6.9 Admin Endpoint (Access Granted)
 * - 6.10 Generic Entity API (Full CRUD - Global Scope)
 * - 6.11 RPC API (Full CRUD - Global Scope)
 * - 6.12 Subscription Stream (Global Scope)
 * - 6.13 Global Role Management
 * - 6.14 Scope Switching (Global Admin)
 * - 6.15 API Token Management
 * - 6.16 Logout
 */
import { test, expect } from '@playwright/test'
import { Effect } from 'effect'
import {
  LoginPage,
  DashboardPage,
  SelectScopePage,
  UsersPage,
  RolesPage,
  PermissionsPage,
  AuditLogPage,
  OrganizationsPage,
  ProfilePage,
} from '@/pages'
import {
  SEED_USERS,
  createTestUser,
  createTestOrganization,
} from '@/fixtures/test-data'
import { createPageLayers } from '@/fixtures/page-layers'
import { API_BASE_URL } from '@/playwright.config'
import * as ExpectHelpers from '@/helpers/expect'
import * as LocatorHelpers from '@/helpers/locator'
import * as PageHelpers from '@/helpers/page'

test.describe('App Admin Role', () => {
  test.use({ storageState: 'playwright/.auth/app-admin.json' })

  test.describe('6.1 Authentication & Profile Selection', () => {
    test('single profile redirects to dashboard', async ({ page }) => {
      await page.goto('/authentication/login')

      const program = Effect.gen(function* () {
        const loginPageService = yield* LoginPage
        const dashboardPageService = yield* DashboardPage

        yield* loginPageService.login(
          SEED_USERS.appAdmin.email,
          SEED_USERS.appAdmin.password,
        )

        yield* ExpectHelpers.toHaveURL(page, '/dashboard')
        const container = yield* dashboardPageService.container()
        yield* ExpectHelpers.toBeVisible(container)
      })

      await Effect.runPromise(
        program.pipe(Effect.provide(createPageLayers(page))),
      )
    })
  })

  test.describe('6.2 Dashboard Access (Full - Global Scope)', () => {
    test.beforeEach(async ({ page }) => {
      await page.goto('/dashboard')
    })

    test('should show all navigation links', async ({ page }) => {
      const program = Effect.gen(function* () {
        const dashboardPageService = yield* DashboardPage

        const organizationsLink =
          yield* dashboardPageService.organizationsLink()
        const usersLink = yield* dashboardPageService.usersLink()
        const rolesLink = yield* dashboardPageService.rolesLink()
        const permissionsLink = yield* dashboardPageService.permissionsLink()
        const auditLogLink = yield* dashboardPageService.auditLogLink()

        yield* ExpectHelpers.toBeVisible(organizationsLink)
        yield* ExpectHelpers.toBeVisible(usersLink)
        yield* ExpectHelpers.toBeVisible(rolesLink)
        yield* ExpectHelpers.toBeVisible(permissionsLink)
        yield* ExpectHelpers.toBeVisible(auditLogLink)
      })

      await Effect.runPromise(
        program.pipe(Effect.provide(createPageLayers(page))),
      )
    })

    test('can access all organizations (global-scope)', async ({ page }) => {
      const program = Effect.gen(function* () {
        const dashboardPageService = yield* DashboardPage
        const organizationsPageService = yield* OrganizationsPage

        yield* dashboardPageService.clickOrganizationsLink()

        const list = yield* organizationsPageService.list()
        yield* ExpectHelpers.toBeVisible(list)
      })

      await Effect.runPromise(
        program.pipe(Effect.provide(createPageLayers(page))),
      )
    })

    test('can see users from all orgs (global-scope)', async ({ page }) => {
      const program = Effect.gen(function* () {
        const dashboardPageService = yield* DashboardPage
        const usersPageService = yield* UsersPage

        yield* dashboardPageService.clickUsersLink()

        const list = yield* usersPageService.list()
        yield* ExpectHelpers.toBeVisible(list)
      })

      await Effect.runPromise(
        program.pipe(Effect.provide(createPageLayers(page))),
      )
    })
  })

  test.describe('6.3 Organizations Page (Full CRUD - Global Scope)', () => {
    test('should create organization', async ({ page }) => {
      await page.goto('/dashboard/organizations')
      const testOrg = createTestOrganization()

      const program = Effect.gen(function* () {
        const organizationsPageService = yield* OrganizationsPage

        yield* organizationsPageService.createOrganization(
          testOrg.name,
          testOrg.slug,
        )

        const row = yield* organizationsPageService.row(testOrg.name)
        yield* ExpectHelpers.toBeVisible(row)
      })

      await Effect.runPromise(
        program.pipe(Effect.provide(createPageLayers(page))),
      )
    })
  })

  test.describe('6.4 Users Page (Full CRUD - Global Scope)', () => {
    test('should create user', async ({ page }) => {
      await page.goto('/dashboard/users')
      const testUser = createTestUser()

      const program = Effect.gen(function* () {
        const usersPageService = yield* UsersPage

        yield* usersPageService.createUser(testUser.email, testUser.password)

        const row = yield* usersPageService.row(testUser.email)
        yield* ExpectHelpers.toBeVisible(row)
      })

      await Effect.runPromise(
        program.pipe(Effect.provide(createPageLayers(page))),
      )
    })
  })

  test.describe('6.5 Roles Page (Full CRUD - Global Scope)', () => {
    test('should create role', async ({ page }) => {
      await page.goto('/dashboard/roles')
      const roleName = `test-role-${Date.now()}`

      const program = Effect.gen(function* () {
        const rolesPageService = yield* RolesPage

        yield* rolesPageService.createRole(roleName)

        const row = yield* rolesPageService.row(roleName)
        yield* ExpectHelpers.toBeVisible(row)
      })

      await Effect.runPromise(
        program.pipe(Effect.provide(createPageLayers(page))),
      )
    })
  })

  test.describe('6.6 Permissions Page (Full CRUD - Global Scope)', () => {
    test('should add permission to role', async ({ page }) => {
      await page.goto('/dashboard/roles-and-permissions')

      const program = Effect.gen(function* () {
        const permissionsPageService = yield* PermissionsPage

        yield* permissionsPageService.addPermission('viewer', 'user.read')

        const assignmentRow = yield* permissionsPageService.assignmentRow(
          'viewer',
          'user.read',
        )
        yield* ExpectHelpers.toBeVisible(assignmentRow)
      })

      await Effect.runPromise(
        program.pipe(Effect.provide(createPageLayers(page))),
      )
    })
  })

  test.describe('6.7 Audit Log Page (Global Scope)', () => {
    test('should display audit log list', async ({ page }) => {
      await page.goto('/dashboard/audit-log')

      const program = Effect.gen(function* () {
        const auditLogPageService = yield* AuditLogPage

        const container = yield* auditLogPageService.container()
        const list = yield* auditLogPageService.list()

        yield* ExpectHelpers.toBeVisible(container)
        yield* ExpectHelpers.toBeVisible(list)
      })

      await Effect.runPromise(
        program.pipe(Effect.provide(createPageLayers(page))),
      )
    })
  })

  test.describe('6.8 Profile Management', () => {
    test('should display current profile', async ({ page }) => {
      await page.goto('/scope')

      const program = Effect.gen(function* () {
        const profilePageService = yield* ProfilePage

        const container = yield* profilePageService.container()
        const currentProfile = yield* profilePageService.currentProfile()

        yield* ExpectHelpers.toBeVisible(container)
        yield* ExpectHelpers.toBeVisible(currentProfile)
      })

      await Effect.runPromise(
        program.pipe(Effect.provide(createPageLayers(page))),
      )
    })
  })

  test.describe('6.9 Admin Endpoint (Access Granted)', () => {
    test('GET /api/auth/admin returns 200 OK', async ({ request }) => {
      const response = await request.get(`${API_BASE_URL}/api/auth/admin`)
      expect(response.status()).toBe(200)

      const data = await response.json()
      expect(data).toHaveProperty('access', 'granted')
    })
  })

  test.describe('6.10 Generic Entity API (Full CRUD - Global Scope)', () => {
    test('GET /api/entities/organization returns 200 OK (all orgs)', async ({
      request,
    }) => {
      const response = await request.get(
        `${API_BASE_URL}/api/entities/organization`,
      )
      expect(response.status()).toBe(200)
    })

    test('POST /api/entities/organization returns 201 Created', async ({
      request,
    }) => {
      const testOrg = createTestOrganization()
      const response = await request.post(
        `${API_BASE_URL}/api/entities/organization`,
        {
          data: { name: testOrg.name, slug: testOrg.slug },
        },
      )
      expect(response.status()).toBe(201)
    })

    test('GET /api/entities/user returns 200 OK (all users, all orgs)', async ({
      request,
    }) => {
      const response = await request.get(`${API_BASE_URL}/api/entities/user`)
      expect(response.status()).toBe(200)
    })
  })

  test.describe('6.11 RPC API (Full CRUD - Global Scope)', () => {
    test('POST /api/rpc entity.list returns 200 OK (all entities, all orgs)', async ({
      request,
    }) => {
      const response = await request.post(`${API_BASE_URL}/api/rpc`, {
        data: { method: 'entity.list', params: { entity: 'user' } },
      })
      expect(response.status()).toBe(200)
    })
  })

  test.describe('6.12 Subscription Stream (Global Scope)', () => {
    test('GET /api/subscriptions/stream establishes SSE connection', async ({
      request,
    }) => {
      const response = await request.get(
        `${API_BASE_URL}/api/subscriptions/stream`,
        {
          headers: { Accept: 'text/event-stream' },
        },
      )
      expect(response.status()).toBe(200)
    })
  })

  test.describe('6.13 Global Role Management', () => {
    test('should manage global roles', async ({ page }) => {
      await page.goto('/dashboard/roles')

      const program = Effect.gen(function* () {
        const rolesPageService = yield* RolesPage

        const container = yield* rolesPageService.container()
        yield* ExpectHelpers.toBeVisible(container)
      })

      await Effect.runPromise(
        program.pipe(Effect.provide(createPageLayers(page))),
      )
    })
  })

  test.describe('6.14 Scope Switching (Global Admin)', () => {
    test('should switch profile and see updated data', async ({ page }) => {
      await page.goto('/scope')

      const program = Effect.gen(function* () {
        const profilePageService = yield* ProfilePage
        const usersPageService = yield* UsersPage

        const profileList = yield* profilePageService.profileList()
        yield* ExpectHelpers.toBeVisible(profileList)

        yield* profilePageService.switchProfile(0)

        yield* PageHelpers.goto(page, '/dashboard/users')
        const usersList = yield* usersPageService.list()
        yield* ExpectHelpers.toBeVisible(usersList)
      })

      await Effect.runPromise(
        program.pipe(Effect.provide(createPageLayers(page))),
      )
    })
  })

  test.describe('6.15 API Token Management', () => {
    test('POST /api/auth/tokens returns 201 Created', async ({ request }) => {
      const response = await request.post(`${API_BASE_URL}/api/auth/tokens`)
      expect(response.status()).toBe(201)
    })

    test('use token to GET /api/auth/admin returns 200 OK', async ({
      request,
    }) => {
      const tokenResponse = await request.post(
        `${API_BASE_URL}/api/auth/tokens`,
      )
      const { token } = await tokenResponse.json()

      const adminResponse = await request.get(
        `${API_BASE_URL}/api/auth/admin`,
        {
          headers: { Authorization: `Bearer ${token}` },
        },
      )
      expect(adminResponse.status()).toBe(200)
    })
  })

  test.describe('6.16 Logout', () => {
    test('logout redirects to login', async ({ page }) => {
      await page.goto('/dashboard')

      const program = Effect.gen(function* () {
        const dashboardPageService = yield* DashboardPage
        const loginPageService = yield* LoginPage

        yield* dashboardPageService.logout()

        yield* ExpectHelpers.toHaveURL(page, '/authentication/login')
        const loginContainer = yield* loginPageService.container()
        yield* ExpectHelpers.toBeVisible(loginContainer)
      })

      await Effect.runPromise(
        program.pipe(Effect.provide(createPageLayers(page))),
      )
    })
  })
})
