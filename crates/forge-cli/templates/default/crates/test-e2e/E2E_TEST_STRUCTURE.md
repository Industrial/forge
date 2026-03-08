# E2E Test Structure Guide

## Overview

This document defines the architecture and patterns for end-to-end tests using **Playwright** and **Effect.ts**. Instead of traditional Page Object Model (POM), we use **Effect.ts composition** with lightweight **page actions** and **Effect Layers** for dependency injection.

## Core Principles

1. **Effect.ts Composition**: Use Effect.ts for test organization, not POM classes
2. **Lightweight Page Actions**: Composable functions, not heavy classes
3. **Effect Layers for Fixtures**: Dependency injection via Effect's Layer system
4. **Tree/BDD Structure**: Matches DAG structure from `E2E_TEST_SCENARIOS.md`
5. **Fully Parallel**: All tests run in parallel with strict isolation
6. **Unique Data Per Test**: Generate unique identifiers, reuse seed data
7. **Cleanup Only What You Create**: Use `afterEach` for test-specific cleanup

---

## Directory Structure

```
test/e2e/
├── fixtures/
│   ├── auth.ts              # Authentication helpers (Effect-based)
│   ├── test-data.ts         # Data factories (Effect-based)
│   ├── playwright.ts        # PlaywrightPage service
│   └── page-layers.ts       # Helper to combine all page layers
├── pages/
│   ├── LoginPage.ts         # Login page Effect.ts service
│   ├── RegisterPage.ts      # Registration page Effect.ts service
│   ├── DashboardPage.ts    # Dashboard page Effect.ts service
│   ├── SelectScopePage.ts   # Scope selection page Effect.ts service
│   └── index.ts             # Central export for all page services
├── setup/
│   ├── global-setup.ts      # One-time seed data setup (Effect-based)
│   └── auth-setup.ts        # Authenticated state setup projects
├── tests/
│   ├── 01-guest.spec.ts     # Guest/unauthenticated user flows
│   ├── 02-viewer.spec.ts    # Viewer role flows
│   ├── 03-editor.spec.ts    # Editor role flows
│   ├── 04-org-owner.spec.ts # Org owner flows
│   ├── 05-org-admin.spec.ts # Org admin flows
│   ├── 06-app-admin.spec.ts # App admin flows
│   └── 07-security.spec.ts  # Security and edge cases
├── playwright.config.ts     # Playwright configuration
└── E2E_TEST_SCENARIOS.md    # DAG structure reference
```

---

## 1. Page Services Pattern (Effect.ts Services)

Instead of heavy Page Object classes, use **Effect.ts services** - each page is its own service file in the `pages/` directory.

### Page Service Structure

Each page is an **Effect.ts service** (using `Context.Tag`) with methods that:
- Take a `PlaywrightPage` from context (via `yield*`)
- Return `Effect<void | Locator>`
- **All methods use `Effect.gen()`** - even for synchronous operations
- Are composable and type-safe

### `pages/LoginPage.ts`

```typescript
import { Page, Locator } from '@playwright/test'
import { Effect, Context, Layer } from 'effect'
import { PlaywrightPage } from '../fixtures/playwright'

/**
 * LoginPage service - Effect.ts service for login page interactions
 * Methods take PlaywrightPage from context and return Effects
 */
export class LoginPage extends Context.Tag('LoginPage')<
  LoginPage,
  {
    readonly container: () => Effect.Effect<Locator>
    readonly form: () => Effect.Effect<Locator>
    readonly email: () => Effect.Effect<Locator>
    readonly password: () => Effect.Effect<Locator>
    readonly submit: () => Effect.Effect<Locator>
    readonly registerLink: () => Effect.Effect<Locator>
    readonly errorMessage: () => Effect.Effect<Locator>
    readonly login: (email: string, password: string) => Effect.Effect<void>
    readonly clickRegisterLink: () => Effect.Effect<void>
  }
>() {}

/**
 * Create LoginPage layer from PlaywrightPage
 */
export const LoginPageLive = Layer.effect(
  LoginPage,
  Effect.gen(function* () {
    const playwrightPage = yield* PlaywrightPage

    return {
      container: () =>
        Effect.gen(function* () {
          return playwrightPage.getByTestId('login-page')
        }),

      form: () =>
        Effect.gen(function* () {
          return playwrightPage.getByTestId('login-form')
        }),

      email: () =>
        Effect.gen(function* () {
          return playwrightPage.getByTestId('login-email-input')
        }),

      password: () =>
        Effect.gen(function* () {
          return playwrightPage.getByTestId('login-password-input')
        }),

      submit: () =>
        Effect.gen(function* () {
          return playwrightPage.getByTestId('login-submit-button')
        }),

      registerLink: () =>
        Effect.gen(function* () {
          return playwrightPage.getByTestId('login-register-link')
        }),

      errorMessage: () =>
        Effect.gen(function* () {
          return playwrightPage.getByTestId('login-error-message')
        }),

      login: (email: string, password: string) =>
        Effect.gen(function* () {
          yield* Effect.promise(() =>
            playwrightPage.getByTestId('login-email-input').fill(email)
          )
          yield* Effect.promise(() =>
            playwrightPage.getByTestId('login-password-input').fill(password)
          )
          yield* Effect.promise(() =>
            playwrightPage.getByTestId('login-submit-button').click()
          )
          yield* Effect.promise(() =>
            playwrightPage.waitForURL(/\/dashboard|\/authentication\/select-scope/, {
              timeout: 5000,
            })
          )
        }),

      clickRegisterLink: () =>
        Effect.gen(function* () {
          yield* Effect.promise(() =>
            playwrightPage.getByTestId('login-register-link').click()
          )
        }),
    }
  })
)
```

### `pages/RegisterPage.ts`

```typescript
import { Page, Locator } from '@playwright/test'
import { Effect, Context, Layer } from 'effect'
import { PlaywrightPage } from '../fixtures/playwright'

export class RegisterPage extends Context.Tag('RegisterPage')<
  RegisterPage,
  {
    readonly container: () => Effect.Effect<Locator>
    readonly form: () => Effect.Effect<Locator>
    readonly email: () => Effect.Effect<Locator>
    readonly password: () => Effect.Effect<Locator>
    readonly submit: () => Effect.Effect<Locator>
    readonly loginLink: () => Effect.Effect<Locator>
    readonly errorMessage: () => Effect.Effect<Locator>
    readonly register: (email: string, password: string) => Effect.Effect<void>
    readonly clickLoginLink: () => Effect.Effect<void>
  }
>() {}

export const RegisterPageLive = Layer.effect(
  RegisterPage,
  Effect.gen(function* () {
    const playwrightPage = yield* PlaywrightPage
    // ... implementation similar to LoginPage
  })
)
```

### `pages/index.ts`

```typescript
/**
 * Central export for all page services
 */
export { LoginPage, LoginPageLive } from './LoginPage'
export { RegisterPage, RegisterPageLive } from './RegisterPage'
export { DashboardPage, DashboardPageLive } from './DashboardPage'
export { SelectScopePage, SelectScopePageLive } from './SelectScopePage'
```

**Key Benefits**:
- ✅ Effect.ts services with proper dependency injection
- ✅ Type-safe: Methods return `Effect<void | Locator>`
- ✅ Composable: Services can be combined with Effect.gen
- ✅ Testable: Easy to mock/replace using Effect Layers
- ✅ Organized: One file per page, easy to find and maintain
- ✅ No classes or inheritance - just Effect.ts services

---

## 2. Effect.ts Fixtures

Use **Effect Layers** for test fixtures and dependency injection:

### `fixtures/playwright.ts`

```typescript
import { Page } from '@playwright/test'
import { Effect, Context, Layer } from 'effect'

/**
 * Playwright Page service - provides Page instance to Effect programs
 * This enables Effect.ts composition for E2E tests
 */
export class PlaywrightPage extends Context.Tag('PlaywrightPage')<
  PlaywrightPage,
  Page
>() {}

/**
 * Create a Layer that provides PlaywrightPage
 * Use this to inject Page into Effect programs
 */
export const createPlaywrightPageLayer = (page: Page): Layer.Layer<PlaywrightPage> =>
  Layer.succeed(PlaywrightPage, page)
```

### `fixtures/page-layers.ts`

```typescript
import { Page } from '@playwright/test'
import { Layer } from 'effect'
import { createPlaywrightPageLayer, PlaywrightPage } from './playwright'
import {
  LoginPage,
  LoginPageLive,
  RegisterPage,
  RegisterPageLive,
  DashboardPage,
  DashboardPageLive,
  SelectScopePage,
  SelectScopePageLive,
} from '../pages'

/**
 * Create all page layers from a Playwright Page instance
 * Combines PlaywrightPage with all page services
 * Page services depend on PlaywrightPage, so we provide it to them
 */
export const createPageLayers = (page: Page) => {
  const playwrightPageLayer = createPlaywrightPageLayer(page)
  
  return Layer.mergeAll(
    playwrightPageLayer,
    LoginPageLive.pipe(Layer.provide(playwrightPageLayer)),
    RegisterPageLive.pipe(Layer.provide(playwrightPageLayer)),
    DashboardPageLive.pipe(Layer.provide(playwrightPageLayer)),
    SelectScopePageLive.pipe(Layer.provide(playwrightPageLayer)),
  )
}
```

### `fixtures/auth.ts` (Optional - for future use)

```typescript
import { Page } from '@playwright/test'
import { Effect, Context, Layer } from 'effect'
import { LoginPage } from '../pages'
import { PlaywrightPage, createPlaywrightPageLayer } from './playwright'

/**
 * Authentication service - provides auth helpers via Effect
 * Uses LoginPage service internally
 */
export class AuthService extends Context.Tag('AuthService')<
  AuthService,
  {
    readonly login: (email: string, password: string) => Effect.Effect<void>
    readonly logout: () => Effect.Effect<void>
    readonly isAuthenticated: () => Effect.Effect<boolean>
  }
>() {}

/**
 * Create authentication layer from Playwright page
 */
export const createAuthLayer = (page: Page): Layer.Layer<AuthService, never, PlaywrightPage | LoginPage> =>
  Layer.effect(
    AuthService,
    Effect.gen(function* () {
      const loginPageService = yield* LoginPage
      const playwrightPage = yield* PlaywrightPage

      return {
        login: (email: string, password: string) =>
          loginPageService.login(email, password),

        logout: () =>
          Effect.gen(function* () {
            yield* Effect.promise(() => playwrightPage.getByTestId('logout-button').click())
            yield* Effect.promise(() => playwrightPage.waitForURL('/authentication/login'))
          }),

        isAuthenticated: () =>
          Effect.gen(function* () {
            const url = playwrightPage.url()
            return !url.includes('/authentication/login')
          }),
      }
    }).pipe(Effect.provide(createPlaywrightPageLayer(page)))
  )
```

### `fixtures/test-data.ts`

```typescript
import { Effect } from 'effect'

/**
 * Test data factories using Effect.ts
 * Generate unique data per test run
 */

export const createTestUser = (overrides?: Partial<TestUser>): TestUser => {
  const timestamp = Date.now()
  const random = Math.random().toString(36).substring(7)
  
  return {
    email: `test-${timestamp}-${random}@example.com`,
    password: 'TestPassword123!',
    name: `Test User ${timestamp}`,
    ...overrides,
  }
}

export const createTestOrganization = (overrides?: Partial<TestOrganization>): TestOrganization => {
  const timestamp = Date.now()
  const random = Math.random().toString(36).substring(7)
  
  return {
    name: `Test Org ${timestamp}`,
    slug: `test-org-${timestamp}-${random}`,
    ...overrides,
  }
}

/**
 * Seed data users (reused across tests)
 */
export const SEED_USERS = {
  viewer: { email: 'viewer@default.org', password: 'password' },
  editor: { email: 'editor@default.org', password: 'password' },
  orgOwner: { email: 'owner@default.org', password: 'password' },
  orgAdmin: { email: 'orgadmin@default.org', password: 'password' },
  appAdmin: { email: 'admin@admin.com', password: 'password' },
} as const

export type TestUser = {
  email: string
  password: string
  name?: string
}

export type TestOrganization = {
  name: string
  slug: string
}
```

---

## 3. Setup Projects (Expensive Operations)

Use Playwright **setup projects** for expensive operations like authentication:

### `setup/auth-setup.ts`

```typescript
import { test as setup } from '@playwright/test'
import { Effect } from 'effect'
import { loginPage } from '../fixtures/pages'
import { SEED_USERS } from '../fixtures/test-data'

import { setup } from '@playwright/test'
import { Effect } from 'effect'
import { LoginPage } from '../pages'
import { SEED_USERS } from '../fixtures/test-data'
import { createPageLayers } from '../fixtures/page-layers'

/**
 * Setup project: Authenticate users once, reuse state across tests
 * This runs once per worker, not per test
 */
setup('authenticate viewer', async ({ page }) => {
  const program = Effect.gen(function* () {
    const loginPageService = yield* LoginPage
    yield* loginPageService.login(SEED_USERS.viewer.email, SEED_USERS.viewer.password)
  })
  
  await Effect.runPromise(program.pipe(Effect.provide(createPageLayers(page))))
  
  // Save authenticated state
  await page.context().storageState({ path: 'playwright/.auth/viewer.json' })
})

setup('authenticate editor', async ({ page }) => {
  const program = Effect.gen(function* () {
    const loginPageService = yield* LoginPage
    yield* loginPageService.login(SEED_USERS.editor.email, SEED_USERS.editor.password)
  })
  
  await Effect.runPromise(program.pipe(Effect.provide(createPageLayers(page))))
  await page.context().storageState({ path: 'playwright/.auth/editor.json' })
})

setup('authenticate org owner', async ({ page }) => {
  const program = Effect.gen(function* () {
    const loginPageService = yield* LoginPage
    yield* loginPageService.login(SEED_USERS.orgOwner.email, SEED_USERS.orgOwner.password)
  })
  
  await Effect.runPromise(program.pipe(Effect.provide(createPageLayers(page))))
  await page.context().storageState({ path: 'playwright/.auth/org-owner.json' })
})
```

### `playwright.config.ts` (Updated)

```typescript
import { defineConfig, devices } from '@playwright/test'
import path from 'node:path'
import { fileURLToPath } from 'node:url'

const __dirname = path.dirname(fileURLToPath(import.meta.url))
const repoRoot = path.resolve(__dirname, '../..')

const baseURL = process.env.E2E_BASE_URL ?? 'http://127.0.0.1:35173'
export const API_BASE_URL = process.env.E2E_API_URL ?? 'http://127.0.0.1:30999'

export default defineConfig({
  testDir: path.join(__dirname, 'tests'),
  
  // Fully parallel execution
  fullyParallel: true,
  workers: process.env.CI ? 2 : undefined, // Use CPU cores in CI, auto-detect locally
  
  forbidOnly: !!process.env.CI,
  retries: process.env.CI ? 2 : 0,
  
  reporter: [['list']],
  
  use: {
    baseURL,
    trace: 'on-first-retry',
    screenshot: 'only-on-failure',
    video: 'retain-on-failure',
  },
  
  // Setup projects run once per worker (expensive operations)
  projects: [
    // Setup projects (run first, once per worker)
    {
      name: 'setup-viewer',
      testMatch: /.*\.setup\.ts/,
    },
    {
      name: 'setup-editor',
      testMatch: /.*\.setup\.ts/,
    },
    {
      name: 'setup-org-owner',
      testMatch: /.*\.setup\.ts/,
    },
    
    // Test projects (run in parallel, use setup state)
    {
      name: 'chromium',
      use: { ...devices['Desktop Chrome'] },
      dependencies: ['setup-viewer', 'setup-editor', 'setup-org-owner'],
    },
  ],
  
  timeout: 30_000,
})

export const REPO_ROOT = repoRoot
```

---

## 4. Test Structure (Tree/BDD Pattern)

Tests follow the DAG structure from `E2E_TEST_SCENARIOS.md`:

### `tests/01-guest.spec.ts`

```typescript
import { test, expect } from '@playwright/test'
import { Effect } from 'effect'
import { LoginPage, RegisterPage } from '../pages'
import { createTestUser, SEED_USERS } from '../fixtures/test-data'
import { createPageLayers } from '../fixtures/page-layers'

test.describe('Guest/Unauthenticated User', () => {
  test.describe('Initial Access', () => {
    test('should redirect to login when visiting root', async ({ page }) => {
      await page.goto('/')
      
      const program = Effect.gen(function* () {
        const loginPageService = yield* LoginPage
        const container = yield* loginPageService.container()
        
        yield* Effect.promise(() => expect(page).toHaveURL('/authentication/login'))
        yield* Effect.promise(() => expect(container).toBeVisible())
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
  })

  test.describe('Registration Flow', () => {
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
        })

        await Effect.runPromise(program.pipe(Effect.provide(createPageLayers(page))))
      })
    })
  })

  test.describe('Login Flow', () => {
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
        })

        await Effect.runPromise(program.pipe(Effect.provide(createPageLayers(page))))
      })

      test('multi-profile redirects to scope selection', async ({ page }) => {
        const program = Effect.gen(function* () {
          const loginPageService = yield* LoginPage
          
          yield* loginPageService.login(SEED_USERS.multiProfile.email, SEED_USERS.multiProfile.password)
          
          yield* Effect.promise(() => expect(page).toHaveURL('/authentication/select-scope'))
        })

        await Effect.runPromise(program.pipe(Effect.provide(createPageLayers(page))))
      })
    })
  })
})
```

### `tests/02-viewer.spec.ts`

```typescript
import { test, expect } from '@playwright/test'
import { Effect } from 'effect'
import { DashboardPage } from '../pages'
import { createPageLayers } from '../fixtures/page-layers'

test.describe('Viewer Role', () => {
  // Use authenticated state from setup project
  test.use({ storageState: 'playwright/.auth/viewer.json' })

  test.describe('Dashboard Access', () => {
    test.beforeEach(async ({ page }) => {
      await page.goto('/dashboard')
    })

    test('should display dashboard page', async ({ page }) => {
      const program = Effect.gen(function* () {
        const dashboardPageService = yield* DashboardPage
        
        const container = yield* dashboardPageService.container()
        const sidebar = yield* dashboardPageService.sidebar()
        
        yield* Effect.promise(() => expect(container).toBeVisible())
        yield* Effect.promise(() => expect(sidebar).toBeVisible())
      })

      await Effect.runPromise(program.pipe(Effect.provide(createPageLayers(page))))
    })

    test('should show navigation links', async ({ page }) => {
      const program = Effect.gen(function* () {
        const dashboardPageService = yield* DashboardPage
        
        const usersLink = yield* dashboardPageService.usersLink()
        const rolesLink = yield* dashboardPageService.rolesLink()
        
        yield* Effect.promise(() => expect(usersLink).toBeVisible())
        yield* Effect.promise(() => expect(rolesLink).toBeVisible())
      })

      await Effect.runPromise(program.pipe(Effect.provide(createPageLayers(page))))
    })
  })
})
```

---

## 5. Data Management & Cleanup

### Unique Data Per Test

```typescript
import { UsersPage } from '../pages'
import { createPageLayers } from '../fixtures/page-layers'

test('creates user with unique email', async ({ page }) => {
  // Generate unique data per test
  const testUser = createTestUser()
  
  const program = Effect.gen(function* () {
    const usersPageService = yield* UsersPage
    
    // Test creates user
    yield* usersPageService.createUser(testUser.email, testUser.password)
    
    // Verify user exists
    const row = yield* usersPageService.row(testUser.email)
    yield* Effect.promise(() => expect(row).toBeVisible())
  })

  await Effect.runPromise(program.pipe(Effect.provide(createPageLayers(page))))
  
  // Cleanup: Delete what we created
  // (Implementation depends on your API)
})
```

### Cleanup Pattern

```typescript
import { UsersPage } from '../pages'
import { createPageLayers } from '../fixtures/page-layers'

test.describe('User Management', () => {
  const createdUsers: string[] = []

  test.afterEach(async ({ page, request }) => {
    // Cleanup only what this test file created
    for (const email of createdUsers) {
      await request.delete(`${API_BASE_URL}/api/entities/user/${email}`)
        .catch(() => {}) // Ignore cleanup errors
    }
    createdUsers.length = 0
  })

  test('creates and deletes user', async ({ page }) => {
    const testUser = createTestUser()
    createdUsers.push(testUser.email)

    const program = Effect.gen(function* () {
      const usersPageService = yield* UsersPage

      // Create user
      yield* usersPageService.createUser(testUser.email, testUser.password)

      // Verify
      const row = yield* usersPageService.row(testUser.email)
      yield* Effect.promise(() => expect(row).toBeVisible())

      // Delete (cleanup handled by afterEach)
      const deleteButton = yield* usersPageService.deleteButton(testUser.email)
      yield* Effect.promise(() => deleteButton.click())
      
      const confirmButton = yield* usersPageService.confirmDeleteButton()
      yield* Effect.promise(() => confirmButton.click())
    })

    await Effect.runPromise(program.pipe(Effect.provide(createPageLayers(page))))
  })
})
```

---

## 6. Error Handling with Effect.ts

```typescript
import { Effect } from 'effect'
import { HttpClientError } from '@effect/platform'
import { UsersPage } from '../pages'
import { createPageLayers } from '../fixtures/page-layers'

test('handles API errors gracefully', async ({ page }) => {
  const program = Effect.gen(function* () {
    const usersPageService = yield* UsersPage
    
    // Attempt operation that may fail
    yield* usersPageService.createUser('invalid@email', 'password').pipe(
      Effect.catchAll((error) => {
        // Handle specific error types
        if (error instanceof HttpClientError.ResponseError) {
          return Effect.succeed('handled')
        }
        return Effect.fail(error)
      })
    )
  })

  await Effect.runPromise(program.pipe(Effect.provide(createPageLayers(page))))
})
```

---

## 7. Parallel Execution Best Practices

### Test Isolation

```typescript
// ✅ GOOD - Each test is independent
test('test 1', async ({ page }) => {
  const user1 = createTestUser() // Unique data
  // ... test
})

test('test 2', async ({ page }) => {
  const user2 = createTestUser() // Unique data
  // ... test
})

// ❌ BAD - Shared mutable state
let sharedUser: TestUser | undefined

test('test 1', async ({ page }) => {
  sharedUser = createTestUser() // Leaks to other tests!
})

test('test 2', async ({ page }) => {
  // May see sharedUser from test 1 - breaks isolation!
})
```

### Avoiding Race Conditions

```typescript
// ✅ GOOD - Use unique identifiers
const uniqueId = `${Date.now()}-${Math.random().toString(36).substring(7)}`
const email = `user-${uniqueId}@example.com`

// ❌ BAD - Sequential IDs that can collide
let id = 0
const email = `user-${++id}@example.com` // Can collide in parallel!
```

---

## 8. Effect.ts Composition Patterns

### Sequential Composition

```typescript
import { LoginPage, DashboardPage, UsersPage } from '../pages'
import { createPageLayers } from '../fixtures/page-layers'

test('multi-step flow', async ({ page }) => {
  const program = Effect.gen(function* () {
    const loginPageService = yield* LoginPage
    const dashboardPageService = yield* DashboardPage
    const usersPageService = yield* UsersPage
    
    // Step 1: Login
    yield* loginPageService.login(email, password)
    
    // Step 2: Navigate
    const usersLink = yield* dashboardPageService.usersLink()
    yield* Effect.promise(() => usersLink.click())
    
    // Step 3: Create user
    yield* usersPageService.createUser(newEmail, newPassword)
    
    // Step 4: Verify
    const row = yield* usersPageService.row(newEmail)
    yield* Effect.promise(() => expect(row).toBeVisible())
  })

  await Effect.runPromise(program.pipe(Effect.provide(createPageLayers(page))))
})
```

### Parallel Composition

```typescript
import { PlaywrightPage } from '../fixtures/playwright'
import { createPageLayers } from '../fixtures/page-layers'

test('loads multiple pages in parallel', async ({ page }) => {
  const program = Effect.gen(function* () {
    const playwrightPage = yield* PlaywrightPage
    
    // Load multiple pages concurrently
    const [users, roles, permissions] = yield* Effect.all([
      Effect.promise(() => playwrightPage.goto('/users')),
      Effect.promise(() => playwrightPage.goto('/roles')),
      Effect.promise(() => playwrightPage.goto('/permissions')),
    ], { concurrency: 3 })
    
    return { users, roles, permissions }
  })

  await Effect.runPromise(program.pipe(Effect.provide(createPageLayers(page))))
})
```

---

## 9. Testing Effect.ts Services

When testing Effect.ts services used by the frontend, use Effect's Layer system:

```typescript
import { Layer } from 'effect'
import { HttpClient } from '@effect/platform'

test('API service handles errors', async ({ page }) => {
  // Create mock HttpClient using Effect Layers
  const mockHttpClient = HttpClient.make((request) => {
    return Effect.fail(new HttpClientError.RequestError({
      request: {} as any,
      reason: 'Transport',
      cause: new Error('Network error'),
    } as any))
  })

  const mockLayer = Layer.succeed(HttpClient.HttpClient, mockHttpClient)

  // Test your Effect-based service
  const result = await Effect.runPromise(
    myService.getData().pipe(
      Effect.provide(mockLayer),
      Effect.catchAll(() => Effect.succeed('fallback'))
    )
  )

  expect(result).toBe('fallback')
})
```

---

## 10. Migration from POM to Effect.ts

### Before (POM)

```typescript
class LoginPage {
  constructor(private page: Page) {}
  
  async login(email: string, password: string) {
    await this.page.getByTestId('login-email-input').fill(email)
    await this.page.getByTestId('login-password-input').fill(password)
    await this.page.getByTestId('login-submit-button').click()
  }
}

test('login', async ({ page }) => {
  const loginPage = new LoginPage(page)
  await loginPage.login('user@example.com', 'password')
})
```

### After (Effect.ts Services)

```typescript
// pages/LoginPage.ts
export class LoginPage extends Context.Tag('LoginPage')<
  LoginPage,
  {
    readonly login: (email: string, password: string) => Effect.Effect<void>
  }
>() {}

export const LoginPageLive = Layer.effect(
  LoginPage,
  Effect.gen(function* () {
    const playwrightPage = yield* PlaywrightPage
    return {
      container: () =>
        Effect.gen(function* () {
          return playwrightPage.getByTestId('login-page')
        }),
      
      login: (email: string, password: string) =>
        Effect.gen(function* () {
          yield* Effect.promise(() => playwrightPage.getByTestId('login-email-input').fill(email))
          yield* Effect.promise(() => playwrightPage.getByTestId('login-password-input').fill(password))
          yield* Effect.promise(() => playwrightPage.getByTestId('login-submit-button').click())
        }),
    }
  })
)

// tests/01-guest.spec.ts
import { LoginPage } from '../pages'
import { createPageLayers } from '../fixtures/page-layers'

test('login', async ({ page }) => {
  const program = Effect.gen(function* () {
    const loginPageService = yield* LoginPage
    yield* loginPageService.login('user@example.com', 'password')
  })
  
  await Effect.runPromise(program.pipe(Effect.provide(createPageLayers(page))))
})
```

**Benefits**:
- ✅ Effect.ts services with proper dependency injection
- ✅ Type-safe: Methods return `Effect<void | Locator>`
- ✅ Composable: Services can be combined with Effect.gen
- ✅ Testable: Easy to mock/replace using Effect Layers
- ✅ Organized: One file per page, easy to find and maintain
- ✅ No classes or inheritance - just Effect.ts services

---

## 11. Best Practices Summary

### ✅ DO

1. **Use Effect.ts services** - Each page is an Effect.ts service (`Context.Tag`)
2. **One file per page** - `pages/LoginPage.ts`, `pages/RegisterPage.ts`, etc.
3. **Methods return Effects** - `Effect<void>` for actions, `Effect<Locator>` for elements
4. **Use createPageLayers** - Helper to combine all page services
5. **Effect Layers for fixtures** - Dependency injection via Layers
6. **Tree/BDD structure** - Match DAG from scenarios
7. **Unique data per test** - Generate unique identifiers
8. **Cleanup only what you create** - Use afterEach for test-specific cleanup
9. **Setup projects for expensive ops** - Auth, seed data
10. **beforeEach sparingly** - Only for navigation, not expensive setup
11. **Fully parallel** - All tests run in parallel
12. **Effect error handling** - Use Effect combinators (catchAll, retry)

### ❌ DON'T

1. **Don't use POM classes** - Use Effect.ts services instead
2. **Don't put all pages in one file** - One page service per file
3. **Don't use vi.mock()** - Use Effect Layers for mocks
4. **Don't share mutable state** - Each test is isolated
5. **Don't cleanup seed data** - Only cleanup test-created data
6. **Don't use beforeEach for expensive ops** - Use setup projects
7. **Don't mix async/await with Effect** - Use Effect composition
8. **Don't test implementation details** - Test user-visible behavior
9. **Don't create dependencies internally** - Use Effect Layers
10. **Don't access PlaywrightPage directly in tests** - Use page services via `yield*`

---

## 12. Example: Complete Test File

See `tests/01-guest.spec.ts` above for a complete example following all patterns.

---

## 13. References

- [Effect.ts Documentation](https://effect.website/)
- [Effect.ts Layers](https://effect.website/docs/guides/layer/)
- [Playwright Best Practices](https://playwright.dev/docs/best-practices)
- [Playwright Parallel Execution](https://playwright.dev/docs/test-parallel)
- `E2E_TEST_SCENARIOS.md` - DAG structure reference

---

## 14. Next Steps

1. Create `fixtures/pages.ts` with page actions
2. Create `fixtures/playwright.ts` for PlaywrightPage service
3. Create `fixtures/auth.ts` for authentication helpers
4. Create `fixtures/test-data.ts` for data factories
5. Create `setup/auth-setup.ts` for authenticated state
6. Update `playwright.config.ts` with setup projects
7. Create test files following the tree structure
8. Implement tests based on `E2E_TEST_SCENARIOS.md`

---

**Remember**: Effect.ts enables **explicit dependency management**, **type-safe composition**, and **testable code**. Embrace its patterns rather than fighting them. Use Effect Layers for dependency injection, compose page actions with Effect.gen, and keep tests isolated and parallel.
