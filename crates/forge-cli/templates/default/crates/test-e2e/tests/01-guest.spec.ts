/**
 * E2E tests for Guest/Unauthenticated User flows
 * 
 * Tests follow the DAG structure from E2E_TEST_SCENARIOS.md:
 * - 1.1 Initial Access
 * - 1.2 Registration Flow
 * - 1.3 Login Flow
 * - 1.4 Public Endpoints
 * - 1.5 Protected Endpoint Access
 * 
 * Uses Effect.ts for composition and Playwright for browser automation.
 */
import { test, expect } from '@playwright/test'
import { Effect } from 'effect'

import { LoginPage, RegisterPage } from '@/pages'
import { createPageLayers } from '@/fixtures/page-layers'
import { createTestUser, SEED_USERS } from '@/fixtures/test-data'
import { API_BASE_URL } from '@/playwright.config'

test.describe('Guest/Unauthenticated User', () => {
  test.describe('1.1 Initial Access', () => {
    test('should redirect to login when visiting root', async ({ page }) => {
      await page.goto('/')
      
      const program = Effect.gen(function* () {
        const loginPageService = yield* LoginPage
        const container = yield* loginPageService.container()
        const form = yield* loginPageService.form()
        
        yield* Effect.promise(() => expect(page).toHaveURL('/authentication/login'))
        yield* Effect.promise(() => expect(container).toBeVisible())
        yield* Effect.promise(() => expect(form).toBeVisible())
      })

      await Effect.runPromise(program.pipe(Effect.provide(createPageLayers(page))))
    })

    test('should redirect to login when visiting protected route', async ({ page }) => {
      await page.goto('/dashboard')
      
      const program = Effect.gen(function* () {
        const loginPageService = yield* LoginPage
        const container = yield* loginPageService.container()
        
        yield* Effect.promise(() => expect(page).toHaveURL('/authentication/login'))
        yield* Effect.promise(() => expect(container).toBeVisible())
      })

      await Effect.runPromise(program.pipe(Effect.provide(createPageLayers(page))))
    })

    test('should show login form when visiting login page', async ({ page }) => {
      await page.goto('/authentication/login')
      
      const program = Effect.gen(function* () {
        const loginPageService = yield* LoginPage
        
        const container = yield* loginPageService.container()
        const form = yield* loginPageService.form()
        const email = yield* loginPageService.email()
        const password = yield* loginPageService.password()
        const submit = yield* loginPageService.submit()
        const registerLink = yield* loginPageService.registerLink()
        
        yield* Effect.promise(() => expect(container).toBeVisible())
        yield* Effect.promise(() => expect(form).toBeVisible())
        yield* Effect.promise(() => expect(email).toBeVisible())
        yield* Effect.promise(() => expect(password).toBeVisible())
        yield* Effect.promise(() => expect(submit).toBeVisible())
        yield* Effect.promise(() => expect(registerLink).toBeVisible())
      })

      await Effect.runPromise(program.pipe(Effect.provide(createPageLayers(page))))
    })

    test('should show registration form when visiting register page', async ({ page }) => {
      await page.goto('/authentication/register')
      
      const program = Effect.gen(function* () {
        const registerPageService = yield* RegisterPage
        
        const container = yield* registerPageService.container()
        const form = yield* registerPageService.form()
        const email = yield* registerPageService.email()
        const password = yield* registerPageService.password()
        const submit = yield* registerPageService.submit()
        const loginLink = yield* registerPageService.loginLink()
        
        yield* Effect.promise(() => expect(container).toBeVisible())
        yield* Effect.promise(() => expect(form).toBeVisible())
        yield* Effect.promise(() => expect(email).toBeVisible())
        yield* Effect.promise(() => expect(password).toBeVisible())
        yield* Effect.promise(() => expect(submit).toBeVisible())
        yield* Effect.promise(() => expect(loginLink).toBeVisible())
      })

      await Effect.runPromise(program.pipe(Effect.provide(createPageLayers(page))))
    })

    test('should navigate from login to register', async ({ page }) => {
      await page.goto('/authentication/login')
      
      const program = Effect.gen(function* () {
        const loginPageService = yield* LoginPage
        const registerPageService = yield* RegisterPage
        
        yield* loginPageService.clickRegisterLink()
        
        yield* Effect.promise(() => expect(page).toHaveURL('/authentication/register'))
        const container = yield* registerPageService.container()
        yield* Effect.promise(() => expect(container).toBeVisible())
      })

      await Effect.runPromise(program.pipe(Effect.provide(createPageLayers(page))))
    })

    test('should navigate from register to login', async ({ page }) => {
      await page.goto('/authentication/register')
      
      const program = Effect.gen(function* () {
        const registerPageService = yield* RegisterPage
        const loginPageService = yield* LoginPage
        
        yield* registerPageService.clickLoginLink()
        
        yield* Effect.promise(() => expect(page).toHaveURL('/authentication/login'))
        const container = yield* loginPageService.container()
        yield* Effect.promise(() => expect(container).toBeVisible())
      })

      await Effect.runPromise(program.pipe(Effect.provide(createPageLayers(page))))
    })
  })

  test.describe('1.2 Registration Flow', () => {
    test.beforeEach(async ({ page }) => {
      await page.goto('/authentication/register')
    })

    test.describe('Invalid Input', () => {
      test('invalid email shows error', async ({ page }) => {
        const program = Effect.gen(function* () {
          const registerPageService = yield* RegisterPage
          
          const emailLocator = yield* registerPageService.email()
          const passwordLocator = yield* registerPageService.password()
          const submitLocator = yield* registerPageService.submit()
          const errorMessageLocator = yield* registerPageService.errorMessage()
          
          yield* Effect.promise(() => emailLocator.fill('invalid-email'))
          yield* Effect.promise(() => passwordLocator.fill('password123'))
          yield* Effect.promise(() => submitLocator.click())
          
          // Wait for error message
          yield* Effect.promise(() => expect(errorMessageLocator).toBeVisible())
        })

        await Effect.runPromise(program.pipe(Effect.provide(createPageLayers(page))))

        // Verify URL hasn't changed
        await expect(page).toHaveURL('/authentication/register')
      })

      test('short password shows error', async ({ page }) => {
        const testUser = createTestUser({ password: 'short' })
        
        const program = Effect.gen(function* () {
          const registerPageService = yield* RegisterPage
          
          const emailLocator = yield* registerPageService.email()
          const passwordLocator = yield* registerPageService.password()
          const submitLocator = yield* registerPageService.submit()
          const errorMessageLocator = yield* registerPageService.errorMessage()
          
          yield* Effect.promise(() => emailLocator.fill(testUser.email))
          yield* Effect.promise(() => passwordLocator.fill(testUser.password))
          yield* Effect.promise(() => submitLocator.click())
          
          // Wait for error message
          yield* Effect.promise(() => expect(errorMessageLocator).toBeVisible())
        })

        await Effect.runPromise(program.pipe(Effect.provide(createPageLayers(page))))

        await expect(page).toHaveURL('/authentication/register')
      })
    })

    test.describe('Valid Registration', () => {
      test('redirects to login on success', async ({ page }) => {
        const testUser = createTestUser()
        
        const program = Effect.gen(function* () {
          const registerPageService = yield* RegisterPage
          const loginPageService = yield* LoginPage
          
          yield* registerPageService.register(testUser.email, testUser.password)
          
          yield* Effect.promise(() => expect(page).toHaveURL('/authentication/login'))
          const loginContainer = yield* loginPageService.container()
          yield* Effect.promise(() => expect(loginContainer).toBeVisible())
          
          const errorMessage = yield* registerPageService.errorMessage()
          yield* Effect.promise(() => expect(errorMessage).not.toBeVisible())
        })

        await Effect.runPromise(program.pipe(Effect.provide(createPageLayers(page))))
      })

      test('can login with new credentials', async ({ page }) => {
        const testUser = createTestUser()
        
        const program = Effect.gen(function* () {
          const registerPageService = yield* RegisterPage
          const loginPageService = yield* LoginPage
          
          // Register
          yield* registerPageService.register(testUser.email, testUser.password)

          // Login with new credentials
          yield* loginPageService.login(testUser.email, testUser.password)

          // Should redirect to dashboard or scope selection
          yield* Effect.promise(() => expect(page).toHaveURL(/\/dashboard|\/authentication\/select-scope/))
          
          const errorMessage = yield* loginPageService.errorMessage()
          yield* Effect.promise(() => expect(errorMessage).not.toBeVisible())
        })

        await Effect.runPromise(program.pipe(Effect.provide(createPageLayers(page))))
      })
    })

    test('should navigate to login via link', async ({ page }) => {
      const program = Effect.gen(function* () {
        const registerPageService = yield* RegisterPage
        const loginPageService = yield* LoginPage
        
        yield* registerPageService.clickLoginLink()
        
        yield* Effect.promise(() => expect(page).toHaveURL('/authentication/login'))
        const container = yield* loginPageService.container()
        yield* Effect.promise(() => expect(container).toBeVisible())
      })

      await Effect.runPromise(program.pipe(Effect.provide(createPageLayers(page))))
    })
  })

  test.describe('1.3 Login Flow', () => {
    test.beforeEach(async ({ page }) => {
      await page.goto('/authentication/login')
    })

    test.describe('Invalid Credentials', () => {
      test('shows error for invalid email', async ({ page }) => {
        const program = Effect.gen(function* () {
          const loginPageService = yield* LoginPage
          
          const emailLocator = yield* loginPageService.email()
          const passwordLocator = yield* loginPageService.password()
          const submitLocator = yield* loginPageService.submit()
          const errorMessageLocator = yield* loginPageService.errorMessage()
          
          yield* Effect.promise(() => emailLocator.fill('invalid@example.com'))
          yield* Effect.promise(() => passwordLocator.fill('password'))
          yield* Effect.promise(() => submitLocator.click())
          
          // Wait for error message
          yield* Effect.promise(() => expect(errorMessageLocator).toBeVisible())
        })

        await Effect.runPromise(program.pipe(Effect.provide(createPageLayers(page))))

        await expect(page).toHaveURL('/authentication/login')
      })
    })

    test.describe('Valid Credentials', () => {
      test('single profile redirects to dashboard', async ({ page }) => {
        const program = Effect.gen(function* () {
          const loginPageService = yield* LoginPage
          
          yield* loginPageService.login(SEED_USERS.viewer.email, SEED_USERS.viewer.password)
          
          yield* Effect.promise(() => expect(page).toHaveURL('/dashboard'))
          const errorMessage = yield* loginPageService.errorMessage()
          yield* Effect.promise(() => expect(errorMessage).not.toBeVisible())
        })

        await Effect.runPromise(program.pipe(Effect.provide(createPageLayers(page))))
      })

      test('multi-profile redirects to scope selection', async ({ page }) => {
        // Note: This test assumes multi-profile user exists in seed data
        // If not available, skip or create in setup
        const program = Effect.gen(function* () {
          const loginPageService = yield* LoginPage
          
          yield* loginPageService.login(SEED_USERS.multiProfile.email, SEED_USERS.multiProfile.password)
          
          yield* Effect.promise(() => expect(page).toHaveURL('/authentication/select-scope'))
          const errorMessage = yield* loginPageService.errorMessage()
          yield* Effect.promise(() => expect(errorMessage).not.toBeVisible())
        })

        await Effect.runPromise(program.pipe(Effect.provide(createPageLayers(page))))
      })
    })

    test('should navigate to register and back to login', async ({ page }) => {
      const program = Effect.gen(function* () {
        const loginPageService = yield* LoginPage
        const registerPageService = yield* RegisterPage
        
        // Navigate to register
        yield* loginPageService.clickRegisterLink()
        yield* Effect.promise(() => expect(page).toHaveURL('/authentication/register'))
        const registerContainer = yield* registerPageService.container()
        yield* Effect.promise(() => expect(registerContainer).toBeVisible())

        // Navigate back to login
        yield* registerPageService.clickLoginLink()
        yield* Effect.promise(() => expect(page).toHaveURL('/authentication/login'))
        const loginContainer = yield* loginPageService.container()
        yield* Effect.promise(() => expect(loginContainer).toBeVisible())
      })

      await Effect.runPromise(program.pipe(Effect.provide(createPageLayers(page))))
    })
  })

  test.describe('1.4 Public Endpoints', () => {
    test('GET /healthz returns 200 OK', async ({ request }) => {
      const response = await request.get(`${API_BASE_URL}/healthz`)
      expect(response.status()).toBe(200)
    })

    test('GET /livez returns 200 OK', async ({ request }) => {
      const response = await request.get(`${API_BASE_URL}/livez`)
      expect(response.status()).toBe(200)
    })

    test('GET /readyz returns 200 OK', async ({ request }) => {
      const response = await request.get(`${API_BASE_URL}/readyz`)
      expect(response.status()).toBe(200)
    })

    test('GET /api/auth/logout returns 200 OK (works for anonymous)', async ({ request }) => {
      const response = await request.get(`${API_BASE_URL}/api/auth/logout`)
      expect(response.status()).toBe(200)
    })
  })

  test.describe('1.5 Protected Endpoint Access (Unauthenticated)', () => {
    test('GET /api/auth/me returns 401 Unauthorized', async ({ request }) => {
      const response = await request.get(`${API_BASE_URL}/api/auth/me`)
      expect(response.status()).toBe(401)
    })

    test('GET /api/auth/profiles returns 401 Unauthorized', async ({ request }) => {
      const response = await request.get(`${API_BASE_URL}/api/auth/profiles`)
      expect(response.status()).toBe(401)
    })

    test('POST /api/auth/tokens returns 401 Unauthorized', async ({ request }) => {
      const response = await request.post(`${API_BASE_URL}/api/auth/tokens`)
      expect(response.status()).toBe(401)
    })

    test('GET /api/auth/admin returns 401 Unauthorized', async ({ request }) => {
      const response = await request.get(`${API_BASE_URL}/api/auth/admin`)
      expect(response.status()).toBe(401)
    })

    test('GET /api/entities/organization returns 401 Unauthorized', async ({ request }) => {
      const response = await request.get(`${API_BASE_URL}/api/entities/organization`)
      expect(response.status()).toBe(401)
    })

    test('GET /api/dashboard/users returns 401 Unauthorized', async ({ request }) => {
      const response = await request.get(`${API_BASE_URL}/api/dashboard/users`)
      expect(response.status()).toBe(401)
    })
  })
})
