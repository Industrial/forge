/**
 * Live implementation of Permissions service using HttpClient.
 */

import { HttpClient, HttpClientRequest } from '@effect/platform'
import { Effect, Layer } from 'effect'
import { Permissions } from './Permissions'
import type { PermissionsService } from './Permissions'
import { Assignment } from '../domain/Assignment'

function parseErr(body: unknown): string {
  if (typeof body === 'object' && body !== null && 'error' in body) {
    return String((body as { error: unknown }).error)
  }
  return 'Request failed.'
}

const PermissionsLive = Layer.effect(
  Permissions,
  Effect.gen(function* () {
    const client = yield* HttpClient.HttpClient

    const getData: PermissionsService['getData'] = () =>
      Effect.gen(function* () {
        yield* Effect.logTrace('PermissionsLive.getData')
        const [assignRes, permRes] = yield* Effect.all([
          client.execute(
            HttpClientRequest.get('/api/dashboard/role-permissions'),
          ),
          client.execute(HttpClientRequest.get('/api/dashboard/permissions')),
        ])
        const assignBody = yield* assignRes.json
        const permBody = yield* permRes.json
        if (assignRes.status === 403 || permRes.status === 403) {
          return yield* Effect.fail(
            new Error('You do not have permission to manage permissions.'),
          )
        }
        if (
          assignRes.status >= 200 &&
          assignRes.status < 300 &&
          permRes.status >= 200 &&
          permRes.status < 300
        ) {
          const rawAssignments =
            (assignBody as { assignments?: unknown[] }).assignments ?? []
          const assignments = rawAssignments.map(
            (a) =>
              new Assignment({
                scope: (a as { scope: string }).scope,
                role_name: (a as { role_name: string }).role_name,
                permission_key: (a as { permission_key: string })
                  .permission_key,
                org_id: (a as { org_id?: string | null }).org_id,
              }),
          ) as readonly Assignment[]
          const permissions =
            (permBody as { permissions?: string[] }).permissions ?? []
          const result = { assignments, permissions }
          yield* Effect.logDebug(
            `PermissionsLive.getData: assignments=${result.assignments.length}, permissions=${result.permissions.length}`,
          )
          return result
        }
        return yield* Effect.fail(new Error('Failed to load data.'))
      })

    const add: PermissionsService['add'] = (body) =>
      Effect.gen(function* () {
        yield* Effect.logTrace('PermissionsLive.add')
        yield* Effect.logDebug(
          `PermissionsLive.add: scope=${body.scope}, role_name=${body.role_name}, permission_key=${body.permission_key}`,
        )
        const req = HttpClientRequest.post(
          '/api/dashboard/role-permissions',
        ).pipe(HttpClientRequest.bodyUnsafeJson(body))
        const response = yield* client.execute(req)
        const resBody = yield* response.json
        if (response.status === 403) {
          return yield* Effect.fail(
            new Error('You do not have permission to manage permissions.'),
          )
        }
        if (response.status < 200 || response.status >= 300) {
          return yield* Effect.fail(new Error(parseErr(resBody)))
        }
        yield* Effect.logDebug('PermissionsLive.add: success')
      })

    const del: PermissionsService['delete'] = (body) =>
      Effect.gen(function* () {
        yield* Effect.logTrace('PermissionsLive.delete')
        yield* Effect.logDebug(
          `PermissionsLive.delete: scope=${body.scope}, role_name=${body.role_name}, permission_key=${body.permission_key}`,
        )
        const req = HttpClientRequest.del(
          '/api/dashboard/role-permissions',
        ).pipe(HttpClientRequest.bodyUnsafeJson(body))
        const response = yield* client.execute(req)
        const resBody = yield* response.json
        if (response.status === 403) {
          return yield* Effect.fail(
            new Error('You do not have permission to manage permissions.'),
          )
        }
        if (response.status < 200 || response.status >= 300) {
          return yield* Effect.fail(new Error(parseErr(resBody)))
        }
        yield* Effect.logDebug('PermissionsLive.delete: success')
      })

    return {
      getData,
      add,
      delete: del,
    }
  }),
)

export { PermissionsLive }
