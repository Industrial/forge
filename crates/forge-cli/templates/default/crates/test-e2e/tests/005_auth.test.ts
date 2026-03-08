import { test, expect } from '@playwright/test'
import {
  prebuiltExists,
  assertProjectLayout,
  assertAuthLayout,
} from '../helpers/prebuilt.js'

const SEED_PASSWORD = 'password'

test.describe('e2e auth', () => {
  test.beforeEach(() => {
    expect(prebuiltExists(), 'run bin/test-e2e first').toBe(true)
    assertProjectLayout()
    assertAuthLayout()
  })

  test('auth layout: project has auth, org, membership, user AuthzContext, admin route', () => {
    // assertAuthLayout in beforeEach covers this
  })

  test('unauthed /dashboard redirects to login', async ({ page }) => {
    await page.goto('/dashboard')
    await expect(page).toHaveURL(/\/login/)
  })

  test('login fails with wrong password and shows error', async ({ page }) => {
    await page.goto('/login')
    await page
      .getByTestId('login-email')
      .locator('input')
      .fill('admin@admin.com')
    await page
      .getByTestId('login-password')
      .locator('input')
      .fill('wrongpassword')
    await page.getByTestId('login-submit').click()
    await expect(page.getByTestId('login-error')).toBeVisible()
    await expect(page).not.toHaveURL(/\/dashboard/)
  })

  test('login with seed user then dashboard visible', async ({ page }) => {
    await page.goto('/login')
    await page
      .getByTestId('login-email')
      .locator('input')
      .fill('admin@admin.com')
    await page
      .getByTestId('login-password')
      .locator('input')
      .fill(SEED_PASSWORD)
    await page.getByTestId('login-submit').click()
    await expect(page).toHaveURL(/\/dashboard/)
    await expect(page.getByTestId('dashboard-heading')).toContainText(
      'Dashboard',
    )
  })

  test('logout then /dashboard redirects to login', async ({ page }) => {
    await page.goto('/login')
    await page
      .getByTestId('login-email')
      .locator('input')
      .fill('admin@admin.com')
    await page
      .getByTestId('login-password')
      .locator('input')
      .fill(SEED_PASSWORD)
    await page.getByTestId('login-submit').click()
    await expect(page).toHaveURL(/\/dashboard/)
    await page.goto('/api/auth/logout')
    await page.goto('/dashboard')
    await expect(page).toHaveURL(/\/login/)
  })

  test('global admin can access /api/auth/admin', async ({ page }) => {
    await page.goto('/login')
    await page
      .getByTestId('login-email')
      .locator('input')
      .fill('admin@admin.com')
    await page
      .getByTestId('login-password')
      .locator('input')
      .fill(SEED_PASSWORD)
    await page.getByTestId('login-submit').click()
    await expect(page).toHaveURL(/\/dashboard/)
    await page.goto('/api/auth/admin')
    await expect(page.locator('body')).toContainText('access granted')
  })

  test('non-admin cannot access /api/auth/admin (Forbidden)', async ({
    page,
  }) => {
    await page.goto('/login')
    await page
      .getByTestId('login-email')
      .locator('input')
      .fill('viewer@default.org')
    await page
      .getByTestId('login-password')
      .locator('input')
      .fill(SEED_PASSWORD)
    await page.getByTestId('login-submit').click()
    await expect(page).toHaveURL(/\/dashboard/)
    await page.goto('/api/auth/admin')
    await expect(page.locator('body')).toContainText('Forbidden')
  })

  test('register then login then dashboard (full flow)', async ({ page }) => {
    const email = `auth-e2e-${Date.now()}@test.com`
    const password = 'password'

    await page.goto('/register')
    await page.getByTestId('register-email').locator('input').fill(email)
    await page.getByTestId('register-password').locator('input').fill(password)
    await page.getByTestId('register-submit').click()
    await expect(page).toHaveURL(/\/login/)

    await page.getByTestId('login-email').locator('input').fill(email)
    await page.getByTestId('login-password').locator('input').fill(password)
    await page.getByTestId('login-submit').click()
    await expect(page).toHaveURL(/\/dashboard/)
    await expect(page.getByTestId('dashboard-heading')).toContainText(
      'Dashboard',
    )
  })
})
