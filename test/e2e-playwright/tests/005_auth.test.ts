import { test, expect } from '@playwright/test';
import {
  prebuiltExists,
  assertProjectLayout,
  assertAuthLayout,
} from '../helpers/prebuilt.js';

const SEED_PASSWORD = 'password123';

test.describe('e2e auth', () => {
  test.beforeEach(() => {
    expect(prebuiltExists(), 'run bin/test-e2e first').toBe(true);
    assertProjectLayout();
    assertAuthLayout();
  });

  test('auth layout: project has auth, org, membership, user AuthzContext, admin route', () => {
    // assertAuthLayout in beforeEach covers this
  });

  test('unauthed /dashboard redirects to login', async ({ page }) => {
    await page.goto('/dashboard');
    await expect(page).toHaveURL(/\/login/, { timeout: 30_000 });
  });

  test('login fails with wrong password and shows error', async ({ page }) => {
    await page.goto('/login');
    await page.getByRole('textbox', { name: /email/i }).fill('admin@admin.com');
    await page.getByRole('textbox', { name: /password/i }).fill('wrongpassword');
    await page.getByRole('button', { name: /log in/i }).click();
    await expect(page.getByTestId('login-error')).toBeVisible({ timeout: 15_000 });
    await expect(page).not.toHaveURL(/\/dashboard/);
  });

  test('login with seed user then dashboard visible', async ({ page }) => {
    await page.goto('/login');
    await page.getByRole('textbox', { name: /email/i }).fill('admin@admin.com');
    await page.getByRole('textbox', { name: /password/i }).fill(SEED_PASSWORD);
    await page.getByRole('button', { name: /log in/i }).click();
    await expect(page).toHaveURL(/\/dashboard/, { timeout: 15_000 });
    await expect(page.getByTestId('dashboard-heading')).toContainText('Dashboard', {
      timeout: 15_000,
    });
  });

  test('logout then /dashboard redirects to login', async ({ page }) => {
    await page.goto('/login');
    await page.getByRole('textbox', { name: /email/i }).fill('admin@admin.com');
    await page.getByRole('textbox', { name: /password/i }).fill(SEED_PASSWORD);
    await page.getByRole('button', { name: /log in/i }).click();
    await expect(page).toHaveURL(/\/dashboard/, { timeout: 15_000 });
    await page.goto('/api/auth/logout');
    await page.goto('/dashboard');
    await expect(page).toHaveURL(/\/login/, { timeout: 15_000 });
  });

  test('global admin can access /api/auth/admin', async ({ page }) => {
    await page.goto('/login');
    await page.getByRole('textbox', { name: /email/i }).fill('admin@admin.com');
    await page.getByRole('textbox', { name: /password/i }).fill(SEED_PASSWORD);
    await page.getByRole('button', { name: /log in/i }).click();
    await expect(page).toHaveURL(/\/dashboard/, { timeout: 15_000 });
    await page.goto('/api/auth/admin');
    await expect(page.locator('body')).toContainText('access granted', { timeout: 10_000 });
  });

  test('non-admin cannot access /api/auth/admin (Forbidden)', async ({ page }) => {
    await page.goto('/login');
    await page.getByRole('textbox', { name: /email/i }).fill('viewer@default.org');
    await page.getByRole('textbox', { name: /password/i }).fill(SEED_PASSWORD);
    await page.getByRole('button', { name: /log in/i }).click();
    await expect(page).toHaveURL(/\/dashboard/, { timeout: 15_000 });
    await page.goto('/api/auth/admin');
    await expect(page.locator('body')).toContainText('Forbidden', { timeout: 10_000 });
  });

  test('register then login then dashboard (full flow)', async ({ page }) => {
    const email = `auth-e2e-${Date.now()}@test.com`;
    const password = 'password123';

    await page.goto('/register');
    await page.getByRole('textbox', { name: /email/i }).fill(email);
    await page.getByRole('textbox', { name: /password/i }).fill(password);
    await page.getByRole('button', { name: /register|create account/i }).click();
    await expect(page).toHaveURL(/\/login/, { timeout: 15_000 });

    await page.getByRole('textbox', { name: /email/i }).fill(email);
    await page.getByRole('textbox', { name: /password/i }).fill(password);
    await page.getByRole('button', { name: /log in/i }).click();
    await expect(page).toHaveURL(/\/dashboard/, { timeout: 15_000 });
    await expect(page.getByTestId('dashboard-heading')).toContainText('Dashboard', {
      timeout: 15_000,
    });
  });
});
