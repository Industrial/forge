/**
 * E2E tests for Viewer Role
 *
 * Tests follow the DAG structure from E2E_TEST_SCENARIOS.md:
 * - 2.1 Authentication & Profile Selection
 * - 2.2 Dashboard Access
 * - 2.3 Users Page (Read-only)
 * - 2.4 Roles Page (Read-only)
 * - 2.5 Permissions Page (Read-only)
 * - 2.6 Audit Log Page
 * - 2.7 Organizations Page (Blocked)
 * - 2.8 Profile Management
 * - 2.9 API Token Management
 * - 2.10 Generic Entity API (Read-only)
 * - 2.11 RPC API (Read-only)
 * - 2.12 Subscription Stream
 * - 2.13 Admin Endpoint (Blocked)
 * - 2.14 Logout
 *
 * Uses Effect.ts for composition and Playwright for browser automation.
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
  ProfilePage,
} from '@/pages'
import { SEED_USERS } from '@/fixtures/test-data'
import { createPageLayers } from '@/fixtures/page-layers'
import { API_BASE_URL } from '@/playwright.config'
import * as ExpectHelpers from '@/helpers/expect'
import * as LocatorHelpers from '@/helpers/locator'
import * as PageHelpers from '@/helpers/page'

test.describe('Viewer Role', () => {
  test.describe('2.1 Authentication & Profile Selection', () => {
    test('single profile redirects to dashboard', async ({ page }) => {
      await page.goto('/authentication/login')

      const program = Effect.gen(function* () {
        const loginPageService = yield* LoginPage
        const dashboardPageService = yield* DashboardPage

        yield* loginPageService.login(
          SEED_USERS.viewer.email,
          SEED_USERS.viewer.password,
        )

        yield* ExpectHelpers.toHaveURL(page, '/dashboard')
        const dashboardContainer = yield* dashboardPageService.container()
        yield* ExpectHelpers.toBeVisible(dashboardContainer)
      })

      await Effect.runPromise(
        program.pipe(Effect.provide(createPageLayers(page))),
      )
    })

    test('multi-profile redirects to scope selection then dashboard', async ({
      page,
    }) => {
      await page.goto('/authentication/login')

      const program = Effect.gen(function* () {
        const loginPageService = yield* LoginPage
        const selectScopePageService = yield* SelectScopePage
        const dashboardPageService = yield* DashboardPage

        yield* loginPageService.login(
          SEED_USERS.multiProfile.email,
          SEED_USERS.multiProfile.password,
        )

        yield* ExpectHelpers.toHaveURL(page, '/authentication/select-scope')
        const scopeContainer = yield* selectScopePageService.container()
        yield* ExpectHelpers.toBeVisible(scopeContainer)

        const profileList = yield* selectScopePageService.profileList()
        yield* ExpectHelpers.toBeVisible(profileList)

        yield* selectScopePageService.selectProfile(0)

        yield* ExpectHelpers.toHaveURL(page, '/dashboard')
        const dashboardContainer = yield* dashboardPageService.container()
        yield* ExpectHelpers.toBeVisible(dashboardContainer)
      })

      await Effect.runPromise(
        program.pipe(Effect.provide(createPageLayers(page))),
      )
    })
  })

  test.describe('2.2 Dashboard Access', () => {
    test.beforeEach(async ({ page }) => {
      await page.goto('/authentication/login')
      const program = Effect.gen(function* () {
        const loginPageService = yield* LoginPage
        yield* loginPageService.login(
          SEED_USERS.viewer.email,
          SEED_USERS.viewer.password,
        )
      })
      await Effect.runPromise(
        program.pipe(Effect.provide(createPageLayers(page))),
      )
      await page.goto('/dashboard')
    })

    test('should display dashboard page', async ({ page }) => {
      const program = Effect.gen(function* () {
        const dashboardPageService = yield* DashboardPage

        const container = yield* dashboardPageService.container()
        const layout = yield* dashboardPageService.layout()
        const sidebar = yield* dashboardPageService.sidebar()
        const navbar = yield* dashboardPageService.navbar()
        const content = yield* dashboardPageService.content()

        yield* ExpectHelpers.toBeVisible(container)
        yield* ExpectHelpers.toBeVisible(layout)
        yield* ExpectHelpers.toBeVisible(sidebar)
        yield* ExpectHelpers.toBeVisible(navbar)
        yield* ExpectHelpers.toBeVisible(content)
      })

      await Effect.runPromise(
        program.pipe(Effect.provide(createPageLayers(page))),
      )
    })

    test('should show navigation links', async ({ page }) => {
      const program = Effect.gen(function* () {
        const dashboardPageService = yield* DashboardPage

        const dashboardLink = yield* dashboardPageService.dashboardLink()
        const usersLink = yield* dashboardPageService.usersLink()
        const rolesLink = yield* dashboardPageService.rolesLink()
        const permissionsLink = yield* dashboardPageService.permissionsLink()
        const auditLogLink = yield* dashboardPageService.auditLogLink()

        yield* ExpectHelpers.toBeVisible(dashboardLink)
        yield* ExpectHelpers.toBeVisible(usersLink)
        yield* ExpectHelpers.toBeVisible(rolesLink)
        yield* ExpectHelpers.toBeVisible(permissionsLink)
        yield* ExpectHelpers.toBeVisible(auditLogLink)
      })

      await Effect.runPromise(
        program.pipe(Effect.provide(createPageLayers(page))),
      )
    })

    test('should not show organizations link', async ({ page }) => {
      const program = Effect.gen(function* () {
        const dashboardPageService = yield* DashboardPage

        const organizationsLink =
          yield* dashboardPageService.organizationsLink()

        yield* ExpectHelpers.notToBeVisible(organizationsLink)
      })

      await Effect.runPromise(
        program.pipe(Effect.provide(createPageLayers(page))),
      )
    })

    test('theme toggle works', async ({ page }) => {
      const program = Effect.gen(function* () {
        const dashboardPageService = yield* DashboardPage

        const themeToggle = yield* dashboardPageService.themeToggleButton()
        yield* ExpectHelpers.toBeVisible(themeToggle)

        yield* dashboardPageService.toggleTheme()

        // Check theme changed (body class or data attribute)
        const body = page.locator('body')
        yield* ExpectHelpers.toHaveAttribute(body, 'data-theme', /dark|light/)

        yield* dashboardPageService.toggleTheme()

        // Theme changes back
        yield* ExpectHelpers.toHaveAttribute(body, 'data-theme', /dark|light/)
      })

      await Effect.runPromise(
        program.pipe(Effect.provide(createPageLayers(page))),
      )
    })
  })

  test.describe('2.3 Users Page', () => {
    test.beforeEach(async ({ page }) => {
      await page.goto('/authentication/login')
      const loginProgram = Effect.gen(function* () {
        const loginPageService = yield* LoginPage
        yield* loginPageService.login(
          SEED_USERS.viewer.email,
          SEED_USERS.viewer.password,
        )
      })
      await Effect.runPromise(
        loginProgram.pipe(Effect.provide(createPageLayers(page))),
      )
      await page.goto('/dashboard')
      const program = Effect.gen(function* () {
        const dashboardPageService = yield* DashboardPage
        yield* dashboardPageService.clickUsersLink()
      })
      await Effect.runPromise(
        program.pipe(Effect.provide(createPageLayers(page))),
      )
      await expect(page).toHaveURL('/dashboard/users')
    })

    test('should display users list', async ({ page }) => {
      const program = Effect.gen(function* () {
        const usersPageService = yield* UsersPage

        const container = yield* usersPageService.container()
        const pageTitle = yield* usersPageService.pageTitle()
        const list = yield* usersPageService.list()

        yield* ExpectHelpers.toBeVisible(container)
        yield* ExpectHelpers.toBeVisible(pageTitle)
        yield* ExpectHelpers.toBeVisible(list)
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

    test('should view user details (read-only)', async ({ page }) => {
      const program = Effect.gen(function* () {
        const usersPageService = yield* UsersPage

        // Click first user row
        const list = yield* usersPageService.list()
        const firstRow = list.locator('[data-testid^="user-row-"]').first()
        yield* LocatorHelpers.click(firstRow)

        const detailsDialog = yield* usersPageService.detailsDialog()
        yield* ExpectHelpers.toBeVisible(detailsDialog)

        // Verify edit/delete buttons are not visible
        const editButton = page.locator('[data-testid="user-edit-button"]')
        const deleteButton = page.locator('[data-testid="user-delete-button"]')
        yield* ExpectHelpers.notToBeVisible(editButton)
        yield* ExpectHelpers.notToBeVisible(deleteButton)
      })

      await Effect.runPromise(
        program.pipe(Effect.provide(createPageLayers(page))),
      )
    })

    test('should not show create button', async ({ page }) => {
      const program = Effect.gen(function* () {
        const usersPageService = yield* UsersPage

        const createButton = yield* usersPageService.createButton()
        yield* ExpectHelpers.notToBeVisible(createButton)
      })

      await Effect.runPromise(
        program.pipe(Effect.provide(createPageLayers(page))),
      )
    })
  })

  test.describe('2.4 Roles Page', () => {
    test.beforeEach(async ({ page }) => {
      await page.goto('/authentication/login')
      const loginProgram = Effect.gen(function* () {
        const loginPageService = yield* LoginPage
        yield* loginPageService.login(
          SEED_USERS.viewer.email,
          SEED_USERS.viewer.password,
        )
      })
      await Effect.runPromise(
        loginProgram.pipe(Effect.provide(createPageLayers(page))),
      )
      await page.goto('/dashboard')
      const program = Effect.gen(function* () {
        const dashboardPageService = yield* DashboardPage
        yield* dashboardPageService.clickRolesLink()
      })
      await Effect.runPromise(
        program.pipe(Effect.provide(createPageLayers(page))),
      )
      await expect(page).toHaveURL('/dashboard/roles')
    })

    test('should display roles list', async ({ page }) => {
      const program = Effect.gen(function* () {
        const rolesPageService = yield* RolesPage

        const container = yield* rolesPageService.container()
        const pageTitle = yield* rolesPageService.pageTitle()
        const list = yield* rolesPageService.list()

        yield* ExpectHelpers.toBeVisible(container)
        yield* ExpectHelpers.toBeVisible(pageTitle)
        yield* ExpectHelpers.toBeVisible(list)
      })

      await Effect.runPromise(
        program.pipe(Effect.provide(createPageLayers(page))),
      )
    })

    test('should view role details (read-only)', async ({ page }) => {
      const program = Effect.gen(function* () {
        const rolesPageService = yield* RolesPage

        const list = yield* rolesPageService.list()
        const firstRow = list.locator('[data-testid^="role-row-"]').first()
        yield* LocatorHelpers.click(firstRow)

        const detailsDialog = yield* rolesPageService.detailsDialog()
        yield* ExpectHelpers.toBeVisible(detailsDialog)

        const editButton = page.locator('[data-testid="role-edit-button"]')
        const deleteButton = page.locator('[data-testid="role-delete-button"]')
        yield* ExpectHelpers.notToBeVisible(editButton)
        yield* ExpectHelpers.notToBeVisible(deleteButton)
      })

      await Effect.runPromise(
        program.pipe(Effect.provide(createPageLayers(page))),
      )
    })

    test('should not show create button', async ({ page }) => {
      const program = Effect.gen(function* () {
        const rolesPageService = yield* RolesPage

        const createButton = yield* rolesPageService.createButton()
        yield* ExpectHelpers.notToBeVisible(createButton)
      })

      await Effect.runPromise(
        program.pipe(Effect.provide(createPageLayers(page))),
      )
    })
  })

  test.describe('2.5 Permissions Page', () => {
    test.beforeEach(async ({ page }) => {
      await page.goto('/authentication/login')
      const loginProgram = Effect.gen(function* () {
        const loginPageService = yield* LoginPage
        yield* loginPageService.login(
          SEED_USERS.viewer.email,
          SEED_USERS.viewer.password,
        )
      })
      await Effect.runPromise(
        loginProgram.pipe(Effect.provide(createPageLayers(page))),
      )
      await page.goto('/dashboard')
      const program = Effect.gen(function* () {
        const dashboardPageService = yield* DashboardPage
        yield* dashboardPageService.clickPermissionsLink()
      })
      await Effect.runPromise(
        program.pipe(Effect.provide(createPageLayers(page))),
      )
      await expect(page).toHaveURL('/dashboard/roles-and-permissions')
    })

    test('should display permissions list', async ({ page }) => {
      const program = Effect.gen(function* () {
        const permissionsPageService = yield* PermissionsPage

        const container = yield* permissionsPageService.container()
        const pageTitle = yield* permissionsPageService.pageTitle()
        const list = yield* permissionsPageService.list()

        yield* ExpectHelpers.toBeVisible(container)
        yield* ExpectHelpers.toBeVisible(pageTitle)
        yield* ExpectHelpers.toBeVisible(list)
      })

      await Effect.runPromise(
        program.pipe(Effect.provide(createPageLayers(page))),
      )
    })

    test('should not show add button', async ({ page }) => {
      const program = Effect.gen(function* () {
        const permissionsPageService = yield* PermissionsPage

        const addButton = yield* permissionsPageService.addButton()
        yield* ExpectHelpers.notToBeVisible(addButton)
      })

      await Effect.runPromise(
        program.pipe(Effect.provide(createPageLayers(page))),
      )
    })
  })

  test.describe('2.6 Audit Log Page', () => {
    test.beforeEach(async ({ page }) => {
      await page.goto('/authentication/login')
      const loginProgram = Effect.gen(function* () {
        const loginPageService = yield* LoginPage
        yield* loginPageService.login(
          SEED_USERS.viewer.email,
          SEED_USERS.viewer.password,
        )
      })
      await Effect.runPromise(
        loginProgram.pipe(Effect.provide(createPageLayers(page))),
      )
      await page.goto('/dashboard')
      const program = Effect.gen(function* () {
        const dashboardPageService = yield* DashboardPage
        yield* dashboardPageService.clickAuditLogLink()
      })
      await Effect.runPromise(
        program.pipe(Effect.provide(createPageLayers(page))),
      )
      await expect(page).toHaveURL('/dashboard/audit-log')
    })

    test('should display audit log list', async ({ page }) => {
      const program = Effect.gen(function* () {
        const auditLogPageService = yield* AuditLogPage

        const container = yield* auditLogPageService.container()
        const pageTitle = yield* auditLogPageService.pageTitle()
        const list = yield* auditLogPageService.list()

        yield* ExpectHelpers.toBeVisible(container)
        yield* ExpectHelpers.toBeVisible(pageTitle)
        yield* ExpectHelpers.toBeVisible(list)
      })

      await Effect.runPromise(
        program.pipe(Effect.provide(createPageLayers(page))),
      )
    })

    test('should filter audit log', async ({ page }) => {
      const program = Effect.gen(function* () {
        const auditLogPageService = yield* AuditLogPage

        yield* auditLogPageService.filter('test')

        const list = yield* auditLogPageService.list()
        yield* Effect.promise(() => expect(list).toBeVisible())
      })

      await Effect.runPromise(
        program.pipe(Effect.provide(createPageLayers(page))),
      )
    })

    test('should paginate audit log', async ({ page }) => {
      const program = Effect.gen(function* () {
        const auditLogPageService = yield* AuditLogPage

        const pagination = yield* auditLogPageService.pagination()
        yield* ExpectHelpers.toBeVisible(pagination)

        yield* auditLogPageService.goToNextPage()

        yield* PageHelpers.waitForURL(page, /page=/, { timeout: 5000 })
      })

      await Effect.runPromise(
        program.pipe(Effect.provide(createPageLayers(page))),
      )
    })

    test('should view audit entry details', async ({ page }) => {
      const program = Effect.gen(function* () {
        const auditLogPageService = yield* AuditLogPage

        const list = yield* auditLogPageService.list()
        const firstRow = list.locator('[data-testid^="audit-log-row-"]').first()
        yield* LocatorHelpers.click(firstRow)

        const detailsDialog = yield* auditLogPageService.detailsDialog()
        yield* ExpectHelpers.toBeVisible(detailsDialog)
      })

      await Effect.runPromise(
        program.pipe(Effect.provide(createPageLayers(page))),
      )
    })
  })

  test.describe('2.7 Organizations Page (Blocked)', () => {
    test('should show 403 when accessing organizations page', async ({
      page,
    }) => {
      await page.goto('/authentication/login')
      const loginProgram = Effect.gen(function* () {
        const loginPageService = yield* LoginPage
        yield* loginPageService.login(
          SEED_USERS.viewer.email,
          SEED_USERS.viewer.password,
        )
      })
      await Effect.runPromise(
        loginProgram.pipe(Effect.provide(createPageLayers(page))),
      )
      await page.goto('/dashboard/organizations')

      // Should show 403 error or redirect
      const error403 = page.locator('[data-testid="error-403"]')
      await expect(
        error403
          .or(page.locator('text=403'))
          .or(page.locator('text=Forbidden')),
      ).toBeVisible()
    })
  })

  test.describe('2.8 Profile Management', () => {
    test.beforeEach(async ({ page }) => {
      await page.goto('/authentication/login')
      const loginProgram = Effect.gen(function* () {
        const loginPageService = yield* LoginPage
        yield* loginPageService.login(
          SEED_USERS.viewer.email,
          SEED_USERS.viewer.password,
        )
      })
      await Effect.runPromise(
        loginProgram.pipe(Effect.provide(createPageLayers(page))),
      )
      await page.goto('/scope')
    })

    test('should display current profile', async ({ page }) => {
      const program = Effect.gen(function* () {
        const profilePageService = yield* ProfilePage

        const container = yield* profilePageService.container()
        const pageTitle = yield* profilePageService.pageTitle()
        const currentProfile = yield* profilePageService.currentProfile()
        const orgName = yield* profilePageService.currentProfileOrgName()
        const roleName = yield* profilePageService.currentProfileRoleName()

        yield* Effect.promise(() => expect(container).toBeVisible())
        yield* Effect.promise(() => expect(pageTitle).toBeVisible())
        yield* ExpectHelpers.toBeVisible(currentProfile)
        yield* ExpectHelpers.toBeVisible(orgName)
        yield* ExpectHelpers.toBeVisible(roleName)
      })

      await Effect.runPromise(
        program.pipe(Effect.provide(createPageLayers(page))),
      )
    })

    test('should view profile details (read-only)', async ({ page }) => {
      const program = Effect.gen(function* () {
        const profilePageService = yield* ProfilePage

        const profileDetails = yield* profilePageService.profileDetails()
        yield* ExpectHelpers.toBeVisible(profileDetails)
      })

      await Effect.runPromise(
        program.pipe(Effect.provide(createPageLayers(page))),
      )
    })
  })

  test.describe('2.9 API Token Management', () => {
    test('POST /api/auth/tokens returns 201 Created', async ({ page }) => {
      await page.goto('/authentication/login')
      const loginProgram = Effect.gen(function* () {
        const loginPageService = yield* LoginPage
        yield* loginPageService.login(
          SEED_USERS.viewer.email,
          SEED_USERS.viewer.password,
        )
      })
      await Effect.runPromise(
        loginProgram.pipe(Effect.provide(createPageLayers(page))),
      )
      const response = await page.request.post(`${API_BASE_URL}/api/auth/tokens`)
      expect(response.status()).toBe(201)

      const data = await response.json()
      expect(data).toHaveProperty('token')
    })

    test('use token to GET /api/auth/me returns 200 OK', async ({
      page,
    }) => {
      await page.goto('/authentication/login')
      const loginProgram = Effect.gen(function* () {
        const loginPageService = yield* LoginPage
        yield* loginPageService.login(
          SEED_USERS.viewer.email,
          SEED_USERS.viewer.password,
        )
      })
      await Effect.runPromise(
        loginProgram.pipe(Effect.provide(createPageLayers(page))),
      )
      // Create token
      const tokenResponse = await page.request.post(
        `${API_BASE_URL}/api/auth/tokens`,
      )
      const { token } = await tokenResponse.json()

      // Use token
      const meResponse = await page.request.get(`${API_BASE_URL}/api/auth/me`, {
        headers: { Authorization: `Bearer ${token}` },
      })
      expect(meResponse.status()).toBe(200)
    })
  })

  test.describe('2.10 Generic Entity API (Read-only)', () => {
    test.beforeEach(async ({ page }) => {
      await page.goto('/authentication/login')
      const loginProgram = Effect.gen(function* () {
        const loginPageService = yield* LoginPage
        yield* loginPageService.login(
          SEED_USERS.viewer.email,
          SEED_USERS.viewer.password,
        )
      })
      await Effect.runPromise(
        loginProgram.pipe(Effect.provide(createPageLayers(page))),
      )
    })

    test('GET /api/entities/user returns 200 OK', async ({ page }) => {
      const response = await page.request.get(`${API_BASE_URL}/api/entities/user`)
      expect(response.status()).toBe(200)
    })

    test('GET /api/entities/role returns 200 OK', async ({ page }) => {
      const response = await page.request.get(`${API_BASE_URL}/api/entities/role`)
      expect(response.status()).toBe(200)
    })

    test('GET /api/entities/permission returns 200 OK', async ({ page }) => {
      const response = await page.request.get(
        `${API_BASE_URL}/api/entities/permission`,
      )
      expect(response.status()).toBe(200)
    })

    test('GET /api/entities/audit returns 200 OK', async ({ page }) => {
      const response = await page.request.get(`${API_BASE_URL}/api/entities/audit`)
      expect(response.status()).toBe(200)
    })

    test('GET /api/entities/organization returns 403 Forbidden', async ({
      page,
    }) => {
      const response = await page.request.get(
        `${API_BASE_URL}/api/entities/organization`,
      )
      expect(response.status()).toBe(403)
    })

    test('POST /api/entities/organization returns 403 Forbidden', async ({
      page,
    }) => {
      const response = await page.request.post(
        `${API_BASE_URL}/api/entities/organization`,
        {
          data: { name: 'Test Org' },
        },
      )
      expect(response.status()).toBe(403)
    })
  })

  test.describe('2.11 RPC API (Read-only)', () => {
    test.beforeEach(async ({ page }) => {
      await page.goto('/authentication/login')
      const loginProgram = Effect.gen(function* () {
        const loginPageService = yield* LoginPage
        yield* loginPageService.login(
          SEED_USERS.viewer.email,
          SEED_USERS.viewer.password,
        )
      })
      await Effect.runPromise(
        loginProgram.pipe(Effect.provide(createPageLayers(page))),
      )
    })

    test('POST /api/rpc entity.list returns 200 OK', async ({ page }) => {
      const response = await page.request.post(`${API_BASE_URL}/api/rpc`, {
        data: { method: 'entity.list', params: { entity: 'user' } },
      })
      expect(response.status()).toBe(200)
    })

    test('POST /api/rpc entity.get returns 200 OK', async ({ page }) => {
      // First get a user ID
      const listResponse = await page.request.post(`${API_BASE_URL}/api/rpc`, {
        data: { method: 'entity.list', params: { entity: 'user' } },
      })
      const listData = await listResponse.json()
      if (listData.result && listData.result.length > 0) {
        const userId = listData.result[0].id
        const response = await page.request.post(`${API_BASE_URL}/api/rpc`, {
          data: {
            method: 'entity.get',
            params: { entity: 'user', id: userId },
          },
        })
        expect(response.status()).toBe(200)
      }
    })

    test('POST /api/rpc entity.create returns 403 Forbidden', async ({
      page,
    }) => {
      const response = await page.request.post(`${API_BASE_URL}/api/rpc`, {
        data: { method: 'entity.create', params: { entity: 'user', data: {} } },
      })
      expect(response.status()).toBe(403)
    })
  })

  test.describe('2.12 Subscription Stream', () => {
    test('GET /api/subscriptions/stream establishes SSE connection', async ({
      page,
    }) => {
      await page.goto('/authentication/login')
      const loginProgram = Effect.gen(function* () {
        const loginPageService = yield* LoginPage
        yield* loginPageService.login(
          SEED_USERS.viewer.email,
          SEED_USERS.viewer.password,
        )
      })
      await Effect.runPromise(
        loginProgram.pipe(Effect.provide(createPageLayers(page))),
      )
      const response = await page.request.get(
        `${API_BASE_URL}/api/subscriptions/stream`,
        {
          headers: { Accept: 'text/event-stream' },
        },
      )
      expect(response.status()).toBe(200)
      expect(response.headers()['content-type']).toContain('text/event-stream')
    })
  })

  test.describe('2.13 Admin Endpoint (Blocked)', () => {
    test('GET /api/auth/admin returns 403 Forbidden', async ({ page }) => {
      await page.goto('/authentication/login')
      const loginProgram = Effect.gen(function* () {
        const loginPageService = yield* LoginPage
        yield* loginPageService.login(
          SEED_USERS.viewer.email,
          SEED_USERS.viewer.password,
        )
      })
      await Effect.runPromise(
        loginProgram.pipe(Effect.provide(createPageLayers(page))),
      )
      const response = await page.request.get(`${API_BASE_URL}/api/auth/admin`)
      expect(response.status()).toBe(403)
    })
  })

  test.describe('2.14 Logout', () => {
    test('logout redirects to login', async ({ page }) => {
      await page.goto('/authentication/login')
      const loginProgram = Effect.gen(function* () {
        const loginPageService = yield* LoginPage
        yield* loginPageService.login(
          SEED_USERS.viewer.email,
          SEED_USERS.viewer.password,
        )
      })
      await Effect.runPromise(
        loginProgram.pipe(Effect.provide(createPageLayers(page))),
      )
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

    test('after logout protected routes redirect to login', async ({
      page,
    }) => {
      await page.goto('/authentication/login')
      const loginProgram = Effect.gen(function* () {
        const loginPageService = yield* LoginPage
        yield* loginPageService.login(
          SEED_USERS.viewer.email,
          SEED_USERS.viewer.password,
        )
      })
      await Effect.runPromise(
        loginProgram.pipe(Effect.provide(createPageLayers(page))),
      )
      await page.goto('/dashboard')

      const program = Effect.gen(function* () {
        const dashboardPageService = yield* DashboardPage
        const loginPageService = yield* LoginPage

        yield* dashboardPageService.logout()

        yield* PageHelpers.goto(page, '/dashboard')
        yield* ExpectHelpers.toHaveURL(page, '/authentication/login')
        const loginContainer = yield* loginPageService.container()
        yield* Effect.promise(() => expect(loginContainer).toBeVisible())
      })

      await Effect.runPromise(
        program.pipe(Effect.provide(createPageLayers(page))),
      )
    })
  })
})
