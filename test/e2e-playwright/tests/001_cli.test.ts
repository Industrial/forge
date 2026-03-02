import { test, expect } from '@playwright/test';
import {
  prebuiltExists,
  assertProjectLayout,
} from '../helpers/prebuilt.js';

test.describe('e2e prebuilt project layout', () => {
  test('prebuilt project exists and has expected layout', () => {
    expect(prebuiltExists(), 'run bin/test-e2e first').toBe(true);
    assertProjectLayout();
  });

  test('browser loads app root (#root)', async ({ page }) => {
    expect(prebuiltExists(), 'run bin/test-e2e first').toBe(true);
    assertProjectLayout();
    await page.goto('/');
    await expect(page.locator('#root')).toBeVisible({ timeout: 30_000 });
  });
});
