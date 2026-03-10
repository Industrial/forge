/**
 * Live implementation of Dashboard service using HttpClient.
 */

import { HttpClientRequest } from '@effect/platform'
import { Effect, Layer } from 'effect'
import { AuthenticatedHttpClient } from '@/services/AuthenticatedHttpClient'
import { Dashboard } from './Dashboard'
import type { DashboardService } from './Dashboard'
import { Organization } from '../domain/Organization'
import { parseError } from '@/lib/parseError'
import { DashboardRole } from '../domain/DashboardRole'

const DashboardLive = Layer.effect(
  Dashboard,
  Effect.gen(function* () {
    const client = yield* AuthenticatedHttpClient

    const getOrganizations: DashboardService['getOrganizations'] = () =>
      Effect.gen(function* () {
        yield* Effect.logTrace('DashboardLive.getOrganizations')
        const response = yield* client.execute(
          HttpClientRequest.get('/api/auth/organizations'),
        )
        const body = yield* response.json
        if (response.status < 200 || response.status >= 300) {
          return yield* Effect.fail(new Error(parseError(body)))
        }
        const data = body as {
          organizations?: Array<{
            id: string
            name: string
            slug: string
            created_at?: string
            updated_at?: string
          }>
        }
        const raw = data.organizations ?? []
        const result = raw.map(
          (o) =>
            new Organization({
              id: o.id,
              name: o.name,
              slug: o.slug,
              created_at: o.created_at,
              updated_at: o.updated_at,
            }),
        ) as readonly Organization[]
        yield* Effect.logDebug(
          `DashboardLive.getOrganizations: count=${result.length}`,
        )
        return result
      })

    const getRolesByOrg: DashboardService['getRolesByOrg'] = (orgId: string) =>
      Effect.gen(function* () {
        yield* Effect.logTrace('DashboardLive.getRolesByOrg')
        yield* Effect.logDebug(`DashboardLive.getRolesByOrg: orgId=${orgId}`)
        if (!orgId) {
          return [] as readonly DashboardRole[]
        }
        const url = `/api/auth/roles?org_id=${encodeURIComponent(orgId)}`
        const response = yield* client.execute(HttpClientRequest.get(url))
        const body = yield* response.json
        if (response.status < 200 || response.status >= 300) {
          return [] as readonly DashboardRole[]
        }
        const data = body as {
          roles?: Array<{
            id: string
            name: string
            display_name: string | null
          }>
        }
        const raw = data.roles ?? []
        const result = raw.map(
          (r) =>
            new DashboardRole({
              id: r.id,
              name: r.name,
              display_name: r.display_name,
            }),
        ) as readonly DashboardRole[]
        yield* Effect.logDebug(
          `DashboardLive.getRolesByOrg: count=${result.length}`,
        )
        return result
      })

    return {
      getOrganizations,
      getRolesByOrg,
    }
  }),
)

export { DashboardLive }
