/**
 * Live implementation of Roles service using HttpClient.
 */

import { HttpClient, HttpClientRequest } from '@effect/platform'
import { Effect, Layer } from 'effect'
import { Roles } from './Roles'
import type { RolesService } from './Roles'
import { Role } from '../domain/Role'
import { DashboardRole } from '../domain/DashboardRole'

function parseErr(body: unknown): string {
  if (typeof body === 'object' && body !== null && 'error' in body) {
    return String((body as { error: unknown }).error)
  }
  return 'Request failed.'
}

const RolesLive = Layer.effect(
  Roles,
  Effect.gen(function* () {
    const client = yield* HttpClient.HttpClient

    const list: RolesService['list'] = () =>
      Effect.gen(function* () {
        const response = yield* client.execute(
          HttpClientRequest.get('/api/dashboard/roles'),
        )
        const body = yield* response.json
        if (response.status === 403) {
          return yield* Effect.fail(
            new Error('You do not have permission to view roles.'),
          )
        }
        if (response.status < 200 || response.status >= 300) {
          return yield* Effect.fail(new Error(parseErr(body)))
        }
        const data = body as {
          roles?: Array<{
            id: string
            org_id: string
            name: string
            display_name: string | null
            created_at?: string
            updated_at?: string
          }>
        }
        const raw = data.roles ?? []
        return raw.map(
          (r) =>
            new Role({
              id: r.id,
              org_id: r.org_id,
              name: r.name,
              display_name: r.display_name,
              created_at: r.created_at,
              updated_at: r.updated_at,
            }),
        ) as readonly Role[]
      })

    const listByOrg: RolesService['listByOrg'] = (orgId: string) =>
      Effect.gen(function* () {
        if (!orgId) return [] as readonly DashboardRole[]
        const url = `/api/dashboard/roles?org_id=${encodeURIComponent(orgId)}`
        const response = yield* client.execute(HttpClientRequest.get(url))
        const body = yield* response.json
        if (response.status < 200 || response.status >= 300) {
          return [] as readonly DashboardRole[]
        }
        const data = body as {
          roles?: Array<{ id: string; name: string; display_name: string | null }>
        }
        const raw = data.roles ?? []
        return raw.map(
          (r) =>
            new DashboardRole({
              id: r.id,
              name: r.name,
              display_name: r.display_name,
            }),
        ) as readonly DashboardRole[]
      })

    const create: RolesService['create'] = (body) =>
      Effect.gen(function* () {
        const response = yield* client.execute(
          HttpClientRequest.post('/api/dashboard/roles').pipe(
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
          return yield* Effect.fail(new Error(parseErr(resBody)))
        }
      })

    const update: RolesService['update'] = (body) =>
      Effect.gen(function* () {
        const response = yield* client.execute(
          HttpClientRequest.patch('/api/dashboard/roles').pipe(
            HttpClientRequest.bodyUnsafeJson(body),
          ),
        )
        const resBody = yield* response.json
        if (response.status === 403) {
          return yield* Effect.fail(
            new Error('You do not have permission to update roles.'),
          )
        }
        if (response.status < 200 || response.status >= 300) {
          return yield* Effect.fail(new Error(parseErr(resBody)))
        }
      })

    const del: RolesService['delete'] = (id: string) =>
      Effect.gen(function* () {
        const response = yield* client.execute(
          HttpClientRequest.del('/api/dashboard/roles').pipe(
            HttpClientRequest.bodyUnsafeJson({ id }),
          ),
        )
        const resBody = yield* response.json
        if (response.status === 403) {
          return yield* Effect.fail(
            new Error('You do not have permission to delete roles.'),
          )
        }
        if (response.status < 200 || response.status >= 300) {
          return yield* Effect.fail(new Error(parseErr(resBody)))
        }
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
