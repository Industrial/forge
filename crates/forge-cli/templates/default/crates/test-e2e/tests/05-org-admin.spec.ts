/**
 * E2E tests for Org Admin Role
 *
 * Note: Org Admin follows the same test scenarios as Org Owner (Section 4)
 * All interactions and verifications use the same data-testid attributes.
 * The only difference is the user credentials used for testing.
 */
import { test, expect } from '@playwright/test'
import { Effect } from 'effect'
import {
  LoginPage,
  DashboardPage,
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
import * as PageHelpers from '@/helpers/page'
import { getAuthHeadersFromPage } from '@/helpers/auth'

test.describe('Org Admin Role', () => {
  test.describe('5.1 Authentication & Profile Selection', () => {
    test('single profile redirects to dashboard', async ({ page }) => {
      await page.goto('/authentication/login')

      const program = Effect.gen(function* () {
        const loginPageService = yield* LoginPage
        const dashboardPageService = yield* DashboardPage

        yield* loginPageService.login(
          SEED_USERS.orgAdmin.email,
          SEED_USERS.orgAdmin.password,
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

  test.describe('5.2 Dashboard Access (Full)', () => {
    test('should show all navigation links', async ({ page }) => {
      await page.goto('/authentication/login')
      const loginProgram = Effect.gen(function* () {
        const loginPageService = yield* LoginPage
        yield* loginPageService.login(
          SEED_USERS.orgAdmin.email,
          SEED_USERS.orgAdmin.password,
        )
      })
      await Effect.runPromise(
        loginProgram.pipe(Effect.provide(createPageLayers(page))),
      )
      await page.goto('/dashboard')
      await page.getByTestId('dashboard-layout').waitFor({ state: 'visible' })

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
  })

  test.describe('5.3 Organizations Page (Full CRUD)', () => {
    test('should create organization', async ({ page }) => {
      await page.goto('/authentication/login')
      const loginProgram = Effect.gen(function* () {
        const loginPageService = yield* LoginPage
        yield* loginPageService.login(
          SEED_USERS.orgAdmin.email,
          SEED_USERS.orgAdmin.password,
        )
      })
      await Effect.runPromise(
        loginProgram.pipe(Effect.provide(createPageLayers(page))),
      )
      await page.goto('/dashboard/organizations')
      await page.getByTestId('dashboard-layout').waitFor({ state: 'visible' })
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

  test.describe('5.4 Users Page (Full CRUD)', () => {
    test('should create user', async ({ page }) => {
      await page.goto('/authentication/login')
      const loginProgram = Effect.gen(function* () {
        const loginPageService = yield* LoginPage
        yield* loginPageService.login(
          SEED_USERS.orgAdmin.email,
          SEED_USERS.orgAdmin.password,
        )
      })
      await Effect.runPromise(
        loginProgram.pipe(Effect.provide(createPageLayers(page))),
      )
      await page.goto('/dashboard/users')
      await page.getByTestId('dashboard-layout').waitFor({ state: 'visible' })
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

  test.describe('5.5 Roles Page (Full CRUD)', () => {
    test('should create role', async ({ page }) => {
      await page.goto('/authentication/login')
      const loginProgram = Effect.gen(function* () {
        const loginPageService = yield* LoginPage
        yield* loginPageService.login(
          SEED_USERS.orgAdmin.email,
          SEED_USERS.orgAdmin.password,
        )
      })
      await Effect.runPromise(
        loginProgram.pipe(Effect.provide(createPageLayers(page))),
      )
      await page.goto('/dashboard/roles')
      await page.getByTestId('dashboard-layout').waitFor({ state: 'visible' })
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

  test.describe('5.6 Permissions Page (Full CRUD)', () => {
    test('should add permission to role', async ({ page }) => {
      await page.goto('/authentication/login')
      const loginProgram = Effect.gen(function* () {
        const loginPageService = yield* LoginPage
        yield* loginPageService.login(
          SEED_USERS.orgAdmin.email,
          SEED_USERS.orgAdmin.password,
        )
      })
      await Effect.runPromise(
        loginProgram.pipe(Effect.provide(createPageLayers(page))),
      )
      await page.goto('/dashboard/roles-and-permissions')
      await page.getByTestId('dashboard-layout').waitFor({ state: 'visible' })

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

  test.describe('5.7 Audit Log Page', () => {
    test('should display audit log list', async ({ page }) => {
      await page.goto('/authentication/login')
      const loginProgram = Effect.gen(function* () {
        const loginPageService = yield* LoginPage
        yield* loginPageService.login(
          SEED_USERS.orgAdmin.email,
          SEED_USERS.orgAdmin.password,
        )
      })
      await Effect.runPromise(
        loginProgram.pipe(Effect.provide(createPageLayers(page))),
      )
      await page.goto('/dashboard/audit-log')
      await page.getByTestId('dashboard-layout').waitFor({ state: 'visible' })

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

  test.describe('5.8 Profile Management', () => {
    test('should display current profile', async ({ page }) => {
      await page.goto('/authentication/login')
      const loginProgram = Effect.gen(function* () {
        const loginPageService = yield* LoginPage
        yield* loginPageService.login(
          SEED_USERS.orgAdmin.email,
          SEED_USERS.orgAdmin.password,
        )
      })
      await Effect.runPromise(
        loginProgram.pipe(Effect.provide(createPageLayers(page))),
      )
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

  test.describe('5.9 Generic Entity API (Full CRUD)', () => {
    test('POST /api/entities/organization returns 201 Created', async ({
      page,
    }) => {
      await page.goto('/authentication/login')
      const loginProgram = Effect.gen(function* () {
        const loginPageService = yield* LoginPage
        yield* loginPageService.login(
          SEED_USERS.orgAdmin.email,
          SEED_USERS.orgAdmin.password,
        )
      })
      await Effect.runPromise(
        loginProgram.pipe(Effect.provide(createPageLayers(page))),
      )
      await page.getByTestId('dashboard-layout').waitFor({
        state: 'visible',
      })
      const headers = await getAuthHeadersFromPage(page)
      const testOrg = createTestOrganization()
      const response = await page.request.post(
        `${API_BASE_URL}/api/entities/organization`,
        {
          headers: { ...headers, 'Content-Type': 'application/json' },
          data: { name: testOrg.name, slug: testOrg.slug },
        },
      )
      expect(response.status()).toBe(201)
    })
  })

  test.describe('5.10 RPC API (Full CRUD)', () => {
    test('POST /api/rpc entity.delete returns 200 OK', async ({ page }) => {
      await page.goto('/authentication/login')
      const loginProgram = Effect.gen(function* () {
        const loginPageService = yield* LoginPage
        yield* loginPageService.login(
          SEED_USERS.orgAdmin.email,
          SEED_USERS.orgAdmin.password,
        )
      })
      await Effect.runPromise(
        loginProgram.pipe(Effect.provide(createPageLayers(page))),
      )
      await page.getByTestId('dashboard-layout').waitFor({
        state: 'visible',
      })
      const headers = await getAuthHeadersFromPage(page)
      const testUser = createTestUser()
      const createResponse = await page.request.post(
        `${API_BASE_URL}/api/rpc`,
        {
          headers: { ...headers, 'Content-Type': 'application/json' },
          data: {
            method: 'entity.create',
            entity_id: 'user',
            params: {
              body: { email: testUser.email, password: testUser.password },
            },
          },
        },
      )
      const created = await createResponse.json()

      const deleteResponse = await page.request.post(
        `${API_BASE_URL}/api/rpc`,
        {
          headers: { ...headers, 'Content-Type': 'application/json' },
          data: {
            method: 'entity.delete',
            entity_id: 'user',
            params: { id: created.result.id },
          },
        },
      )
      expect(deleteResponse.status()).toBe(200)
    })
  })

  test.describe('5.11 Subscription Stream', () => {
    test('GET /api/subscriptions/stream establishes SSE connection', async ({
      page,
    }) => {
      await page.goto('/authentication/login')
      const loginProgram = Effect.gen(function* () {
        const loginPageService = yield* LoginPage
        yield* loginPageService.login(
          SEED_USERS.orgAdmin.email,
          SEED_USERS.orgAdmin.password,
        )
      })
      await Effect.runPromise(
        loginProgram.pipe(Effect.provide(createPageLayers(page))),
      )
      await page.getByTestId('dashboard-layout').waitFor({
        state: 'visible',
      })
      const headers = await getAuthHeadersFromPage(page)
      const response = await page.request.get(
        `${API_BASE_URL}/api/subscriptions/stream`,
        {
          headers: { ...headers, Accept: 'text/event-stream' },
        },
      )
      expect(response.status()).toBe(200)
    })
  })

  test.describe('5.12 Admin Endpoint (Blocked)', () => {
    test('GET /api/auth/admin returns 403 Forbidden', async ({ page }) => {
      await page.goto('/authentication/login')
      const loginProgram = Effect.gen(function* () {
        const loginPageService = yield* LoginPage
        yield* loginPageService.login(
          SEED_USERS.orgAdmin.email,
          SEED_USERS.orgAdmin.password,
        )
      })
      await Effect.runPromise(
        loginProgram.pipe(Effect.provide(createPageLayers(page))),
      )
      await page.getByTestId('dashboard-layout').waitFor({
        state: 'visible',
      })
      const headers = await getAuthHeadersFromPage(page)
      const response = await page.request.get(
        `${API_BASE_URL}/api/auth/admin`,
        { headers },
      )
      expect(response.status()).toBe(403)
    })
  })

  test.describe('5.13 Scope Switching', () => {
    test('should switch profile and see updated data', async ({ page }) => {
      await page.goto('/authentication/login')
      const loginProgram = Effect.gen(function* () {
        const loginPageService = yield* LoginPage
        yield* loginPageService.login(
          SEED_USERS.orgAdmin.email,
          SEED_USERS.orgAdmin.password,
        )
      })
      await Effect.runPromise(
        loginProgram.pipe(Effect.provide(createPageLayers(page))),
      )
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

  test.describe('5.14 Logout', () => {
    test('logout redirects to login', async ({ page }) => {
      await page.goto('/authentication/login')
      const loginProgram = Effect.gen(function* () {
        const loginPageService = yield* LoginPage
        yield* loginPageService.login(
          SEED_USERS.orgAdmin.email,
          SEED_USERS.orgAdmin.password,
        )
      })
      await Effect.runPromise(
        loginProgram.pipe(Effect.provide(createPageLayers(page))),
      )
      await page.goto('/dashboard')
      await page.getByTestId('dashboard-layout').waitFor({ state: 'visible' })

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
