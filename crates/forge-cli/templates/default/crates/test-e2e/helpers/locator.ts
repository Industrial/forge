import type { Locator } from '@playwright/test'
import { Effect } from 'effect'

/**
 * Effect.ts helpers for Playwright Locator actions
 * These wrap Playwright's promise-based APIs in Effect.ts for use with yield*
 */

/**
 * Click a locator
 */
export const click = (locator: Locator): Effect.Effect<void> =>
  Effect.promise(() => locator.click())

/**
 * Fill a locator with text
 */
export const fill = (locator: Locator, value: string): Effect.Effect<void> =>
  Effect.promise(() => locator.fill(value))

/**
 * Clear and fill a locator with text
 */
export const clearAndFill = (
  locator: Locator,
  value: string,
): Effect.Effect<void> =>
  Effect.promise(() => locator.clear().then(() => locator.fill(value)))

/**
 * Get text content from a locator
 */
export const textContent = (locator: Locator): Effect.Effect<string | null> =>
  Effect.promise(() => locator.textContent())

/**
 * Get inner text from a locator
 */
export const innerText = (locator: Locator): Effect.Effect<string> =>
  Effect.promise(() => locator.innerText())

/**
 * Check if locator is visible
 */
export const isVisible = (locator: Locator): Effect.Effect<boolean> =>
  Effect.promise(() => locator.isVisible())

/**
 * Check if locator is enabled
 */
export const isEnabled = (locator: Locator): Effect.Effect<boolean> =>
  Effect.promise(() => locator.isEnabled())

/**
 * Check if locator is checked (for checkboxes/radios)
 */
export const isChecked = (locator: Locator): Effect.Effect<boolean> =>
  Effect.promise(() => locator.isChecked())

/**
 * Wait for locator to be visible
 */
export const waitForVisible = (
  locator: Locator,
  options?: {
    timeout?: number
    state?: 'attached' | 'detached' | 'visible' | 'hidden'
  },
): Effect.Effect<void> =>
  Effect.promise(() =>
    locator.waitFor({ ...options, state: options?.state ?? 'visible' }),
  )

/**
 * Wait for locator to be hidden
 */
export const waitForHidden = (
  locator: Locator,
  options?: { timeout?: number },
): Effect.Effect<void> =>
  Effect.promise(() => locator.waitFor({ ...options, state: 'hidden' }))

/**
 * Hover over a locator
 */
export const hover = (locator: Locator): Effect.Effect<void> =>
  Effect.promise(() => locator.hover())

/**
 * Double click a locator
 */
export const dblclick = (locator: Locator): Effect.Effect<void> =>
  Effect.promise(() => locator.dblclick())

/**
 * Right click a locator
 */
export const clickRight = (locator: Locator): Effect.Effect<void> =>
  Effect.promise(() => locator.click({ button: 'right' }))

/**
 * Select an option in a select element
 */
export const selectOption = (
  locator: Locator,
  values: string | string[],
): Effect.Effect<string[]> => Effect.promise(() => locator.selectOption(values))

/**
 * Check a checkbox or radio button
 */
export const check = (locator: Locator): Effect.Effect<void> =>
  Effect.promise(() => locator.check())

/**
 * Uncheck a checkbox
 */
export const uncheck = (locator: Locator): Effect.Effect<void> =>
  Effect.promise(() => locator.uncheck())

/**
 * Get attribute value from a locator
 */
export const getAttribute = (
  locator: Locator,
  name: string,
): Effect.Effect<string | null> =>
  Effect.promise(() => locator.getAttribute(name))

/**
 * Get count of matching locators
 */
export const count = (locator: Locator): Effect.Effect<number> =>
  Effect.promise(() => locator.count())

/**
 * Get first matching locator
 */
export const first = (locator: Locator): Effect.Effect<Locator> =>
  Effect.gen(function* () {
    return locator.first()
  })

/**
 * Get nth matching locator
 */
export const nth = (locator: Locator, index: number): Effect.Effect<Locator> =>
  Effect.gen(function* () {
    return locator.nth(index)
  })

/**
 * Filter locators by text content
 */
export const filterByText = (
  locator: Locator,
  text: string,
): Effect.Effect<Locator> =>
  Effect.gen(function* () {
    return locator.filter({ hasText: text })
  })

/**
 * Get locator by test ID within parent locator
 */
export const getByTestId = (
  locator: Locator,
  testId: string,
): Effect.Effect<Locator> =>
  Effect.gen(function* () {
    return locator.getByTestId(testId)
  })
