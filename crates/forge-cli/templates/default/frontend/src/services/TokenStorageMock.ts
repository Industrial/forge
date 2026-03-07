/**
 * In-memory TokenStorage for tests. Control token and scope via the returned API.
 */
import { Effect, Option } from 'effect'
import { Layer } from 'effect'

import { TokenStorage } from '@/services/TokenStorage'

export function makeTokenStorageMock(): {
  layer: Layer.Layer<TokenStorage>
  setToken: (token: string) => void
  clearToken: () => void
  setScope: (organizationId: string, roleId: string) => void
  clearScope: () => void
} {
  let token: string | null = null
  let organizationId: string | null = null
  let roleId: string | null = null

  const storage: TokenStorage = {
    getToken: () =>
      Effect.sync(() =>
        token != null && token !== '' ? Option.some(token) : Option.none(),
      ),
    setToken: (t: string) => Effect.sync(() => { token = t }),
    clearToken: () => Effect.sync(() => { token = null }),
    getScope: () =>
      Effect.sync(() =>
        organizationId != null && roleId != null
          ? Option.some({
              organizationId,
              roleId,
            })
          : Option.none(),
      ),
    setScope: (org: string, role: string) =>
      Effect.sync(() => {
        organizationId = org
        roleId = role
      }),
    clearScope: () =>
      Effect.sync(() => {
        organizationId = null
        roleId = null
      }),
  }

  const layer = Layer.succeed(TokenStorage, storage)
  return {
    layer,
    setToken: (t: string) => { token = t },
    clearToken: () => { token = null },
    setScope: (org: string, role: string) => {
      organizationId = org
      roleId = role
    },
    clearScope: () => {
      organizationId = null
      roleId = null
    },
  }
}
