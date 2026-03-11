/**
 * E2E tests for Editor Role
 *
 * Tests follow the DAG structure from E2E_TEST_SCENARIOS.md:
 * - 3.1 Authentication & Profile Selection
 * - 3.2 Dashboard Access
 * - 3.3 Users Page (Create/Update)
 * - 3.4 Roles Page (Create/Update)
 * - 3.5 Permissions Page (Full CRUD)
 * - 3.6 Audit Log Page (Blocked)
 * - 3.7 Organizations Page (Blocked)
 * - 3.8 Profile Management
 * - 3.9 Generic Entity API (Users, Roles, Permissions)
 * - 3.10 RPC API (Create/Update)
 * - 3.11 Subscription Stream
 * - 3.12 Admin Endpoint (Blocked)
 * - 3.13 Logout
 */
import { test, expect } from '@playwright/test'
import { Effect } from 'effect'
import {
  LoginPage,
  DashboardPage,
  UsersPage,
  RolesPage,
  PermissionsPage,
  ProfilePage,
} from '../pages'
import { SEED_USERS, createTestUser } from '../fixtures/test-data'
import { createPageLayers } from '../fixtures/page-layers'
import { API_BASE_URL } from '../playwright.config'
import * as ExpectHelpers from '../helpers/expect'
import * as LocatorHelpers from '../helpers/locator'
import { getAuthHeadersFromPage } from '../helpers/auth'

test.describe('Editor Role', () => {
  test.describe('3.1 Authentication & Profile Selection', () => {
    test('single profile redirects to dashboard', async ({ page }) => {
      await page.goto('/authentication/login')

      const program = Effect.gen(function* () {
        const loginPageService = yield* LoginPage
        const dashboardPageService = yield* DashboardPage

        yield* loginPageService.login(
          SEED_USERS.editor.email,
          SEED_USERS.editor.password,
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

  test.describe('3.2 Dashboard Access', () => {
    test.beforeEach(async ({ page }) => {
      await page.goto('/authentication/login')
      const loginProgram = Effect.gen(function* () {
        const loginPageService = yield* LoginPage
        yield* loginPageService.login(
          SEED_USERS.editor.email,
          SEED_USERS.editor.password,
        )
      })
      await Effect.runPromise(
        loginProgram.pipe(Effect.provide(createPageLayers(page))),
      )
      await page.goto('/dashboard')
      await page.getByTestId('dashboard-layout').waitFor({ state: 'visible' })
    })

    test('should show navigation links', async ({ page }) => {
      const program = Effect.gen(function* () {
        const dashboardPageService = yield* DashboardPage

        const dashboardLink = yield* dashboardPageService.dashboardLink()
        const usersLink = yield* dashboardPageService.usersLink()
        const rolesLink = yield* dashboardPageService.rolesLink()
        const permissionsLink = yield* dashboardPageService.permissionsLink()

        yield* ExpectHelpers.toBeVisible(dashboardLink)
        yield* ExpectHelpers.toBeVisible(usersLink)
        yield* ExpectHelpers.toBeVisible(rolesLink)
        yield* ExpectHelpers.toBeVisible(permissionsLink)
      })

      await Effect.runPromise(
        program.pipe(Effect.provide(createPageLayers(page))),
      )
    })

    test('should not show organizations or audit log links', async ({
      page,
    }) => {
      const program = Effect.gen(function* () {
        const dashboardPageService = yield* DashboardPage

        const organizationsLink =
          yield* dashboardPageService.organizationsLink()
        const auditLogLink = yield* dashboardPageService.auditLogLink()

        yield* ExpectHelpers.notToBeVisible(organizationsLink)
        yield* ExpectHelpers.notToBeVisible(auditLogLink)
      })

      await Effect.runPromise(
        program.pipe(Effect.provide(createPageLayers(page))),
      )
    })
  })

  test.describe('3.3 Users Page (Create/Update)', () => {
    test.beforeEach(async ({ page }) => {
      await page.goto('/authentication/login')
      const loginProgram = Effect.gen(function* () {
        const loginPageService = yield* LoginPage
        yield* loginPageService.login(
          SEED_USERS.editor.email,
          SEED_USERS.editor.password,
        )
      })
      await Effect.runPromise(
        loginProgram.pipe(Effect.provide(createPageLayers(page))),
      )
      await page.goto('/dashboard/users')
      await page.getByTestId('dashboard-layout').waitFor({ state: 'visible' })
    })

    test('should show create button', async ({ page }) => {
      const program = Effect.gen(function* () {
        const usersPageService = yield* UsersPage

        const createButton = yield* usersPageService.createButton()
        yield* ExpectHelpers.toBeVisible(createButton)
      })

      await Effect.runPromise(
        program.pipe(Effect.provide(createPageLayers(page))),
      )
    })

    test('should create user', async ({ page }) => {
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

    test('should update user', async ({ page }) => {
      const testUser = createTestUser()
      const newEmail = `updated-${testUser.email}`

      const program = Effect.gen(function* () {
        const usersPageService = yield* UsersPage

        // Create user first
        yield* usersPageService.createUser(testUser.email, testUser.password)

        // Update user
        yield* usersPageService.updateUser(testUser.email, newEmail)

        const updatedRow = yield* usersPageService.row(newEmail)
        yield* ExpectHelpers.toBeVisible(updatedRow)
      })

      await Effect.runPromise(
        program.pipe(Effect.provide(createPageLayers(page))),
      )
    })

    test('should filter users', async ({ page }) => {
      const program = Effect.gen(function* () {
        const usersPageService = yield* UsersPage

        yield* usersPageService.filter('test')

        const list = yield* usersPageService.list()
        yield* ExpectHelpers.toBeVisible(list)
      })

      await Effect.runPromise(
        program.pipe(Effect.provide(createPageLayers(page))),
      )
    })

    test('should view user details with edit button', async ({ page }) => {
      const program = Effect.gen(function* () {
        const usersPageService = yield* UsersPage

        const list = yield* usersPageService.list()
        const firstRow = list.locator('[data-testid^="user-row-"]').first()
        yield* LocatorHelpers.click(firstRow)

        const detailsDialog = yield* usersPageService.detailsDialog()
        yield* ExpectHelpers.toBeVisible(detailsDialog)

        const editButton = page.locator('[data-testid="user-edit-button"]')
        yield* ExpectHelpers.toBeVisible(editButton)

        const deleteButton = page.locator('[data-testid="user-delete-button"]')
        yield* ExpectHelpers.notToBeVisible(deleteButton)
      })

      await Effect.runPromise(
        program.pipe(Effect.provide(createPageLayers(page))),
      )
    })
  })

  test.describe('3.4 Roles Page (Create/Update)', () => {
    test.beforeEach(async ({ page }) => {
      await page.goto('/authentication/login')
      const loginProgram = Effect.gen(function* () {
        const loginPageService = yield* LoginPage
        yield* loginPageService.login(
          SEED_USERS.editor.email,
          SEED_USERS.editor.password,
        )
      })
      await Effect.runPromise(
        loginProgram.pipe(Effect.provide(createPageLayers(page))),
      )
      await page.goto('/dashboard/roles')
      await page.getByTestId('dashboard-layout').waitFor({ state: 'visible' })
    })

    test('should create role', async ({ page }) => {
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

    test('should update role', async ({ page }) => {
      const roleName = `test-role-${Date.now()}`
      const newRoleName = `updated-${roleName}`

      const program = Effect.gen(function* () {
        const rolesPageService = yield* RolesPage

        yield* rolesPageService.createRole(roleName)
        yield* rolesPageService.updateRole(roleName, newRoleName)

        const updatedRow = yield* rolesPageService.row(newRoleName)
        yield* ExpectHelpers.toBeVisible(updatedRow)
      })

      await Effect.runPromise(
        program.pipe(Effect.provide(createPageLayers(page))),
      )
    })

    test('should view role details with edit button', async ({ page }) => {
      const program = Effect.gen(function* () {
        const rolesPageService = yield* RolesPage

        const list = yield* rolesPageService.list()
        const firstRow = list.locator('[data-testid^="role-row-"]').first()
        yield* LocatorHelpers.click(firstRow)

        const detailsDialog = yield* rolesPageService.detailsDialog()
        yield* Effect.promise(() => expect(detailsDialog).toBeVisible())

        const editButton = page.locator('[data-testid="role-edit-button"]')
        yield* ExpectHelpers.toBeVisible(editButton)

        const deleteButton = page.locator('[data-testid="role-delete-button"]')
        yield* ExpectHelpers.notToBeVisible(deleteButton)
      })

      await Effect.runPromise(
        program.pipe(Effect.provide(createPageLayers(page))),
      )
    })
  })

  test.describe('3.5 Permissions Page (Full CRUD)', () => {
    test.beforeEach(async ({ page }) => {
      await page.goto('/authentication/login')
      const loginProgram = Effect.gen(function* () {
        const loginPageService = yield* LoginPage
        yield* loginPageService.login(
          SEED_USERS.editor.email,
          SEED_USERS.editor.password,
        )
      })
      await Effect.runPromise(
        loginProgram.pipe(Effect.provide(createPageLayers(page))),
      )
      await page.goto('/dashboard/roles-and-permissions')
      await page.getByTestId('dashboard-layout').waitFor({ state: 'visible' })
    })

    test('should show add button', async ({ page }) => {
      const program = Effect.gen(function* () {
        const permissionsPageService = yield* PermissionsPage

        const addButton = yield* permissionsPageService.addButton()
        yield* ExpectHelpers.toBeVisible(addButton)
      })

      await Effect.runPromise(
        program.pipe(Effect.provide(createPageLayers(page))),
      )
    })

    test('should add permission to role', async ({ page }) => {
      const program = Effect.gen(function* () {
        const permissionsPageService = yield* PermissionsPage

        // Note: This assumes roles and permissions exist
        // In real tests, you'd create them first or use seed data
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

  test.describe('3.6 Audit Log Page (Blocked)', () => {
    test('should show 403 when accessing audit log page', async ({ page }) => {
      await page.goto('/authentication/login')
      const loginProgram = Effect.gen(function* () {
        const loginPageService = yield* LoginPage
        yield* loginPageService.login(
          SEED_USERS.editor.email,
          SEED_USERS.editor.password,
        )
      })
      await Effect.runPromise(
        loginProgram.pipe(Effect.provide(createPageLayers(page))),
      )
      await page.goto('/dashboard/audit-log')
      await page.getByTestId('dashboard-layout').waitFor({ state: 'visible' })

      const error403 = page.locator('[data-testid="error-403"]')
      await expect(
        error403
          .or(page.locator('text=403'))
          .or(page.locator('text=Forbidden')),
      ).toBeVisible()
    })
  })

  test.describe('3.7 Organizations Page (Blocked)', () => {
    test('should show 403 when accessing organizations page', async ({
      page,
    }) => {
      await page.goto('/authentication/login')
      const loginProgram = Effect.gen(function* () {
        const loginPageService = yield* LoginPage
        yield* loginPageService.login(
          SEED_USERS.editor.email,
          SEED_USERS.editor.password,
        )
      })
      await Effect.runPromise(
        loginProgram.pipe(Effect.provide(createPageLayers(page))),
      )
      await page.goto('/dashboard/organizations')
      await page.getByTestId('dashboard-layout').waitFor({ state: 'visible' })

      const error403 = page.locator('[data-testid="error-403"]')
      await expect(
        error403
          .or(page.locator('text=403'))
          .or(page.locator('text=Forbidden')),
      ).toBeVisible()
    })
  })

  test.describe('3.8 Profile Management', () => {
    test('should display current profile', async ({ page }) => {
      await page.goto('/authentication/login')
      const loginProgram = Effect.gen(function* () {
        const loginPageService = yield* LoginPage
        yield* loginPageService.login(
          SEED_USERS.editor.email,
          SEED_USERS.editor.password,
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

  test.describe('3.9 Generic Entity API', () => {
    test('POST /api/entities/user returns 201 Created', async ({ page }) => {
      await page.goto('/authentication/login')
      const loginProgram = Effect.gen(function* () {
        const loginPageService = yield* LoginPage
        yield* loginPageService.login(
          SEED_USERS.editor.email,
          SEED_USERS.editor.password,
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
      const response = await page.request.post(
        `${API_BASE_URL}/api/entities/user`,
        {
          headers: { ...headers, 'Content-Type': 'application/json' },
          data: { email: testUser.email, password: testUser.password },
        },
      )
      expect(response.status()).toBe(201)
    })

    test('PATCH /api/entities/user/{id} returns 200 OK', async ({ page }) => {
      await page.goto('/authentication/login')
      const loginProgram = Effect.gen(function* () {
        const loginPageService = yield* LoginPage
        yield* loginPageService.login(
          SEED_USERS.editor.email,
          SEED_USERS.editor.password,
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
        `${API_BASE_URL}/api/entities/user`,
        {
          headers: { ...headers, 'Content-Type': 'application/json' },
          data: { email: testUser.email, password: testUser.password },
        },
      )
      const created = await createResponse.json()

      const updateResponse = await page.request.patch(
        `${API_BASE_URL}/api/entities/user/${created.id}`,
        {
          headers: { ...headers, 'Content-Type': 'application/json' },
          data: { email: `updated-${testUser.email}` },
        },
      )
      expect(updateResponse.status()).toBe(200)
    })

    test('GET /api/entities/organization returns 403 Forbidden', async ({
      page,
    }) => {
      await page.goto('/authentication/login')
      const loginProgram = Effect.gen(function* () {
        const loginPageService = yield* LoginPage
        yield* loginPageService.login(
          SEED_USERS.editor.email,
          SEED_USERS.editor.password,
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
        `${API_BASE_URL}/api/entities/organization`,
        { headers },
      )
      expect(response.status()).toBe(403)
    })
  })

  test.describe('3.10 RPC API (Create/Update)', () => {
    test('POST /api/rpc entity.create returns 200 OK', async ({ page }) => {
      await page.goto('/authentication/login')
      const loginProgram = Effect.gen(function* () {
        const loginPageService = yield* LoginPage
        yield* loginPageService.login(
          SEED_USERS.editor.email,
          SEED_USERS.editor.password,
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
      const response = await page.request.post(`${API_BASE_URL}/api/rpc`, {
        headers: { ...headers, 'Content-Type': 'application/json' },
        data: {
          method: 'entity.create',
          entity_id: 'user',
          params: {
            body: { email: testUser.email, password: testUser.password },
          },
        },
      })
      expect(response.status()).toBe(200)
    })

    test('POST /api/rpc entity.delete returns 403 Forbidden', async ({
      page,
    }) => {
      await page.goto('/authentication/login')
      const loginProgram = Effect.gen(function* () {
        const loginPageService = yield* LoginPage
        yield* loginPageService.login(
          SEED_USERS.editor.email,
          SEED_USERS.editor.password,
        )
      })
      await Effect.runPromise(
        loginProgram.pipe(Effect.provide(createPageLayers(page))),
      )
      await page.getByTestId('dashboard-layout').waitFor({
        state: 'visible',
      })
      const headers = await getAuthHeadersFromPage(page)
      const response = await page.request.post(`${API_BASE_URL}/api/rpc`, {
        headers: { ...headers, 'Content-Type': 'application/json' },
        data: {
          method: 'entity.delete',
          entity_id: 'user',
          params: { id: 'test-id' },
        },
      })
      expect(response.status()).toBe(403)
    })
  })

  test.describe('3.11 Subscription Stream', () => {
    test('GET /api/subscriptions/stream establishes SSE connection', async ({
      page,
    }) => {
      await page.goto('/authentication/login')
      const loginProgram = Effect.gen(function* () {
        const loginPageService = yield* LoginPage
        yield* loginPageService.login(
          SEED_USERS.editor.email,
          SEED_USERS.editor.password,
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

  test.describe('3.12 Admin Endpoint (Blocked)', () => {
    test('GET /api/auth/admin returns 403 Forbidden', async ({ page }) => {
      await page.goto('/authentication/login')
      const loginProgram = Effect.gen(function* () {
        const loginPageService = yield* LoginPage
        yield* loginPageService.login(
          SEED_USERS.editor.email,
          SEED_USERS.editor.password,
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
        {
          headers,
        },
      )
      expect(response.status()).toBe(403)
    })
  })

  test.describe('3.13 Logout', () => {
    test('logout redirects to login', async ({ page }) => {
      await page.goto('/authentication/login')
      const loginProgram = Effect.gen(function* () {
        const loginPageService = yield* LoginPage
        yield* loginPageService.login(
          SEED_USERS.editor.email,
          SEED_USERS.editor.password,
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
