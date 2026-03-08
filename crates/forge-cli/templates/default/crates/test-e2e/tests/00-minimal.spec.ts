/**
 * Minimal test to debug Playwright hanging issue
 */
import { test, expect } from '@playwright/test'

test('minimal test', async ({ page }) => {
  await page.goto('about:blank')
  expect(page.url()).toBe('about:blank')
})
