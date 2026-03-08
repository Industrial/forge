import { test, expect } from '@playwright/test'
import {
  prebuiltExists,
  assertProjectLayout,
  readFile,
} from '../helpers/prebuilt.js'

test.describe('e2e prebuilt database config', () => {
  test('db config exists and contains sqlite', () => {
    expect(prebuiltExists(), 'run bin/test-e2e first').toBe(true)
    assertProjectLayout()
    const dbToml = readFile('config/db.toml')
    expect(dbToml).toContain('[database]')
    expect(dbToml).toMatch(/sqlite/)
  })

  test('browser loads app root', async ({ page }) => {
    expect(prebuiltExists()).toBe(true)
    assertProjectLayout()
    await page.goto('/')
    await expect(page.locator('#root')).toBeVisible()
  })
})
