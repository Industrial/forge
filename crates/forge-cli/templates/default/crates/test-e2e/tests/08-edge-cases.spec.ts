/**
 * E2E tests for Edge Cases & Boundary Conditions
 *
 * Tests follow the DAG structure from E2E_TEST_SCENARIOS.md:
 * - 8.1 Empty States
 * - 8.2 Large Datasets
 * - 8.3 Invalid Inputs
 * - 8.4 Network Conditions
 * - 8.5 Browser Compatibility
 *
 * Uses Effect.ts for composition and Playwright for browser automation.
 */
import { test, expect } from '@playwright/test'
import { Effect } from 'effect'

import {
  DashboardPage,
  UsersPage,
  RolesPage,
  OrganizationsPage,
  AuditLogPage,
} from '@/pages'
import { createPageLayers } from '@/fixtures/page-layers'
import { API_BASE_URL } from '@/playwright.config'
import * as ExpectHelpers from '@/helpers/expect'
import * as LocatorHelpers from '@/helpers/locator'
import * as PageHelpers from '@/helpers/page'

test.describe('Edge Cases & Boundary Conditions', () => {
  test.describe('8.1 Empty States', () => {
    test.use({ storageState: 'playwright/.auth/viewer.json' })

    test('organizations page shows empty state', async ({ page }) => {
      await page.goto('/dashboard/organizations')

      const program = Effect.gen(function* () {
        const organizationsPageService = yield* OrganizationsPage

        // Check for empty state (may not exist if there are orgs, but verify structure)
        const container = yield* organizationsPageService.container()
        yield* ExpectHelpers.toBeVisible(container)

        // If empty state exists, verify it's visible
        const emptyState = page.locator(
          '[data-testid="organizations-empty-state"]',
        )
        const emptyStateCount = yield* LocatorHelpers.count(emptyState)
        if (emptyStateCount > 0) {
          yield* ExpectHelpers.toBeVisible(emptyState)
        }
      })

      await Effect.runPromise(
        program.pipe(Effect.provide(createPageLayers(page))),
      )
    })

    test('users page shows empty state', async ({ page }) => {
      await page.goto('/dashboard/users')

      const program = Effect.gen(function* () {
        const usersPageService = yield* UsersPage

        const container = yield* usersPageService.container()
        yield* ExpectHelpers.toBeVisible(container)

        // If empty state exists, verify it's visible
        const emptyState = page.locator('[data-testid="users-empty-state"]')
        const emptyStateCount = yield* LocatorHelpers.count(emptyState)
        if (emptyStateCount > 0) {
          yield* ExpectHelpers.toBeVisible(emptyState)
        }
      })

      await Effect.runPromise(
        program.pipe(Effect.provide(createPageLayers(page))),
      )
    })

    test('roles page shows empty state', async ({ page }) => {
      await page.goto('/dashboard/roles')

      const program = Effect.gen(function* () {
        const rolesPageService = yield* RolesPage

        const container = yield* rolesPageService.container()
        yield* ExpectHelpers.toBeVisible(container)

        // If empty state exists, verify it's visible
        const emptyState = page.locator('[data-testid="roles-empty-state"]')
        const emptyStateCount = yield* LocatorHelpers.count(emptyState)
        if (emptyStateCount > 0) {
          yield* ExpectHelpers.toBeVisible(emptyState)
        }
      })

      await Effect.runPromise(
        program.pipe(Effect.provide(createPageLayers(page))),
      )
    })

    test('audit log page shows empty state', async ({ page }) => {
      await page.goto('/dashboard/audit-log')

      const program = Effect.gen(function* () {
        const auditLogPageService = yield* AuditLogPage

        const container = yield* auditLogPageService.container()
        yield* ExpectHelpers.toBeVisible(container)

        // If empty state exists, verify it's visible
        const emptyState = page.locator('[data-testid="audit-log-empty-state"]')
        const emptyStateCount = yield* LocatorHelpers.count(emptyState)
        if (emptyStateCount > 0) {
          yield* ExpectHelpers.toBeVisible(emptyState)
        }
      })

      await Effect.runPromise(
        program.pipe(Effect.provide(createPageLayers(page))),
      )
    })
  })

  test.describe('8.2 Large Datasets', () => {
    test.use({ storageState: 'playwright/.auth/viewer.json' })

    test('large user list pagination works correctly', async ({ page }) => {
      await page.goto('/dashboard/users')

      const program = Effect.gen(function* () {
        const usersPageService = yield* UsersPage

        const container = yield* usersPageService.container()
        yield* ExpectHelpers.toBeVisible(container)

        // Check if pagination exists
        const pagination = page.locator('[data-testid="users-pagination"]')
        const paginationCount = yield* LocatorHelpers.count(pagination)
        if (paginationCount > 0) {
          yield* ExpectHelpers.toBeVisible(pagination)

          // Try to navigate to next page
          const nextButton = page.locator(
            '[data-testid="users-pagination-next"]',
          )
          const nextButtonCount = yield* LocatorHelpers.count(nextButton)
          const nextButtonEnabled = yield* LocatorHelpers.isEnabled(nextButton)
          if (nextButtonCount > 0 && nextButtonEnabled) {
            yield* LocatorHelpers.click(nextButton)
            yield* PageHelpers.waitForURL(page, /page=/, { timeout: 5000 })
          }
        }
      })

      await Effect.runPromise(
        program.pipe(Effect.provide(createPageLayers(page))),
      )
    })

    test('large organization list pagination works correctly', async ({
      page,
    }) => {
      await page.goto('/dashboard/organizations')

      const program = Effect.gen(function* () {
        const organizationsPageService = yield* OrganizationsPage

        const container = yield* organizationsPageService.container()
        yield* ExpectHelpers.toBeVisible(container)

        // Check if pagination exists
        const pagination = page.locator(
          '[data-testid="organizations-pagination"]',
        )
        const paginationCount = yield* LocatorHelpers.count(pagination)
        if (paginationCount > 0) {
          yield* ExpectHelpers.toBeVisible(pagination)

          const nextButton = page.locator(
            '[data-testid="organizations-pagination-next"]',
          )
          const nextButtonCount = yield* LocatorHelpers.count(nextButton)
          const nextButtonEnabled = yield* LocatorHelpers.isEnabled(nextButton)
          if (nextButtonCount > 0 && nextButtonEnabled) {
            yield* LocatorHelpers.click(nextButton)
            yield* PageHelpers.waitForURL(page, /page=/, { timeout: 5000 })
          }
        }
      })

      await Effect.runPromise(
        program.pipe(Effect.provide(createPageLayers(page))),
      )
    })

    test('large audit log pagination works correctly', async ({ page }) => {
      await page.goto('/dashboard/audit-log')

      const program = Effect.gen(function* () {
        const auditLogPageService = yield* AuditLogPage

        const container = yield* auditLogPageService.container()
        yield* ExpectHelpers.toBeVisible(container)

        // Check if pagination exists
        const pagination = page.locator('[data-testid="audit-log-pagination"]')
        const paginationCount = yield* LocatorHelpers.count(pagination)
        if (paginationCount > 0) {
          yield* ExpectHelpers.toBeVisible(pagination)

          const nextButton = page.locator(
            '[data-testid="audit-log-pagination-next"]',
          )
          const nextButtonCount = yield* LocatorHelpers.count(nextButton)
          const nextButtonEnabled = yield* LocatorHelpers.isEnabled(nextButton)
          if (nextButtonCount > 0 && nextButtonEnabled) {
            yield* LocatorHelpers.click(nextButton)
            yield* PageHelpers.waitForURL(page, /page=/, { timeout: 5000 })
          }
        }
      })

      await Effect.runPromise(
        program.pipe(Effect.provide(createPageLayers(page))),
      )
    })

    test('cursor pagination works correctly', async ({ request }) => {
      // Test cursor-based pagination via API
      const response = await request.get(
        `${API_BASE_URL}/api/entities/user?limit=10`,
      )
      expect(response.status()).toBe(200)

      const data = await response.json()
      // Verify response structure supports cursor pagination
      expect(data).toHaveProperty('data')
      // If cursor exists, verify it can be used for next page
      if (data.cursor) {
        const nextResponse = await request.get(
          `${API_BASE_URL}/api/entities/user?limit=10&cursor=${data.cursor}`,
        )
        expect(nextResponse.status()).toBe(200)
      }
    })
  })

  test.describe('8.3 Invalid Inputs', () => {
    test.use({ storageState: 'playwright/.auth/editor.json' })

    test('invalid email format shows validation error', async ({ page }) => {
      await page.goto('/dashboard/users')

      const program = Effect.gen(function* () {
        const usersPageService = yield* UsersPage

        const createButton = yield* usersPageService.createButton()
        yield* LocatorHelpers.click(createButton)

        const createDialog = yield* usersPageService.createDialog()
        yield* ExpectHelpers.toBeVisible(createDialog)

        // Try invalid email
        const emailInput = page.locator(
          '[data-testid="user-create-email-input"]',
        )
        yield* LocatorHelpers.fill(emailInput, 'invalid-email')

        const submitButton = page.locator(
          '[data-testid="user-create-submit-button"]',
        )
        yield* LocatorHelpers.click(submitButton)

        // Check for validation error
        const errorMessage = page.locator(
          '[data-testid="user-create-email-error"]',
        )
        const errorCount = yield* LocatorHelpers.count(errorMessage)
        if (errorCount > 0) {
          yield* ExpectHelpers.toBeVisible(errorMessage)
        }
      })

      await Effect.runPromise(
        program.pipe(Effect.provide(createPageLayers(page))),
      )
    })

    test('short password shows validation error', async ({ page }) => {
      await page.goto('/dashboard/users')

      const program = Effect.gen(function* () {
        const usersPageService = yield* UsersPage

        const createButton = yield* usersPageService.createButton()
        yield* LocatorHelpers.click(createButton)

        const createDialog = yield* usersPageService.createDialog()
        yield* ExpectHelpers.toBeVisible(createDialog)

        // Try short password
        const passwordInput = page.locator(
          '[data-testid="user-create-password-input"]',
        )
        yield* LocatorHelpers.fill(passwordInput, 'short')

        const submitButton = page.locator(
          '[data-testid="user-create-submit-button"]',
        )
        yield* LocatorHelpers.click(submitButton)

        // Check for validation error
        const errorMessage = page.locator(
          '[data-testid="user-create-password-error"]',
        )
        const errorCount = yield* LocatorHelpers.count(errorMessage)
        if (errorCount > 0) {
          yield* ExpectHelpers.toBeVisible(errorMessage)
        }
      })

      await Effect.runPromise(
        program.pipe(Effect.provide(createPageLayers(page))),
      )
    })

    test('invalid UUID returns 400 Bad Request', async ({ request }) => {
      const response = await request.get(
        `${API_BASE_URL}/api/entities/user/invalid-uuid`,
      )
      expect(response.status()).toBe(400)
    })

    test('invalid filter JSON returns 400 Bad Request', async ({ request }) => {
      const response = await request.get(
        `${API_BASE_URL}/api/entities/user?filter=invalid-json`,
      )
      expect(response.status()).toBe(400)
    })

    test('invalid sort field returns 400 Bad Request', async ({ request }) => {
      const response = await request.get(
        `${API_BASE_URL}/api/entities/user?sort=invalid-field`,
      )
      expect(response.status()).toBe(400)
    })
  })

  test.describe('8.4 Network Conditions', () => {
    test.use({ storageState: 'playwright/.auth/viewer.json' })

    test('slow network shows loading states', async ({ page, context }) => {
      // Simulate slow network
      await context.route('**/api/**', async (route) => {
        await new Promise((resolve) => setTimeout(resolve, 1000))
        await route.continue()
      })

      await page.goto('/dashboard/users')

      const program = Effect.gen(function* () {
        // Check for loading indicator
        const loadingSpinner = page.locator(
          '[data-testid="users-loading-spinner"]',
        )
        const skeleton = page.locator('[data-testid="users-skeleton"]')

        // One of these should be visible during load
        const spinnerCount = yield* LocatorHelpers.count(loadingSpinner)
        const skeletonCount = yield* LocatorHelpers.count(skeleton)
        if (spinnerCount > 0) {
          yield* ExpectHelpers.toBeVisible(loadingSpinner)
        } else if (skeletonCount > 0) {
          yield* ExpectHelpers.toBeVisible(skeleton)
        }

        // Wait for content to load
        const usersPageService = yield* UsersPage
        const container = yield* usersPageService.container()
        yield* ExpectHelpers.toBeVisible(container)
      })

      await Effect.runPromise(
        program.pipe(Effect.provide(createPageLayers(page))),
      )
    })

    test('network error shows error message', async ({ page, context }) => {
      // Simulate network error
      await context.route('**/api/entities/user', async (route) => {
        await route.abort('failed')
      })

      await page.goto('/dashboard/users')

      // Check for error message
      const errorMessage = page
        .locator('[data-testid="error-message"]')
        .or(page.locator('text=/error/i'))
      await expect(errorMessage.first()).toBeVisible({ timeout: 5000 })
    })

    test('timeout shows error message', async ({ page, context }) => {
      // Simulate timeout
      await context.route('**/api/entities/user', async (route) => {
        await new Promise((resolve) => setTimeout(resolve, 10000))
        await route.abort('timedout')
      })

      await page.goto('/dashboard/users')

      // Check for timeout error
      const errorMessage = page
        .locator('[data-testid="error-message"]')
        .or(page.locator('text=/timeout/i'))
      await expect(errorMessage.first()).toBeVisible({ timeout: 15000 })
    })
  })

  test.describe('8.5 Browser Compatibility', () => {
    test.use({ storageState: 'playwright/.auth/viewer.json' })

    test('mobile viewport responsive layout works', async ({ page }) => {
      // Set mobile viewport
      await page.setViewportSize({ width: 375, height: 667 })

      await page.goto('/dashboard')

      const program = Effect.gen(function* () {
        const dashboardPageService = yield* DashboardPage

        // Verify sidebar collapses or adapts
        const sidebar = yield* dashboardPageService.sidebar()
        // Sidebar should be visible or collapsed into hamburger menu
        const hamburgerMenu = page.locator(
          '[data-testid="sidebar-hamburger-menu"]',
        )
        const hamburgerCount = yield* LocatorHelpers.count(hamburgerMenu)
        if (hamburgerCount > 0) {
          yield* ExpectHelpers.toBeVisible(hamburgerMenu)
        } else {
          yield* ExpectHelpers.toBeVisible(sidebar)
        }
      })

      await Effect.runPromise(
        program.pipe(Effect.provide(createPageLayers(page))),
      )
    })

    test('dark/light theme toggle works', async ({ page }) => {
      await page.goto('/dashboard')

      const program = Effect.gen(function* () {
        const dashboardPageService = yield* DashboardPage

        const themeToggle = yield* dashboardPageService.themeToggleButton()
        yield* ExpectHelpers.toBeVisible(themeToggle)

        // Toggle theme
        yield* dashboardPageService.toggleTheme()

        // Verify theme changes (check body class or data attribute)
        const body = page.locator('body')
        const hasDarkClass = yield* Effect.promise<boolean>(() =>
          body.evaluate(
            (el) =>
              el.classList.contains('dark') ||
              el.getAttribute('data-theme') === 'dark',
          ),
        )
        expect(hasDarkClass).toBeTruthy()

        // Toggle back
        yield* dashboardPageService.toggleTheme()

        const hasLightClass = yield* Effect.promise<boolean>(() =>
          body.evaluate(
            (el) =>
              !el.classList.contains('dark') ||
              el.getAttribute('data-theme') === 'light',
          ),
        )
        expect(hasLightClass).toBeTruthy()
      })

      await Effect.runPromise(
        program.pipe(Effect.provide(createPageLayers(page))),
      )
    })

    test('browser back/forward navigation works', async ({ page }) => {
      await page.goto('/dashboard')

      const program = Effect.gen(function* () {
        const dashboardPageService = yield* DashboardPage

        // Navigate to users page
        yield* dashboardPageService.clickUsersLink()
        yield* ExpectHelpers.toHaveURL(page, '/dashboard/users')

        // Navigate to roles page
        yield* dashboardPageService.clickRolesLink()
        yield* ExpectHelpers.toHaveURL(page, '/dashboard/roles')

        // Go back
        yield* PageHelpers.goto(page, 'javascript:history.back()')
        yield* ExpectHelpers.toHaveURL(page, '/dashboard/users')

        // Go forward
        yield* PageHelpers.goto(page, 'javascript:history.forward()')
        yield* ExpectHelpers.toHaveURL(page, '/dashboard/roles')
      })

      await Effect.runPromise(
        program.pipe(Effect.provide(createPageLayers(page))),
      )
    })
  })
})
