import { test, expect } from '@playwright/test'
import {
  prebuiltExists,
  assertProjectLayout,
  readFile,
} from '../helpers/prebuilt.js'

test.describe('e2e prebuilt config', () => {
  test('config files exist and have expected content', () => {
    expect(prebuiltExists(), 'run bin/test-e2e first').toBe(true)
    assertProjectLayout()
    const appToml = readFile('config/app.toml')
    expect(appToml).toContain('[app]')
    expect(appToml).toContain('[server]')
    const dbToml = readFile('config/db.toml')
    expect(dbToml).toContain('[database]')
  })

  test('browser loads app root', async ({ page }) => {
    expect(prebuiltExists()).toBe(true)
    assertProjectLayout()
    await page.goto('/')
    await expect(page.locator('#root')).toBeVisible()
  })
})
