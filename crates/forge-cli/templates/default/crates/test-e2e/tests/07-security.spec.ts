/**
 * E2E tests for Cross-Role Scenarios
 *
 * Tests follow the DAG structure from E2E_TEST_SCENARIOS.md:
 * - 7.1 Multi-Organization User Flow
 * - 7.2 Permission Escalation Prevention
 * - 7.3 Concurrent Operations
 * - 7.4 Error Handling
 * - 7.5 Session Management
 * - 7.6 Data Consistency
 */
import { test, expect } from '@playwright/test'
import { Effect } from 'effect'
import { LoginPage, DashboardPage, SelectScopePage, UsersPage } from '@/pages'
import { SEED_USERS, createTestUser } from '@/fixtures/test-data'
import { createPageLayers } from '@/fixtures/page-layers'
import { API_BASE_URL } from '@/playwright.config'
import * as ExpectHelpers from '@/helpers/expect'
import * as LocatorHelpers from '@/helpers/locator'
import * as PageHelpers from '@/helpers/page'

test.describe('Cross-Role Scenarios', () => {
  test.describe('7.1 Multi-Organization User Flow', () => {
    test('should switch between organizations', async ({ page }) => {
      await page.goto('/authentication/login')

      const program = Effect.gen(function* () {
        const loginPageService = yield* LoginPage
        const selectScopePageService = yield* SelectScopePage
        const dashboardPageService = yield* DashboardPage
        const usersPageService = yield* UsersPage

        // Login with multi-profile user
        yield* loginPageService.login(
          SEED_USERS.multiProfile.email,
          SEED_USERS.multiProfile.password,
        )

        yield* ExpectHelpers.toHaveURL(page, '/authentication/select-scope')

        const profileList = yield* selectScopePageService.profileList()
        yield* ExpectHelpers.toBeVisible(profileList)

        // Select first profile
        yield* selectScopePageService.selectProfile(0)

        yield* ExpectHelpers.toHaveURL(page, '/dashboard')

        // Navigate to users
        yield* dashboardPageService.clickUsersLink()
        const usersList1 = yield* usersPageService.list()
        yield* ExpectHelpers.toBeVisible(usersList1)

        // Switch to different profile
        yield* PageHelpers.goto(page, '/scope')
        yield* selectScopePageService.selectProfile(1)

        yield* PageHelpers.goto(page, '/dashboard/users')
        const usersList2 = yield* usersPageService.list()
        yield* ExpectHelpers.toBeVisible(usersList2)
      })

      await Effect.runPromise(
        program.pipe(Effect.provide(createPageLayers(page))),
      )
    })
  })

  test.describe('7.2 Permission Escalation Prevention', () => {
    test.describe('Viewer permissions', () => {
      test('Viewer cannot access editor-only features', async ({ page }) => {
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
        await page.goto('/dashboard/users')

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

    test.describe('Editor permissions', () => {
      test('Editor cannot access org-admin-only features', async ({ page }) => {
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

        const error403 = page.locator('[data-testid="error-403"]')
        await expect(
          error403
            .or(page.locator('text=403'))
            .or(page.locator('text=Forbidden')),
        ).toBeVisible()
      })
    })

    test.describe('Org Admin permissions', () => {
      test('Org Admin cannot access global-admin-only features', async ({
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
          `${API_BASE_URL}/api/auth/admin`,
        )
        expect(response.status()).toBe(403)
      })
    })

    test.describe('App Admin permissions', () => {
      test('App Admin can access global-admin features', async ({ page }) => {
        await page.goto('/authentication/login')
        const loginProgram = Effect.gen(function* () {
          const loginPageService = yield* LoginPage
          yield* loginPageService.login(
            SEED_USERS.appAdmin.email,
            SEED_USERS.appAdmin.password,
          )
        })
        await Effect.runPromise(
          loginProgram.pipe(Effect.provide(createPageLayers(page))),
        )
        const response = await page.request.get(
          `${API_BASE_URL}/api/auth/admin`,
        )
        expect(response.status()).toBe(200)
      })
    })
  })

  test.describe('7.3 Concurrent Operations', () => {
    test('subscription stream receives invalidation events', async ({
      page,
      context,
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
      await page.goto('/dashboard/users')

      // Create a second page for subscription stream
      const page2 = await context.newPage()
      await page2.goto('/authentication/login')
      const loginProgram2 = Effect.gen(function* () {
        const loginPageService = yield* LoginPage
        yield* loginPageService.login(
          SEED_USERS.editor.email,
          SEED_USERS.editor.password,
        )
      })
      await Effect.runPromise(
        loginProgram2.pipe(Effect.provide(createPageLayers(page2))),
      )
      await page2.goto('/dashboard/users')

      const testUser = createTestUser()

      // Create user in first page
      const program1 = Effect.gen(function* () {
        const usersPageService = yield* UsersPage
        yield* usersPageService.createUser(testUser.email, testUser.password)
      })

      await Effect.runPromise(
        program1.pipe(Effect.provide(createPageLayers(page))),
      )

      // Verify second page sees the update (via subscription stream)
      const program2 = Effect.gen(function* () {
        const usersPageService = yield* UsersPage
        const row = yield* usersPageService.row(testUser.email)
        yield* ExpectHelpers.toBeVisible(row)
      })

      await Effect.runPromise(
        program2.pipe(Effect.provide(createPageLayers(page2))),
      )

      await page2.close()
    })
  })

  test.describe('7.4 Error Handling', () => {
    test('invalid API request returns 400 Bad Request', async ({ request }) => {
      // No auth needed for this test
      const response = await request.post(`${API_BASE_URL}/api/entities/user`, {
        data: { invalid: 'data' },
      })
      expect(response.status()).toBe(400)
    })

    test('unauthorized request returns 401 Unauthorized', async ({
      request,
    }) => {
      const response = await request.get(`${API_BASE_URL}/api/auth/me`)
      expect(response.status()).toBe(401)
    })

    test.describe('with Viewer auth', () => {
      test('forbidden request returns 403 Forbidden', async ({ page }) => {
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
        const response = await page.request.post(
          `${API_BASE_URL}/api/entities/user`,
          {
            data: { email: 'test@example.com', password: 'password' },
          },
        )
        expect(response.status()).toBe(403)
      })

      test('not found returns 404 Not Found', async ({ page }) => {
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
          `${API_BASE_URL}/api/entities/user/invalid-id`,
        )
        expect(response.status()).toBe(404)
      })
    })
  })

  test.describe('7.5 Session Management', () => {
    test('logout invalidates token', async ({ page, context }) => {
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
        yield* dashboardPageService.logout()
      })

      await Effect.runPromise(
        program.pipe(Effect.provide(createPageLayers(page))),
      )

      // Try to access protected route
      await page.goto('/dashboard')
      await expect(page).toHaveURL('/authentication/login')
    })
  })

  test.describe('7.6 Data Consistency', () => {
    test('created entity appears in list', async ({ page }) => {
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
})
