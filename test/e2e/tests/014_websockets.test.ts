import { test, expect } from '@playwright/test'
import {
  prebuiltExists,
  assertProjectLayout,
  readFile,
} from '../helpers/prebuilt.js'

test.describe('e2e websocket', () => {
  test.skip('ws-demo echo', async ({ page }) => {
    expect(prebuiltExists(), 'run bin/test-e2e first').toBe(true)
    assertProjectLayout()
    const appTsx = readFile('frontend/src/App.tsx')
    expect(appTsx, 'prebuilt App.tsx should include ws-demo route').toContain(
      'ws-demo',
    )
    await page.goto('/ws-demo')
    await expect(page).toHaveURL(/\/ws-demo/)
    await expect(page.getByTestId('ws-demo-root')).toBeVisible()
    const msg = 'ping'
    await page.getByTestId('ws-demo-input').fill(msg)
    await page.getByTestId('ws-demo-send').click()
    await expect(page.getByTestId('ws-demo-messages')).toContainText(msg)
  })
})
