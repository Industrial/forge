/**
 * Production implementation of Authentication.
 * Uses only the auth reactive store and localStorage for state.
 */
import { Effect, Option } from 'effect'
import { HttpClient, HttpClientRequest } from '@effect/platform'
import { Layer } from 'effect'

import { Authentication } from '@/features/authentication/services/Authentication'
import { AuthenticationUser } from '@/features/authentication/domain/AuthenticationUser'
import { AuthenticationError } from '@/features/authentication/errors/AuthenticationError'
import { ScopeError } from '@/features/authentication/errors'
import {
  AuthenticationStateReactiveStoreTag,
  initialAuthenticationState,
  type AuthenticationState,
} from '@/features/authentication/stores'
import { getBaseUrl } from '@/lib/baseUrl'

const TOKEN_KEY = 'token'
const ORG_ID_KEY = 'currentOrgId'
const ROLE_ID_KEY = 'currentRoleId'

function getStorage(): Storage | null {
  if (typeof window === 'undefined') return null
  return window.localStorage
}

function parseMeResponse(
  body: unknown,
  token: string,
): Option.Option<AuthenticationUser> {
  if (body == null || typeof body !== 'object' || !('user' in body)) {
    return Option.none()
  }
  const u = (body as { user?: { id?: unknown; email?: unknown } }).user
  if (u == null) return Option.none()
  return Option.some(
    new AuthenticationUser({
      id: String(u.id ?? ''),
      email: String(u.email ?? ''),
      token,
    }),
  )
}

function getNeedsScopeSelect(body: unknown): Option.Option<boolean> {
  if (
    body == null ||
    typeof body !== 'object' ||
    !('needs_scope_select' in body)
  ) {
    return Option.none()
  }
  const v = (body as { needs_scope_select?: boolean }).needs_scope_select
  return typeof v === 'boolean' ? Option.some(v) : Option.none()
}

export const AuthenticationLive = Layer.effect(
  Authentication,
  Effect.gen(function* () {
    const client = yield* HttpClient.HttpClient
    const store = yield* AuthenticationStateReactiveStoreTag
    const baseUrl = getBaseUrl()

    const fetchMeAndUpdate = (
      token: string,
    ): Effect.Effect<Option.Option<AuthenticationUser>, never, never> =>
      Effect.gen(function* () {
        const req = HttpClientRequest.get(`${baseUrl}/api/auth/me`).pipe(
          HttpClientRequest.setHeader('Authorization', `Bearer ${token}`),
        )
        const response = yield* client.execute(req)
        const ok = response.status >= 200 && response.status < 300
        if (!ok) {
          yield* store.update(() => initialAuthenticationState)
          const s = getStorage()
          if (s) {
            s.removeItem(TOKEN_KEY)
            s.removeItem(ORG_ID_KEY)
            s.removeItem(ROLE_ID_KEY)
          }
          return Option.none()
        }
        const body = yield* response.json
        const user = parseMeResponse(body, token)
        const needsScopeSelect = getNeedsScopeSelect(body)
        const newState: AuthenticationState = {
          token: Option.some(token),
          user,
          needsScopeSelect,
        }
        yield* store.update(() => newState)
        return user
      }).pipe(
        Effect.catchAll(() =>
          Effect.succeed(Option.none<AuthenticationUser>()),
        ),
      )

    return {
      getCurrentUser: (): Effect.Effect<
        Option.Option<AuthenticationUser>,
        AuthenticationError,
        never
      > =>
        Effect.gen(function* () {
          const s = getStorage()
          const token = s?.getItem(TOKEN_KEY) ?? null
          if (token == null || token === '') {
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
          const body = yield* response.json.pipe(
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
            const message =
              body != null &&
              typeof body === 'object' &&
              'message' in body &&
              typeof (body as { message: unknown }).message === 'string'
                ? (body as { message: string }).message
                : 'Login failed'
            return yield* Effect.fail(new AuthenticationError({ message }))
          }
          const token =
            body != null &&
            typeof body === 'object' &&
            'token' in body &&
            typeof (body as { token: unknown }).token === 'string'
              ? (body as { token: string }).token
              : null
          if (token == null) {
            return yield* Effect.fail(
              new AuthenticationError({
                message: 'Login response missing token',
              }),
            )
          }
          const s = getStorage()
          if (s) s.setItem(TOKEN_KEY, token)
          const userOpt = yield* fetchMeAndUpdate(token)
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

      logout: (): Effect.Effect<void, never, never> =>
        Effect.sync(() => {
          const s = getStorage()
          if (s) {
            s.removeItem(TOKEN_KEY)
            s.removeItem(ORG_ID_KEY)
            s.removeItem(ROLE_ID_KEY)
          }
        }).pipe(
          Effect.flatMap(() => store.update(() => initialAuthenticationState)),
        ),

      selectScope: (
        organizationId: string,
        roleId: string,
      ): Effect.Effect<void, ScopeError, never> =>
        Effect.gen(function* () {
          const s = getStorage()
          if (s) {
            s.setItem(ORG_ID_KEY, organizationId)
            s.setItem(ROLE_ID_KEY, roleId)
          }
          yield* store.update((state) => ({
            ...state,
            needsScopeSelect: Option.some(false),
          }))
        }),
    }
  }),
)
