/**
 * TokenStorage implementation using localStorage.
 */
import { Effect, Option } from 'effect'
import { Layer } from 'effect'

import { TokenStorage } from '../services/TokenStorage'

const TOKEN_KEY = 'token'
const ORG_ID_KEY = 'currentOrgId'
const ROLE_ID_KEY = 'currentRoleId'

function getStorage(): Storage | null {
  if (typeof window === 'undefined') {
    return null
  }
  return window.localStorage
}

export const TokenStorageLive = Layer.succeed(TokenStorage, {
  getToken: () =>
    Effect.gen(function* () {
      yield* Effect.logTrace('TokenStorageLive.getToken')

      const s = getStorage()
      const token = s?.getItem(TOKEN_KEY) ?? null
      const result =
        token != null && token !== '' ? Option.some(token) : Option.none()

      yield* Effect.logDebug(
        `TokenStorageLive.getToken: hasToken=${Option.isSome(result)}`,
      )

      return result
    }),

  setToken: (token: string) =>
    Effect.gen(function* () {
      yield* Effect.logTrace('TokenStorageLive.setToken')

      const s = getStorage()
      if (s) {
        s.setItem(TOKEN_KEY, token)
        yield* Effect.logDebug('TokenStorageLive.setToken: wrote token')
      } else {
        yield* Effect.logDebug('TokenStorageLive.setToken: no storage')
      }
    }),

  clearToken: () =>
    Effect.gen(function* () {
      yield* Effect.logTrace('TokenStorageLive.clearToken')

      const s = getStorage()
      if (s) {
        s.removeItem(TOKEN_KEY)
        yield* Effect.logDebug('TokenStorageLive.clearToken: removed token')
      } else {
        yield* Effect.logDebug('TokenStorageLive.clearToken: no storage')
      }
    }),

  getScope: () =>
    Effect.gen(function* () {
      yield* Effect.logTrace('TokenStorageLive.getScope')

      const s = getStorage()
      const orgId = s?.getItem(ORG_ID_KEY) ?? null
      const roleId = s?.getItem(ROLE_ID_KEY) ?? null
      const result =
        orgId != null && orgId !== '' && roleId != null
          ? Option.some({ organizationId: orgId, roleId })
          : Option.none()

      yield* Effect.logDebug(
        `TokenStorageLive.getScope: hasScope=${Option.isSome(result)}`,
      )

      return result
    }),

  setScope: (organizationId: string, roleId: string) =>
    Effect.gen(function* () {
      yield* Effect.logTrace('TokenStorageLive.setScope')

      const s = getStorage()
      if (s) {
        s.setItem(ORG_ID_KEY, organizationId)
        s.setItem(ROLE_ID_KEY, roleId)
        yield* Effect.logDebug(
          `TokenStorageLive.setScope: organizationId=${organizationId}, roleId=${roleId}`,
        )
      } else {
        yield* Effect.logDebug('TokenStorageLive.setScope: no storage')
      }
    }),

  clearScope: () =>
    Effect.gen(function* () {
      yield* Effect.logTrace('TokenStorageLive.clearScope')

      const s = getStorage()
      if (s) {
        s.removeItem(ORG_ID_KEY)
        s.removeItem(ROLE_ID_KEY)
        yield* Effect.logDebug('TokenStorageLive.clearScope: removed scope')
      } else {
        yield* Effect.logDebug('TokenStorageLive.clearScope: no storage')
      }
    }),
})
