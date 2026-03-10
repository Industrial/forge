/**
 * Live implementation of Roles service using HttpClient.
 */

import { HttpClientRequest } from '@effect/platform'
import { Effect, Layer } from 'effect'
import { AuthenticatedHttpClient } from '@/services/AuthenticatedHttpClient'
import { Roles } from './Roles'
import type { RolesService } from './Roles'
import { parseError } from '@/lib/parseError'
import { Role } from '../domain/Role'
import { DashboardRole } from '../domain/DashboardRole'

const RolesLive = Layer.effect(
  Roles,
  Effect.gen(function* () {
    const client = yield* AuthenticatedHttpClient

    const list: RolesService['list'] = () =>
      Effect.gen(function* () {
        yield* Effect.logTrace('RolesLive.list')
        const response = yield* client.execute(
          HttpClientRequest.get('/api/auth/roles'),
        )
        const body = yield* response.json
        if (response.status === 403) {
          return yield* Effect.fail(
            new Error('You do not have permission to view roles.'),
          )
        }
        if (response.status < 200 || response.status >= 300) {
          return yield* Effect.fail(new Error(parseError(body)))
        }
        const data = body as {
          roles?: Array<{
            id: string
            org_id: string
            org_name?: string
            name: string
            display_name: string | null
            created_at?: string
            updated_at?: string
          }>
        }
        const raw = data.roles ?? []
        const result = raw.map(
          (r) =>
            new Role({
              id: r.id,
              org_id: r.org_id,
              org_name: r.org_name,
              name: r.name,
              display_name: r.display_name,
              created_at: r.created_at,
              updated_at: r.updated_at,
            }),
        ) as readonly Role[]
        yield* Effect.logDebug(`RolesLive.list: count=${result.length}`)
        return result
      })

    const listByOrg: RolesService['listByOrg'] = (orgId: string) =>
      Effect.gen(function* () {
        yield* Effect.logTrace('RolesLive.listByOrg')
        yield* Effect.logDebug(`RolesLive.listByOrg: orgId=${orgId}`)
        if (!orgId) return [] as readonly DashboardRole[]
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
        yield* Effect.logDebug(`RolesLive.listByOrg: count=${result.length}`)
        return result
      })

    const create: RolesService['create'] = (body) =>
      Effect.gen(function* () {
        yield* Effect.logTrace('RolesLive.create')
        yield* Effect.logDebug(
          `RolesLive.create: org_id=${body.org_id}, name=${body.name}`,
        )
        const response = yield* client.execute(
          HttpClientRequest.post('/api/auth/roles').pipe(
            HttpClientRequest.bodyUnsafeJson(body),
          ),
        )
        const resBody = yield* response.json
        if (response.status === 403) {
          return yield* Effect.fail(
            new Error('You do not have permission to create roles.'),
          )
        }
        if (response.status < 200 || response.status >= 300) {
          return yield* Effect.fail(new Error(parseError(resBody)))
        }
        yield* Effect.logDebug('RolesLive.create: success')
      })

    const update: RolesService['update'] = (body) =>
      Effect.gen(function* () {
        yield* Effect.logTrace('RolesLive.update')
        yield* Effect.logDebug(`RolesLive.update: id=${body.id}`)
        const response = yield* client.execute(
          HttpClientRequest.patch(`/api/auth/roles/${encodeURIComponent(body.id)}`).pipe(
            HttpClientRequest.bodyUnsafeJson({
              name: body.name,
              display_name: body.display_name,
            }),
          ),
        )
        const resBody = yield* response.json
        if (response.status === 403) {
          return yield* Effect.fail(
            new Error('You do not have permission to update roles.'),
          )
        }
        if (response.status < 200 || response.status >= 300) {
          return yield* Effect.fail(new Error(parseError(resBody)))
        }
        yield* Effect.logDebug('RolesLive.update: success')
      })

    const del: RolesService['delete'] = (id: string) =>
      Effect.gen(function* () {
        yield* Effect.logTrace('RolesLive.delete')
        yield* Effect.logDebug(`RolesLive.delete: id=${id}`)
        const response = yield* client.execute(
          HttpClientRequest.del(`/api/auth/roles/${encodeURIComponent(id)}`),
        )
        const resBody = yield* response.json
        if (response.status === 403) {
          return yield* Effect.fail(
            new Error('You do not have permission to delete roles.'),
          )
        }
        if (response.status < 200 || response.status >= 300) {
          return yield* Effect.fail(new Error(parseError(resBody)))
        }
        yield* Effect.logDebug('RolesLive.delete: success')
      })

    return {
      list,
      listByOrg,
      create,
      update,
      delete: del,
    }
  }),
)

export { RolesLive }
