import { test, expect } from '@playwright/test'
import {
  prebuiltExists,
  assertProjectLayout,
  pathExists,
  readFile,
} from '../helpers/prebuilt.js'

test.describe('e2e prebuilt migrations layout', () => {
  test('migrations and workspace layout exist', () => {
    expect(prebuiltExists(), 'run bin/test-e2e first').toBe(true)
    assertProjectLayout()
    expect(pathExists('crates/db/src/migrations/mod.rs')).toBe(true)
    expect(pathExists('crates/db/src/models/mod.rs')).toBe(true)
    expect(pathExists('crates/db/src/seeds/mod.rs')).toBe(true)
    const cargoToml = readFile('Cargo.toml')
    expect(cargoToml).toContain('[workspace]')
  })

  test('browser loads app root', async ({ page }) => {
    expect(prebuiltExists()).toBe(true)
    assertProjectLayout()
    await page.goto('/')
    await expect(page.locator('#root')).toBeVisible()
  })
})
