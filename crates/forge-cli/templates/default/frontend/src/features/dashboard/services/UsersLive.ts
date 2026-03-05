/**
 * Live implementation of Users service using HttpClient.
 */

import { HttpClient, HttpClientRequest } from '@effect/platform'
import { Effect, Layer } from 'effect'
import { Users } from './Users'
import type { UsersService } from './Users'
import { User, UserMembership } from '../domain/User'

function parseError(body: unknown): string {
  if (typeof body === 'object' && body !== null && 'error' in body) {
    return String((body as { error: unknown }).error)
  }
  return 'Request failed.'
}

const UsersLive = Layer.effect(
  Users,
  Effect.gen(function* () {
    const client = yield* HttpClient.HttpClient

    const list: UsersService['list'] = () =>
      Effect.gen(function* () {
        const response = yield* client.execute(
          HttpClientRequest.get('/api/dashboard/users'),
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
            memberships?: Array<{ org_id: string; org_name: string; roles: string[] }>
          }>
        }
        const raw = data.users ?? []
        return raw.map(
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
      })

    const create: UsersService['create'] = (body) =>
      Effect.gen(function* () {
        const request = HttpClientRequest.post('/api/dashboard/users').pipe(
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
      })

    const update: UsersService['update'] = (body) =>
      Effect.gen(function* () {
        const response = yield* client.execute(
          HttpClientRequest.patch('/api/dashboard/users').pipe(
            HttpClientRequest.bodyUnsafeJson(body),
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
      })

    const del: UsersService['delete'] = (id: string) =>
      Effect.gen(function* () {
        const response = yield* client.execute(
          HttpClientRequest.del('/api/dashboard/users').pipe(
            HttpClientRequest.bodyUnsafeJson({ id }),
          ),
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
