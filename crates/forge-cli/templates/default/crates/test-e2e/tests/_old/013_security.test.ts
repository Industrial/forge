import { test, expect } from '@playwright/test'
import { API_BASE_URL } from '../../playwright.config.js'
import { prebuiltExists, assertProjectLayout } from '../../helpers/prebuilt.js'

function assertHeader(
  headers: Headers,
  name: string,
  expectedSubstr: string,
): void {
  const value = headers.get(name) ?? ''
  expect(value.toLowerCase()).toContain(expectedSubstr.toLowerCase())
}

test.describe('e2e security headers', () => {
  test('OWASP-aligned headers on /healthz', async () => {
    expect(prebuiltExists(), 'run bin/test-e2e first').toBe(true)
    assertProjectLayout()
    const base = API_BASE_URL.replace(/\/$/, '')
    const res = await fetch(`${base}/healthz`)
    expect(res.ok).toBe(true)
    const h = res.headers
    assertHeader(h, 'x-content-type-options', 'nosniff')
    assertHeader(h, 'x-frame-options', 'DENY')
    assertHeader(h, 'referrer-policy', 'strict-origin-when-cross-origin')
    assertHeader(h, 'content-security-policy', 'frame-ancestors')
    assertHeader(h, 'permissions-policy', 'geolocation=()')
    assertHeader(h, 'cross-origin-resource-policy', 'same-site')
  })

  test('browser loads app root', async ({ page }) => {
    expect(prebuiltExists()).toBe(true)
    assertProjectLayout()
    await page.goto('/')
    await expect(page.locator('#root')).toBeVisible()
  })
})
