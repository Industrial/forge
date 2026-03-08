import { Page, Locator } from '@playwright/test'
import { expect } from '@playwright/test'
import { Effect } from 'effect'

/**
 * Effect.ts helpers for Playwright expect assertions
 * These wrap Playwright's expect API in Effect.ts for use with yield*
 */

/**
 * Assert locator is visible
 */
export const toBeVisible = (locator: Locator): Effect.Effect<void> =>
  Effect.promise(() => expect(locator).toBeVisible())

/**
 * Assert locator is not visible
 */
export const notToBeVisible = (locator: Locator): Effect.Effect<void> =>
  Effect.promise(() => expect(locator).not.toBeVisible())

/**
 * Assert locator is enabled
 */
export const toBeEnabled = (locator: Locator): Effect.Effect<void> =>
  Effect.promise(() => expect(locator).toBeEnabled())

/**
 * Assert locator is disabled
 */
export const toBeDisabled = (locator: Locator): Effect.Effect<void> =>
  Effect.promise(() => expect(locator).toBeDisabled())

/**
 * Assert locator is checked
 */
export const toBeChecked = (locator: Locator): Effect.Effect<void> =>
  Effect.promise(() => expect(locator).toBeChecked())

/**
 * Assert locator is not checked
 */
export const notToBeChecked = (locator: Locator): Effect.Effect<void> =>
  Effect.promise(() => expect(locator).not.toBeChecked())

/**
 * Assert page has URL
 */
export const toHaveURL = (
  page: Page,
  url: string | RegExp,
): Effect.Effect<void> => Effect.promise(() => expect(page).toHaveURL(url))

/**
 * Assert locator has text
 */
export const toHaveText = (
  locator: Locator,
  text: string | RegExp,
): Effect.Effect<void> => Effect.promise(() => expect(locator).toHaveText(text))

/**
 * Assert locator contains text
 */
export const toContainText = (
  locator: Locator,
  text: string | RegExp,
): Effect.Effect<void> =>
  Effect.promise(() => expect(locator).toContainText(text))

/**
 * Assert locator has value
 */
export const toHaveValue = (
  locator: Locator,
  value: string | RegExp,
): Effect.Effect<void> =>
  Effect.promise(() => expect(locator).toHaveValue(value))

/**
 * Assert locator has attribute
 */
export const toHaveAttribute = (
  locator: Locator,
  name: string,
  value: string | RegExp,
): Effect.Effect<void> =>
  Effect.promise(() => expect(locator).toHaveAttribute(name, value))

/**
 * Assert locator has count
 */
export const toHaveCount = (
  locator: Locator,
  count: number,
): Effect.Effect<void> =>
  Effect.promise(() => expect(locator).toHaveCount(count))

/**
 * Assert page has title
 */
export const toHaveTitle = (
  page: Page,
  title: string | RegExp,
): Effect.Effect<void> => Effect.promise(() => expect(page).toHaveTitle(title))

/**
 * Assert locator is focused
 */
export const toBeFocused = (locator: Locator): Effect.Effect<void> =>
  Effect.promise(() => expect(locator).toBeFocused())

/**
 * Assert locator has CSS class
 */
export const toHaveClass = (
  locator: Locator,
  className: string | RegExp | (string | RegExp)[],
): Effect.Effect<void> =>
  Effect.promise(() => expect(locator).toHaveClass(className))
