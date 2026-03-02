import { test, expect } from '@playwright/test';
import {
  prebuiltExists,
  assertProjectLayout,
} from '../helpers/prebuilt.js';

test.describe('e2e websocket', () => {
  test('ws-demo echo', async ({ page }) => {
    expect(prebuiltExists(), 'run bin/test-e2e first').toBe(true);
    assertProjectLayout();
    await page.goto('/ws-demo');
    await expect(page.locator('#root')).toBeVisible({ timeout: 30_000 });
    const msg = 'ping';
    await page.locator('input').fill(msg);
    await page.locator('button').click();
    await expect(page.locator('body')).toContainText(msg, { timeout: 10_000 });
  });
});
