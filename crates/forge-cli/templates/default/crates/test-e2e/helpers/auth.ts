/**
 * Auth helpers for E2E: read token/scope from the app's localStorage
 * so API requests to E2E_API_URL (different origin) can be authenticated.
 *
 * Keys match TokenStorageLive in the frontend (token, currentOrgId, currentRoleId).
 */
import type { Page } from '@playwright/test'

export interface AuthHeaders {
  Authorization: string
  'X-Organization-Id'?: string
  'X-Role-Id'?: string
}

/**
 * Read token and scope from the page's localStorage and return headers
 * suitable for API requests (Bearer token + scope headers).
 * Call after login and after dashboard has loaded (so scope is set).
 */
export async function getAuthHeadersFromPage(page: Page): Promise<AuthHeaders> {
  const storage = await page.evaluate(() => ({
    token: localStorage.getItem('token'),
    orgId: localStorage.getItem('currentOrgId'),
    roleId: localStorage.getItem('currentRoleId'),
  }))
  if (!storage.token) {
    throw new Error(
      'getAuthHeadersFromPage: no token in localStorage. Ensure login ran and dashboard has loaded.',
    )
  }
  const headers: AuthHeaders = {
    Authorization: `Bearer ${storage.token}`,
  }
  if (storage.orgId) headers['X-Organization-Id'] = storage.orgId
  if (storage.roleId) headers['X-Role-Id'] = storage.roleId
  return headers
}
