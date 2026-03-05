import { HttpClient, HttpClientRequest } from '@effect/platform'
import { Effect } from 'effect'

type UserMembership = {
  org_id: string
  org_name: string
  roles: string[]
}

export type User = {
  id: string
  email: string
  is_active: boolean
  is_admin: boolean
  created_at: string
  memberships: UserMembership[]
}

/**
 * Effect that fetches the users list from the dashboard API using HttpClient.
 * Depends on HttpClient (from @effect/platform); auth and baseUrl come from the layer at runtime.
 * Fails with Error on non-OK or permission/unauthorized responses.
 */
export const fetchUsersEffect = Effect.gen(function* () {
  const client = yield* HttpClient.HttpClient
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
  const ok = response.status >= 200 && response.status < 300
  if (!ok) {
    const msg =
      typeof body === 'object' && body !== null && 'error' in body
        ? String((body as { error: unknown }).error)
        : `HTTP ${response.status}`
    return yield* Effect.fail(new Error(msg))
  }
  const data = body as { users?: User[] }
  return (data.users ?? []) as User[]
}).pipe(
  Effect.withSpan('fetchUsers', {
    attributes: { endpoint: '/api/dashboard/users' },
  }),
)

function parseErrorResponse(body: unknown): string {
  if (typeof body === 'object' && body !== null && 'error' in body) {
    return String((body as { error: unknown }).error)
  }
  return 'Request failed.'
}

/**
 * Effect that creates a user via POST /api/dashboard/users.
 * Requires HttpClient. Fails with Error on non-OK or permission denied.
 */
export const createUserEffect = (body: {
  email: string
  password: string
  org_id: string
  role_ids: string[]
}) =>
  Effect.gen(function* () {
    const client = yield* HttpClient.HttpClient
    const request = HttpClientRequest.post('/api/dashboard/users').pipe(
      HttpClientRequest.bodyUnsafeJson(body),
    )
    const response = yield* client.execute(request)
    if (response.status === 403) {
      return yield* Effect.fail(
        new Error('You do not have permission to create users.'),
      )
    }
    const resBody = yield* response.json
    if (response.status < 200 || response.status >= 300) {
      return yield* Effect.fail(new Error(parseErrorResponse(resBody)))
    }
  })

/**
 * Effect that updates a user via PATCH /api/dashboard/users.
 * Requires HttpClient. Fails with Error on non-OK or permission denied.
 */
export const updateUserEffect = (body: {
  id: string
  email?: string
  is_active?: boolean
}) =>
  Effect.gen(function* () {
    const client = yield* HttpClient.HttpClient
    const request = HttpClientRequest.patch('/api/dashboard/users').pipe(
      HttpClientRequest.bodyUnsafeJson(body),
    )
    const response = yield* client.execute(request)
    if (response.status === 403) {
      return yield* Effect.fail(
        new Error('You do not have permission to update users.'),
      )
    }
    const resBody = yield* response.json
    if (response.status < 200 || response.status >= 300) {
      return yield* Effect.fail(new Error(parseErrorResponse(resBody)))
    }
  })

/**
 * Effect that deletes a user via DELETE /api/dashboard/users.
 * Requires HttpClient. Fails with Error on non-OK or permission denied.
 */
export const deleteUserEffect = (id: string) =>
  Effect.gen(function* () {
    const client = yield* HttpClient.HttpClient
    const request = HttpClientRequest.del('/api/dashboard/users').pipe(
      HttpClientRequest.bodyUnsafeJson({ id }),
    )
    const response = yield* client.execute(request)
    if (response.status === 403) {
      return yield* Effect.fail(
        new Error('You do not have permission to delete users.'),
      )
    }
    const resBody = yield* response.json
    if (response.status < 200 || response.status >= 300) {
      return yield* Effect.fail(new Error(parseErrorResponse(resBody)))
    }
  })
