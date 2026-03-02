import { test, expect } from '@playwright/test';
import {
  prebuiltExists,
  assertProjectLayout,
  pathExists,
  readFile,
} from '../helpers/prebuilt.js';

test.describe('e2e authz', () => {
  test('authz layout and protected route: unauthed redirect, register+login then dashboard', async ({
    page,
  }) => {
    expect(prebuiltExists(), 'run bin/test-e2e first').toBe(true);
    assertProjectLayout();
    expect(pathExists('crates/db/src/models/organization.rs')).toBe(true);
    expect(pathExists('crates/db/src/models/membership.rs')).toBe(true);
    const userModel = readFile('crates/db/src/models/user.rs');
    expect(userModel).toContain('impl AuthzContext');
    const authHandlers = readFile('crates/app/src/handlers/auth.rs');
    expect(authHandlers).toMatch(/admin/);
    expect(
      authHandlers.includes('is_admin') || authHandlers.includes('record_authz_denied')
    ).toBe(true);

    await page.goto('/dashboard');
    await expect(page).toHaveURL(/\/login/, { timeout: 15_000 });

    const email = `authz-e2e-${Date.now()}@test.com`;
    const password = 'password';

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
