/**
 * TokenStorage implementation using localStorage.
 */
import { Effect, Option } from 'effect'
import { Layer } from 'effect'

import { TokenStorage } from '@/services/TokenStorage'

const TOKEN_KEY = 'token'
const ORG_ID_KEY = 'currentOrgId'
const ROLE_ID_KEY = 'currentRoleId'

function getStorage(): Storage | null {
  if (typeof window === 'undefined') return null
  return window.localStorage
}

export const TokenStorageLive = Layer.succeed(TokenStorage, {
  getToken: () =>
    Effect.sync(() => {
      const s = getStorage()
      const token = s?.getItem(TOKEN_KEY) ?? null
      return token != null && token !== '' ? Option.some(token) : Option.none()
    }),

  setToken: (token: string) =>
    Effect.sync(() => {
      const s = getStorage()
      if (s) s.setItem(TOKEN_KEY, token)
    }),

  clearToken: () =>
    Effect.sync(() => {
      const s = getStorage()
      if (s) s.removeItem(TOKEN_KEY)
    }),

  getScope: () =>
    Effect.sync(() => {
      const s = getStorage()
      const orgId = s?.getItem(ORG_ID_KEY) ?? null
      const roleId = s?.getItem(ROLE_ID_KEY) ?? null
      if (orgId != null && orgId !== '' && roleId != null) {
        return Option.some({ organizationId: orgId, roleId })
      }
      return Option.none()
    }),

  setScope: (organizationId: string, roleId: string) =>
    Effect.sync(() => {
      const s = getStorage()
      if (s) {
        s.setItem(ORG_ID_KEY, organizationId)
        s.setItem(ROLE_ID_KEY, roleId)
      }
    }),

  clearScope: () =>
    Effect.sync(() => {
      const s = getStorage()
      if (s) {
        s.removeItem(ORG_ID_KEY)
        s.removeItem(ROLE_ID_KEY)
      }
    }),
})
