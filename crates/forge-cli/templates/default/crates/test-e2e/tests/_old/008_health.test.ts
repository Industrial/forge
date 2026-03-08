import { test, expect } from '@playwright/test'
import { API_BASE_URL } from '../../playwright.config.js'
import { prebuiltExists, assertProjectLayout } from '../../helpers/prebuilt.js'

async function expectHealthBodyOk(url: string): Promise<void> {
  const res = await fetch(url)
  expect(res.ok).toBe(true)
  const text = await res.text()
  expect(text).toContain('ok')
  expect(text).not.toMatch(/components|database/)
}

test.describe('e2e health endpoints', () => {
  test('prebuilt layout and healthz, livez, readyz return ok', async () => {
    expect(prebuiltExists(), 'run bin/test-e2e first').toBe(true)
    assertProjectLayout()
    const base = API_BASE_URL.replace(/\/$/, '')
    await expectHealthBodyOk(`${base}/healthz`)
    await expectHealthBodyOk(`${base}/livez`)
    await expectHealthBodyOk(`${base}/readyz`)
  })
})
