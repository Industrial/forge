/**
 * Live implementation of Users service using HttpClient.
 */

import { HttpClientRequest } from '@effect/platform'
import { Effect, Layer } from 'effect'
import { AuthenticatedHttpClient } from '../../../services/AuthenticatedHttpClient'
import { Users } from './Users'
import type { UsersService } from './Users'
import { parseError } from '../../../lib/parseError'
import { User, UserMembership } from '../domain/User'

const UsersLive = Layer.effect(
  Users,
  Effect.gen(function* () {
    const client = yield* AuthenticatedHttpClient

    const list: UsersService['list'] = () =>
      Effect.gen(function* () {
        yield* Effect.logTrace('UsersLive.list')
        const response = yield* client.execute(
          HttpClientRequest.get('/api/auth/users'),
        )
        if (response.status === 401) {
          return yield* Effect.fail(
            new Error('Session expired or not logged in. Please log in again.'),
          )
        }
        if (response.status === 403) {
          return yield* Effect.fail(
            new Error('You do not have permission to view users.'),
          )
        }
        const body = yield* response.json
        if (response.status < 200 || response.status >= 300) {
          const msg =
            typeof body === 'object' && body !== null && 'error' in body
              ? String((body as { error: unknown }).error)
              : `HTTP ${response.status}`
          return yield* Effect.fail(new Error(msg))
        }
        const data = body as {
          users?: Array<{
            id: string
            email: string
            is_active: boolean
            is_admin: boolean
            created_at: string
            memberships?: Array<{
              org_id: string
              org_name: string
              roles: string[]
            }>
          }>
        }
        const raw = data.users ?? []
        const result = raw.map(
          (u) =>
            new User({
              id: u.id,
              email: u.email,
              is_active: u.is_active,
              is_admin: u.is_admin,
              created_at: u.created_at,
              memberships: (u.memberships ?? []).map(
                (m) =>
                  new UserMembership({
                    org_id: m.org_id,
                    org_name: m.org_name,
                    roles: m.roles ?? [],
                  }),
              ),
            }),
        ) as readonly User[]
        yield* Effect.logDebug(`UsersLive.list: count=${result.length}`)
        return result
      })

    const create: UsersService['create'] = (body) =>
      Effect.gen(function* () {
        yield* Effect.logTrace('UsersLive.create')
        yield* Effect.logDebug(
          `UsersLive.create: email=${body.email}, org_id=${body.org_id}`,
        )
        const request = HttpClientRequest.post('/api/auth/users').pipe(
          HttpClientRequest.bodyUnsafeJson({
            ...body,
            role_ids: [...body.role_ids],
          }),
        )
        const response = yield* client.execute(request)
        if (response.status === 403) {
          return yield* Effect.fail(
            new Error('You do not have permission to create users.'),
          )
        }
        const resBody = yield* response.json
        if (response.status < 200 || response.status >= 300) {
          return yield* Effect.fail(new Error(parseError(resBody)))
        }
        yield* Effect.logDebug('UsersLive.create: success')
      })

    const update: UsersService['update'] = (body) =>
      Effect.gen(function* () {
        yield* Effect.logTrace('UsersLive.update')
        yield* Effect.logDebug(`UsersLive.update: id=${body.id}`)
        const response = yield* client.execute(
          HttpClientRequest.patch(`/api/auth/users/${body.id}`).pipe(
            HttpClientRequest.bodyUnsafeJson({
              ...(body.email !== undefined && { email: body.email }),
              ...(body.is_active !== undefined && {
                is_active: body.is_active,
              }),
            }),
          ),
        )
        if (response.status === 403) {
          return yield* Effect.fail(
            new Error('You do not have permission to update users.'),
          )
        }
        const resBody = yield* response.json
        if (response.status < 200 || response.status >= 300) {
          return yield* Effect.fail(new Error(parseError(resBody)))
        }
        yield* Effect.logDebug('UsersLive.update: success')
      })

    const del: UsersService['delete'] = (id: string) =>
      Effect.gen(function* () {
        yield* Effect.logTrace('UsersLive.delete')
        yield* Effect.logDebug(`UsersLive.delete: id=${id}`)
        const response = yield* client.execute(
          HttpClientRequest.del(`/api/auth/users/${id}`),
        )
        if (response.status === 403) {
          return yield* Effect.fail(
            new Error('You do not have permission to delete users.'),
          )
        }
        const resBody = yield* response.json
        if (response.status < 200 || response.status >= 300) {
          return yield* Effect.fail(new Error(parseError(resBody)))
        }
        yield* Effect.logDebug('UsersLive.delete: success')
      })

    return {
      list,
      create,
      update,
      delete: del,
    }
  }),
)

export { UsersLive }
