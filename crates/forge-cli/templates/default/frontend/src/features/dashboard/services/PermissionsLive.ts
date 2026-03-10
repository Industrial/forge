/**
 * Live implementation of Permissions service using HttpClient.
 */

import { HttpClientRequest } from '@effect/platform'
import { Effect, Layer } from 'effect'
import { AuthenticatedHttpClient } from '@/services/AuthenticatedHttpClient'
import { Permissions } from './Permissions'
import type { PermissionsService } from './Permissions'
import { parseError } from '@/lib/parseError'
import { Assignment } from '../domain/Assignment'

const PermissionsLive = Layer.effect(
  Permissions,
  Effect.gen(function* () {
    const client = yield* AuthenticatedHttpClient

    const getData: PermissionsService['getData'] = () =>
      Effect.gen(function* () {
        yield* Effect.logTrace('PermissionsLive.getData')
        const [assignRes, permRes] = yield* Effect.all([
          client.execute(HttpClientRequest.get('/api/auth/role-permissions')),
          client.execute(HttpClientRequest.get('/api/auth/permissions')),
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
        const req = HttpClientRequest.post('/api/auth/role-permissions').pipe(
          HttpClientRequest.bodyUnsafeJson(body),
        )
        const response = yield* client.execute(req)
        const resBody = yield* response.json
        if (response.status === 403) {
          return yield* Effect.fail(
            new Error('You do not have permission to manage permissions.'),
          )
        }
        if (response.status < 200 || response.status >= 300) {
          return yield* Effect.fail(new Error(parseError(resBody)))
        }
        yield* Effect.logDebug('PermissionsLive.add: success')
      })

    const del: PermissionsService['delete'] = (body) =>
      Effect.gen(function* () {
        yield* Effect.logTrace('PermissionsLive.delete')
        yield* Effect.logDebug(
          `PermissionsLive.delete: scope=${body.scope}, role_name=${body.role_name}, permission_key=${body.permission_key}`,
        )
        const req = HttpClientRequest.del('/api/auth/role-permissions').pipe(
          HttpClientRequest.bodyUnsafeJson(body),
        )
        const response = yield* client.execute(req)
        const resBody = yield* response.json
        if (response.status === 403) {
          return yield* Effect.fail(
            new Error('You do not have permission to manage permissions.'),
          )
        }
        if (response.status < 200 || response.status >= 300) {
          return yield* Effect.fail(new Error(parseError(resBody)))
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
