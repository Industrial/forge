import { Page } from '@playwright/test'
import { Effect } from 'effect'

/**
 * Effect.ts helpers for Playwright Page actions
 * These wrap Playwright's promise-based APIs in Effect.ts for use with yield*
 */

/**
 * Navigate to a URL
 */
export const goto = (page: Page, url: string, options?: { timeout?: number; waitUntil?: 'load' | 'domcontentloaded' | 'networkidle' | 'commit' }): Effect.Effect<void> =>
  Effect.promise(() => page.goto(url, options))

/**
 * Wait for URL to match pattern
 */
export const waitForURL = (
  page: Page,
  url: string | RegExp,
  options?: { timeout?: number; waitUntil?: 'load' | 'domcontentloaded' | 'networkidle' | 'commit' },
): Effect.Effect<void> =>
  Effect.promise(() => page.waitForURL(url, options))

/**
 * Get current URL
 */
export const url = (page: Page): Effect.Effect<string> =>
  Effect.gen(function* () {
    return page.url()
  })

/**
 * Reload the page
 */
export const reload = (page: Page, options?: { timeout?: number; waitUntil?: 'load' | 'domcontentloaded' | 'networkidle' | 'commit' }): Effect.Effect<void> =>
  Effect.promise(() => page.reload(options))

/**
 * Go back in browser history
 */
export const goBack = (page: Page, options?: { timeout?: number; waitUntil?: 'load' | 'domcontentloaded' | 'networkidle' | 'commit' }): Effect.Effect<void> =>
  Effect.promise(() => page.goBack(options))

/**
 * Go forward in browser history
 */
export const goForward = (page: Page, options?: { timeout?: number; waitUntil?: 'load' | 'domcontentloaded' | 'networkidle' | 'commit' }): Effect.Effect<void> =>
  Effect.promise(() => page.goForward(options))

/**
 * Get page title
 */
export const title = (page: Page): Effect.Effect<string> =>
  Effect.promise(() => page.title())

/**
 * Take a screenshot
 */
export const screenshot = (
  page: Page,
  options?: { path?: string; fullPage?: boolean; timeout?: number },
): Effect.Effect<Buffer> =>
  Effect.promise(() => page.screenshot(options))

/**
 * Wait for a specific timeout
 */
export const wait = (page: Page, timeout: number): Effect.Effect<void> =>
  Effect.promise(() => page.waitForTimeout(timeout))

/**
 * Wait for a function to return truthy value
 */
export const waitForFunction = <T>(
  page: Page,
  fn: () => T | Promise<T>,
  options?: { timeout?: number; polling?: number | 'raf' },
): Effect.Effect<T> =>
  Effect.promise(() => page.waitForFunction(fn, options))

/**
 * Wait for load state
 */
export const waitForLoadState = (
  page: Page,
  state?: 'load' | 'domcontentloaded' | 'networkidle',
  options?: { timeout?: number },
): Effect.Effect<void> =>
  Effect.promise(() => page.waitForLoadState(state, options))

/**
 * Get locator by test ID from page
 */
export const getByTestId = (page: Page, testId: string): Effect.Effect<import('@playwright/test').Locator> =>
  Effect.gen(function* () {
    return page.getByTestId(testId)
  })

/**
 * Get locator by text from page
 */
export const getByText = (page: Page, text: string | RegExp): Effect.Effect<import('@playwright/test').Locator> =>
  Effect.gen(function* () {
    return page.getByText(text)
  })

/**
 * Get locator by role from page
 */
export const getByRole = (
  page: Page,
  role: string,
  options?: { name?: string | RegExp; checked?: boolean; disabled?: boolean; exact?: boolean; expanded?: boolean; includeHidden?: boolean; level?: number; pressed?: boolean; selected?: boolean },
): Effect.Effect<import('@playwright/test').Locator> =>
  Effect.gen(function* () {
    return page.getByRole(role as any, options)
  })

/**
 * Get locator by label from page
 */
export const getByLabel = (page: Page, text: string | RegExp): Effect.Effect<import('@playwright/test').Locator> =>
  Effect.gen(function* () {
    return page.getByLabel(text)
  })

/**
 * Get locator by placeholder from page
 */
export const getByPlaceholder = (page: Page, text: string | RegExp): Effect.Effect<import('@playwright/test').Locator> =>
  Effect.gen(function* () {
    return page.getByPlaceholder(text)
  })
