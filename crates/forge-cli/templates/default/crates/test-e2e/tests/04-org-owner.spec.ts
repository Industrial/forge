/**
 * E2E tests for Org Owner Role
 *
 * Tests follow the DAG structure from E2E_TEST_SCENARIOS.md:
 * - 4.1 Authentication & Profile Selection
 * - 4.2 Dashboard Access (Full)
 * - 4.3 Organizations Page (Full CRUD)
 * - 4.4 Users Page (Full CRUD)
 * - 4.5 Roles Page (Full CRUD)
 * - 4.6 Permissions Page (Full CRUD)
 * - 4.7 Audit Log Page
 * - 4.8 Profile Management
 * - 4.9 Generic Entity API (Full CRUD - Org-scoped)
 * - 4.10 RPC API (Full CRUD)
 * - 4.11 Subscription Stream
 * - 4.12 Admin Endpoint (Blocked)
 * - 4.13 Scope Switching
 * - 4.14 Logout
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
import * as LocatorHelpers from '@/helpers/locator'
import * as PageHelpers from '@/helpers/page'

test.describe('Org Owner Role', () => {
  test.describe('4.1 Authentication & Profile Selection', () => {
    test('single profile redirects to dashboard', async ({ page }) => {
      await page.goto('/authentication/login')

      const program = Effect.gen(function* () {
        const loginPageService = yield* LoginPage
        const dashboardPageService = yield* DashboardPage

        yield* loginPageService.login(
          SEED_USERS.orgOwner.email,
          SEED_USERS.orgOwner.password,
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

  test.describe('4.2 Dashboard Access (Full)', () => {
    test.beforeEach(async ({ page }) => {
      await page.goto('/authentication/login')
      const loginProgram = Effect.gen(function* () {
        const loginPageService = yield* LoginPage
        yield* loginPageService.login(
          SEED_USERS.orgOwner.email,
          SEED_USERS.orgOwner.password,
        )
      })
      await Effect.runPromise(
        loginProgram.pipe(Effect.provide(createPageLayers(page))),
      )
      await page.goto('/dashboard')
    })

    test('should show all navigation links', async ({ page }) => {
      const program = Effect.gen(function* () {
        const dashboardPageService = yield* DashboardPage

        const dashboardLink = yield* dashboardPageService.dashboardLink()
        const organizationsLink =
          yield* dashboardPageService.organizationsLink()
        const usersLink = yield* dashboardPageService.usersLink()
        const rolesLink = yield* dashboardPageService.rolesLink()
        const permissionsLink = yield* dashboardPageService.permissionsLink()
        const auditLogLink = yield* dashboardPageService.auditLogLink()

        yield* ExpectHelpers.toBeVisible(dashboardLink)
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

  test.describe('4.3 Organizations Page (Full CRUD)', () => {
    test.beforeEach(async ({ page }) => {
      await page.goto('/authentication/login')
      const loginProgram = Effect.gen(function* () {
        const loginPageService = yield* LoginPage
        yield* loginPageService.login(
          SEED_USERS.orgOwner.email,
          SEED_USERS.orgOwner.password,
        )
      })
      await Effect.runPromise(
        loginProgram.pipe(Effect.provide(createPageLayers(page))),
      )
      await page.goto('/dashboard/organizations')
    })

    test('should create organization', async ({ page }) => {
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

    test('should update organization', async ({ page }) => {
      const testOrg = createTestOrganization()
      const newName = `updated-${testOrg.name}`

      const program = Effect.gen(function* () {
        const organizationsPageService = yield* OrganizationsPage

        yield* organizationsPageService.createOrganization(
          testOrg.name,
          testOrg.slug,
        )
        yield* organizationsPageService.updateOrganization(
          testOrg.name,
          newName,
        )

        const updatedRow = yield* organizationsPageService.row(newName)
        yield* ExpectHelpers.toBeVisible(updatedRow)
      })

      await Effect.runPromise(
        program.pipe(Effect.provide(createPageLayers(page))),
      )
    })

    test('should delete organization', async ({ page }) => {
      const testOrg = createTestOrganization()

      const program = Effect.gen(function* () {
        const organizationsPageService = yield* OrganizationsPage

        yield* organizationsPageService.createOrganization(
          testOrg.name,
          testOrg.slug,
        )
        yield* organizationsPageService.deleteOrganization(testOrg.name)

        const row = yield* organizationsPageService.row(testOrg.name)
        yield* ExpectHelpers.notToBeVisible(row)
      })

      await Effect.runPromise(
        program.pipe(Effect.provide(createPageLayers(page))),
      )
    })

    test('should filter organizations', async ({ page }) => {
      const program = Effect.gen(function* () {
        const organizationsPageService = yield* OrganizationsPage

        yield* organizationsPageService.filter('test')

        const list = yield* organizationsPageService.list()
        yield* ExpectHelpers.toBeVisible(list)
      })

      await Effect.runPromise(
        program.pipe(Effect.provide(createPageLayers(page))),
      )
    })
  })

  test.describe('4.4 Users Page (Full CRUD)', () => {
    test.beforeEach(async ({ page }) => {
      await page.goto('/authentication/login')
      const loginProgram = Effect.gen(function* () {
        const loginPageService = yield* LoginPage
        yield* loginPageService.login(
          SEED_USERS.orgOwner.email,
          SEED_USERS.orgOwner.password,
        )
      })
      await Effect.runPromise(
        loginProgram.pipe(Effect.provide(createPageLayers(page))),
      )
      await page.goto('/dashboard/users')
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

        yield* usersPageService.createUser(testUser.email, testUser.password)
        yield* usersPageService.updateUser(testUser.email, newEmail)

        const updatedRow = yield* usersPageService.row(newEmail)
        yield* ExpectHelpers.toBeVisible(updatedRow)
      })

      await Effect.runPromise(
        program.pipe(Effect.provide(createPageLayers(page))),
      )
    })

    test('should delete user', async ({ page }) => {
      const testUser = createTestUser()

      const program = Effect.gen(function* () {
        const usersPageService = yield* UsersPage

        yield* usersPageService.createUser(testUser.email, testUser.password)
        yield* usersPageService.deleteUser(testUser.email)

        const row = yield* usersPageService.row(testUser.email)
        yield* ExpectHelpers.notToBeVisible(row)
      })

      await Effect.runPromise(
        program.pipe(Effect.provide(createPageLayers(page))),
      )
    })
  })

  test.describe('4.5 Roles Page (Full CRUD)', () => {
    test.beforeEach(async ({ page }) => {
      await page.goto('/authentication/login')
      const loginProgram = Effect.gen(function* () {
        const loginPageService = yield* LoginPage
        yield* loginPageService.login(
          SEED_USERS.orgOwner.email,
          SEED_USERS.orgOwner.password,
        )
      })
      await Effect.runPromise(
        loginProgram.pipe(Effect.provide(createPageLayers(page))),
      )
      await page.goto('/dashboard/roles')
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

    test('should delete role', async ({ page }) => {
      const roleName = `test-role-${Date.now()}`

      const program = Effect.gen(function* () {
        const rolesPageService = yield* RolesPage

        yield* rolesPageService.createRole(roleName)
        yield* rolesPageService.deleteRole(roleName)

        const row = yield* rolesPageService.row(roleName)
        yield* ExpectHelpers.notToBeVisible(row)
      })

      await Effect.runPromise(
        program.pipe(Effect.provide(createPageLayers(page))),
      )
    })
  })

  test.describe('4.6 Permissions Page (Full CRUD)', () => {
    test.beforeEach(async ({ page }) => {
      await page.goto('/authentication/login')
      const loginProgram = Effect.gen(function* () {
        const loginPageService = yield* LoginPage
        yield* loginPageService.login(
          SEED_USERS.orgOwner.email,
          SEED_USERS.orgOwner.password,
        )
      })
      await Effect.runPromise(
        loginProgram.pipe(Effect.provide(createPageLayers(page))),
      )
      await page.goto('/dashboard/roles-and-permissions')
    })

    test('should add permission to role', async ({ page }) => {
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

    test('should remove permission from role', async ({ page }) => {
      const program = Effect.gen(function* () {
        const permissionsPageService = yield* PermissionsPage

        // Add first
        yield* permissionsPageService.addPermission('viewer', 'user.read')

        // Then remove
        yield* permissionsPageService.removePermission('viewer', 'user.read')

        const assignmentRow = yield* permissionsPageService.assignmentRow(
          'viewer',
          'user.read',
        )
        yield* ExpectHelpers.notToBeVisible(assignmentRow)
      })

      await Effect.runPromise(
        program.pipe(Effect.provide(createPageLayers(page))),
      )
    })
  })

  test.describe('4.7 Audit Log Page', () => {
    test.beforeEach(async ({ page }) => {
      await page.goto('/authentication/login')
      const loginProgram = Effect.gen(function* () {
        const loginPageService = yield* LoginPage
        yield* loginPageService.login(
          SEED_USERS.orgOwner.email,
          SEED_USERS.orgOwner.password,
        )
      })
      await Effect.runPromise(
        loginProgram.pipe(Effect.provide(createPageLayers(page))),
      )
      await page.goto('/dashboard/audit-log')
    })

    test('should display audit log list', async ({ page }) => {
      const program = Effect.gen(function* () {
        const auditLogPageService = yield* AuditLogPage

        const container = yield* auditLogPageService.container()
        const list = yield* auditLogPageService.list()

        yield* Effect.promise(() => expect(container).toBeVisible())
        yield* ExpectHelpers.toBeVisible(list)
      })

      await Effect.runPromise(
        program.pipe(Effect.provide(createPageLayers(page))),
      )
    })
  })

  test.describe('4.8 Profile Management', () => {
    test('should display current profile', async ({ page }) => {
      await page.goto('/authentication/login')
      const loginProgram = Effect.gen(function* () {
        const loginPageService = yield* LoginPage
        yield* loginPageService.login(
          SEED_USERS.orgOwner.email,
          SEED_USERS.orgOwner.password,
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

  test.describe('4.9 Generic Entity API (Full CRUD)', () => {
    test('POST /api/entities/organization returns 201 Created', async ({
      page,
    }) => {
      await page.goto('/authentication/login')
      const loginProgram = Effect.gen(function* () {
        const loginPageService = yield* LoginPage
        yield* loginPageService.login(
          SEED_USERS.orgOwner.email,
          SEED_USERS.orgOwner.password,
        )
      })
      await Effect.runPromise(
        loginProgram.pipe(Effect.provide(createPageLayers(page))),
      )
      const testOrg = createTestOrganization()
      const response = await page.request.post(
        `${API_BASE_URL}/api/entities/organization`,
        {
          data: { name: testOrg.name, slug: testOrg.slug },
        },
      )
      expect(response.status()).toBe(201)
    })

    test('DELETE /api/entities/organization/{id} returns 200 OK', async ({
      page,
    }) => {
      await page.goto('/authentication/login')
      const loginProgram = Effect.gen(function* () {
        const loginPageService = yield* LoginPage
        yield* loginPageService.login(
          SEED_USERS.orgOwner.email,
          SEED_USERS.orgOwner.password,
        )
      })
      await Effect.runPromise(
        loginProgram.pipe(Effect.provide(createPageLayers(page))),
      )
      const testOrg = createTestOrganization()
      const createResponse = await page.request.post(
        `${API_BASE_URL}/api/entities/organization`,
        {
          data: { name: testOrg.name, slug: testOrg.slug },
        },
      )
      const created = await createResponse.json()

      const deleteResponse = await page.request.delete(
        `${API_BASE_URL}/api/entities/organization/${created.id}`,
      )
      expect(deleteResponse.status()).toBe(200)
    })
  })

  test.describe('4.10 RPC API (Full CRUD)', () => {
    test('POST /api/rpc entity.delete returns 200 OK', async ({ page }) => {
      await page.goto('/authentication/login')
      const loginProgram = Effect.gen(function* () {
        const loginPageService = yield* LoginPage
        yield* loginPageService.login(
          SEED_USERS.orgOwner.email,
          SEED_USERS.orgOwner.password,
        )
      })
      await Effect.runPromise(
        loginProgram.pipe(Effect.provide(createPageLayers(page))),
      )
      const testUser = createTestUser()
      const createResponse = await page.request.post(`${API_BASE_URL}/api/rpc`, {
        data: {
          method: 'entity.create',
          params: {
            entity: 'user',
            data: { email: testUser.email, password: testUser.password },
          },
        },
      })
      const created = await createResponse.json()

      const deleteResponse = await page.request.post(`${API_BASE_URL}/api/rpc`, {
        data: {
          method: 'entity.delete',
          params: { entity: 'user', id: created.result.id },
        },
      })
      expect(deleteResponse.status()).toBe(200)
    })
  })

  test.describe('4.11 Subscription Stream', () => {
    test('GET /api/subscriptions/stream establishes SSE connection', async ({
      page,
    }) => {
      await page.goto('/authentication/login')
      const loginProgram = Effect.gen(function* () {
        const loginPageService = yield* LoginPage
        yield* loginPageService.login(
          SEED_USERS.orgOwner.email,
          SEED_USERS.orgOwner.password,
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
    })
  })

  test.describe('4.12 Admin Endpoint (Blocked)', () => {
    test('GET /api/auth/admin returns 403 Forbidden', async ({ page }) => {
      await page.goto('/authentication/login')
      const loginProgram = Effect.gen(function* () {
        const loginPageService = yield* LoginPage
        yield* loginPageService.login(
          SEED_USERS.orgOwner.email,
          SEED_USERS.orgOwner.password,
        )
      })
      await Effect.runPromise(
        loginProgram.pipe(Effect.provide(createPageLayers(page))),
      )
      const response = await page.request.get(`${API_BASE_URL}/api/auth/admin`)
      expect(response.status()).toBe(403)
    })
  })

  test.describe('4.13 Scope Switching', () => {
    test('should switch profile and see updated data', async ({ page }) => {
      await page.goto('/authentication/login')
      const loginProgram = Effect.gen(function* () {
        const loginPageService = yield* LoginPage
        yield* loginPageService.login(
          SEED_USERS.orgOwner.email,
          SEED_USERS.orgOwner.password,
        )
      })
      await Effect.runPromise(
        loginProgram.pipe(Effect.provide(createPageLayers(page))),
      )
      await page.goto('/scope')

      const program = Effect.gen(function* () {
        const profilePageService = yield* ProfilePage
        const dashboardPageService = yield* DashboardPage
        const usersPageService = yield* UsersPage

        const profileList = yield* profilePageService.profileList()
        yield* ExpectHelpers.toBeVisible(profileList)

        // Switch to first profile
        yield* profilePageService.switchProfile(0)

        // Navigate to dashboard and verify users list
        yield* PageHelpers.goto(page, '/dashboard/users')
        const usersList = yield* usersPageService.list()
        yield* ExpectHelpers.toBeVisible(usersList)
      })

      await Effect.runPromise(
        program.pipe(Effect.provide(createPageLayers(page))),
      )
    })
  })

  test.describe('4.14 Logout', () => {
    test('logout redirects to login', async ({ page }) => {
      await page.goto('/authentication/login')
      const loginProgram = Effect.gen(function* () {
        const loginPageService = yield* LoginPage
        yield* loginPageService.login(
          SEED_USERS.orgOwner.email,
          SEED_USERS.orgOwner.password,
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
  })
})
