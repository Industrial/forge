/**
 * Production implementation of Authentication.
 * Uses auth reactive store and TokenStorage (Live = localStorage).
 * Uses shared API types (AuthMeBody, LoginResponse, RegisterResponse) and Schema at boundary.
 */
import { Effect, Option, pipe, Schema } from 'effect'
import { HttpClient, HttpClientRequest } from '@effect/platform'
import { Layer } from 'effect'

import {
  ApiErrorBodySchema,
  AuthMeBodySchema,
  LoginResponseSchema,
  type AuthMeBody,
} from '@/api/types'
import { Authentication } from '@/features/authentication/services/Authentication'
import { AuthenticationUser } from '@/features/authentication/domain/AuthenticationUser'
import { AuthenticationError } from '@/features/authentication/errors/AuthenticationError'
import { ScopeError } from '@/features/authentication/errors'
import {
  AuthStoreTag,
  initialAuthenticationState,
  type AuthenticationState,
} from '@/features/authentication/stores/AuthenticationStateReactiveStore'
import { TokenStorage } from '@/services/TokenStorage'
import { getBaseUrl } from '@/lib/baseUrl'

/** Pure: parse /api/auth/me body into Option<AuthenticationUser>. */
function parseMeResponse(
  body: AuthMeBody,
  token: string,
): Option.Option<AuthenticationUser> {
  const user = body.user
  if (user == null) return Option.none()
  return Option.some(
    new AuthenticationUser({
      id: String(user.id ?? ''),
      email: String(user.email ?? ''),
      token,
    }),
  )
}

/** Pure: read needs_scope_select from body. */
function getNeedsScopeSelect(body: AuthMeBody): Option.Option<boolean> {
  return body.needs_scope_select !== undefined
    ? Option.some(body.needs_scope_select)
    : Option.none()
}

/** Pure: read permissions from body (top-level or user). */
function getPermissions(body: AuthMeBody): readonly string[] {
  const list = body.permissions ?? body.user?.permissions
  return Array.isArray(list)
    ? list.filter((p): p is string => typeof p === 'string')
    : []
}

export const AuthenticationLive = Layer.effect(
  Authentication,
  Effect.gen(function* () {
    const client = yield* HttpClient.HttpClient
    const store = yield* AuthStoreTag
    const tokenStorage = yield* TokenStorage
    const baseUrl = getBaseUrl()

    const fetchMeAndUpdate = (
      token: string,
    ): Effect.Effect<Option.Option<AuthenticationUser>, never, never> =>
      Effect.gen(function* () {
        yield* Effect.logTrace('AuthenticationLive.fetchMeAndUpdate')

        const req = HttpClientRequest.get(`${baseUrl}/api/auth/me`).pipe(
          HttpClientRequest.setHeader('Authorization', `Bearer ${token}`),
        )
        const response = yield* client.execute(req)
        const ok = response.status >= 200 && response.status < 300
        yield* Effect.logDebug(
          `AuthenticationLive.fetchMeAndUpdate: status=${response.status}`,
        )

        if (!ok) {
          yield* store.update(() => initialAuthenticationState)
          yield* tokenStorage.clearToken()
          yield* tokenStorage.clearScope()
          return Option.none()
        }
        const rawBody = yield* response.json
        const body = yield* pipe(
          Schema.decodeUnknown(AuthMeBodySchema)(rawBody),
          Effect.catchAll(() => Effect.succeed({} as AuthMeBody)),
        )
        yield* Effect.logDebug(
          'AuthenticationLive.fetchMeAndUpdate: body keys=' +
            String(Object.keys(body)),
        )

        const user = parseMeResponse(body, token)
        const permissions = getPermissions(body)
        yield* Effect.logDebug(
          `AuthenticationLive.fetchMeAndUpdate: user present=${Option.isSome(user)}, permissions count=${permissions.length}`,
        )

        const needsScopeSelect = getNeedsScopeSelect(body)
        const newState: AuthenticationState = {
          token: Option.some(token),
          user,
          needsScopeSelect,
          permissions,
        }

        yield* store.update(() => newState)

        return user
      }).pipe(
        Effect.catchAll(() =>
          Effect.succeed(Option.none<AuthenticationUser>()),
        ),
      )

    return {
      restoreSession: (): Effect.Effect<void, never, never> =>
        Effect.gen(function* () {
          yield* Effect.logTrace('AuthenticationLive.restoreSession')

          const tokenOpt = yield* tokenStorage.getToken()
          yield* Effect.logDebug(
            `AuthenticationLive.restoreSession: hasToken=${Option.isSome(tokenOpt)}`,
          )

          yield* Option.match(tokenOpt, {
            onNone: () => Effect.void,
            onSome: (token) => fetchMeAndUpdate(token).pipe(Effect.asVoid),
          })
        }).pipe(Effect.catchAll(() => Effect.void)),

      getCurrentUser: (): Effect.Effect<
        Option.Option<AuthenticationUser>,
        AuthenticationError,
        never
      > =>
        Effect.gen(function* () {
          yield* Effect.logTrace('AuthenticationLive.getCurrentUser')

          const tokenOpt = yield* tokenStorage.getToken()
          yield* Effect.logDebug(
            `AuthenticationLive.getCurrentUser: hasToken=${Option.isSome(tokenOpt)}`,
          )

          const token = Option.getOrElse(tokenOpt, () => '')

          if (token === '') {
            yield* store.update(() => initialAuthenticationState)
            return Option.none()
          }

          return yield* fetchMeAndUpdate(token)
        }),

      login: (
        email: string,
        password: string,
      ): Effect.Effect<AuthenticationUser, AuthenticationError, never> =>
        Effect.gen(function* () {
          yield* Effect.logTrace('AuthenticationLive.login')

          const url = `${baseUrl}/api/auth/login`
          const req = HttpClientRequest.post(url).pipe(
            HttpClientRequest.bodyUnsafeJson({ email, password }),
          )
          const response = yield* client.execute(req).pipe(
            Effect.mapError(
              (e) =>
                new AuthenticationError({
                  message: e instanceof Error ? e.message : String(e),
                  cause: e,
                }),
            ),
          )
          const rawBody = yield* response.json.pipe(
            Effect.mapError(
              (e) =>
                new AuthenticationError({
                  message:
                    e instanceof Error ? e.message : 'Invalid response body',
                  cause: e,
                }),
            ),
          )
          const ok = response.status >= 200 && response.status < 300
          if (!ok) {
            const errBody = yield* pipe(
              Schema.decodeUnknown(ApiErrorBodySchema)(rawBody),
              Effect.catchAll(() =>
                Effect.succeed({ message: undefined, error: undefined }),
              ),
            )
            const message =
              errBody.message ??
              (typeof errBody.error === 'string'
                ? errBody.error
                : errBody.error?.message) ??
              'Login failed'
            return yield* Effect.fail(new AuthenticationError({ message }))
          }

          const decoded = yield* pipe(
            Schema.decodeUnknown(LoginResponseSchema)(rawBody),
            Effect.mapError(
              (e) =>
                new AuthenticationError({
                  message: e.message ?? 'Invalid login response',
                  cause: e,
                }),
            ),
          )
          const token = decoded.token
          yield* Effect.logDebug(
            `AuthenticationLive.login: hasToken=${token != null}`,
          )

          if (!token) {
            return yield* Effect.fail(
              new AuthenticationError({
                message: 'Login response missing token',
              }),
            )
          }

          yield* tokenStorage.setToken(token)

          const userOpt = yield* fetchMeAndUpdate(token)
          yield* Effect.logDebug(
            `AuthenticationLive.login: userLoaded=${Option.isSome(userOpt)}`,
          )

          return userOpt.pipe(
            Option.match({
              onNone: () =>
                Effect.fail(
                  new AuthenticationError({
                    message: 'Failed to load user after login',
                  }),
                ),
              onSome: (u) => Effect.succeed(u),
            }),
          )
        }).pipe(Effect.flatten),

      register: (
        email: string,
        password: string,
      ): Effect.Effect<void, AuthenticationError, never> =>
        Effect.gen(function* () {
          yield* Effect.logTrace('AuthenticationLive.register')

          const url = `${baseUrl}/api/auth/register`
          const req = HttpClientRequest.post(url).pipe(
            HttpClientRequest.bodyUnsafeJson({ email, password }),
          )
          const response = yield* client.execute(req).pipe(
            Effect.mapError(
              (e) =>
                new AuthenticationError({
                  message: e instanceof Error ? e.message : String(e),
                  cause: e,
                }),
            ),
          )
          const rawBody = yield* response.json.pipe(
            Effect.mapError(
              (e) =>
                new AuthenticationError({
                  message:
                    e instanceof Error ? e.message : 'Invalid response body',
                  cause: e,
                }),
            ),
          )
          yield* Effect.logDebug(
            `AuthenticationLive.register: body=${JSON.stringify(rawBody)}`,
          )

          const ok = response.status >= 200 && response.status < 300
          if (!ok) {
            const errBody = yield* pipe(
              Schema.decodeUnknown(ApiErrorBodySchema)(rawBody),
              Effect.catchAll(() =>
                Effect.succeed({ message: undefined, error: undefined }),
              ),
            )
            const message =
              errBody.message ??
              (typeof errBody.error === 'string'
                ? errBody.error
                : errBody.error?.message) ??
              'Registration failed'
            return yield* Effect.fail(new AuthenticationError({ message }))
          }
        }),

      logout: (): Effect.Effect<void, never, never> =>
        Effect.gen(function* () {
          yield* Effect.logTrace('AuthenticationLive.logout')

          yield* tokenStorage.clearToken()
          yield* tokenStorage.clearScope()
          yield* store.update(() => initialAuthenticationState)
        }),

      selectScope: (
        organizationId: string,
        roleId: string,
      ): Effect.Effect<void, ScopeError, never> =>
        Effect.gen(function* () {
          yield* Effect.logTrace('AuthenticationLive.selectScope')

          yield* tokenStorage.setScope(organizationId, roleId)
          
          // Refetch user and permissions after scope selection since permissions are scope-dependent
          const tokenOpt = yield* tokenStorage.getToken()
          yield* pipe(
            Option.match(tokenOpt, {
              onNone: () => Effect.void,
              onSome: (token) =>
                Effect.gen(function* () {
                  // Include scope headers when fetching /api/auth/me to get scope-dependent permissions
                  const req = HttpClientRequest.get(`${baseUrl}/api/auth/me`).pipe(
                    HttpClientRequest.setHeader('Authorization', `Bearer ${token}`),
                    HttpClientRequest.setHeader('x-organization-id', organizationId),
                    HttpClientRequest.setHeader('x-role-id', roleId),
                  )
                  const response = yield* client.execute(req)
                  const ok = response.status >= 200 && response.status < 300
                  yield* Effect.logDebug(
                    `AuthenticationLive.selectScope: fetchMe status=${response.status}`,
                  )

                  if (!ok) {
                    return
                  }
                  const rawBody = yield* response.json
                  const body = yield* pipe(
                    Schema.decodeUnknown(AuthMeBodySchema)(rawBody),
                    Effect.catchAll(() => Effect.succeed({} as AuthMeBody)),
                  )
                  yield* Effect.logDebug(
                    `AuthenticationLive.selectScope: permissions count=${getPermissions(body).length}`,
                  )

                  const user = parseMeResponse(body, token)
                  const permissions = getPermissions(body)
                  const needsScopeSelect = getNeedsScopeSelect(body)
                  const newState: AuthenticationState = {
                    token: Option.some(token),
                    user,
                    needsScopeSelect,
                    permissions,
                  }

                  yield* store.update(() => newState)
                }),
            }),
            Effect.catchAll((error) =>
              Effect.gen(function* () {
                // If refetch fails, log but don't fail scope selection (scope is already set)
                yield* Effect.logDebug(
                  `AuthenticationLive.selectScope: failed to refetch permissions: ${String(error)}`,
                )
                return Effect.void
              }),
            ),
          )
        }),
    }
  }),
)
