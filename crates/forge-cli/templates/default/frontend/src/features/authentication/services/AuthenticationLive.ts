/**
 * Production implementation of Authentication.
 * Uses auth reactive store and TokenStorage (Live = localStorage).
 * Uses shared API types (AuthMeBody, LoginResponse, RegisterResponse) and Schema at boundary.
 *
 * Architecture:
 * - Pure functions: No side effects, easy to unit test
 * - HTTP request builders: Pure functions that return requests
 * - Response parsers: Effect functions (R = never)
 * - HTTP executors: Effect functions with HttpClient requirement
 * - Composition functions: Orchestrate multiple operations
 * - Layer implementation: Service methods access dependencies via yield*
 */
import { Effect, Option, pipe, Schema } from 'effect'
import {
  HttpClient,
  HttpClientRequest,
  HttpClientError,
} from '@effect/platform'
import { Layer } from 'effect'

import {
  ApiErrorBodySchema,
  AuthMeBodySchema,
  LoginResponseSchema,
  type AuthMeBody,
} from '@/api/types'
import { Authentication } from '@/features/authentication/services/Authentication'
import { AuthenticationUser } from '@/features/authentication/domain/AuthenticationUser'
import { Scope } from '@/features/authentication/domain/Scope'
import {
  ScopeError,
  LoginFailedError,
  RegistrationFailedError,
  UserNotFoundError,
  InvalidTokenError,
  TokenMissingError,
} from '@/features/authentication/errors'
import {
  AuthStoreTag,
  initialAuthenticationState,
  type AuthenticationState,
} from '@/features/authentication/stores/AuthenticationStateReactiveStore'
import { TokenStorage } from '@/services/TokenStorage'
import { getBaseUrl } from '@/lib/baseUrl'
import type { ReactiveStore } from '@/lib/ReactiveStore'

// ============================================================================
// SECTION 1: Pure Helper Functions
// ============================================================================

/** Pure: parse /api/auth/me body into Option<AuthenticationUser>. */
export function parseMeResponse(
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

/** Pure: read permissions from body (top-level or user). */
export function getPermissions(body: AuthMeBody): readonly string[] {
  const list = body.permissions ?? body.user?.permissions
  return Array.isArray(list)
    ? list.filter((p): p is string => typeof p === 'string')
    : []
}

/** Pure: extract error message from API error response body. */
export function extractApiErrorMessage(
  errBody: { message?: string; error?: string | { message?: string } },
  fallback: string,
): string {
  return (
    errBody.message ??
    (typeof errBody.error === 'string'
      ? errBody.error
      : errBody.error?.message) ??
    fallback
  )
}

/** Pure: build authentication state from /api/auth/me response. */
export function buildAuthState(
  token: string,
  body: AuthMeBody,
  currentScope?: { organizationId: string; roleId: string },
): AuthenticationState {
  return {
    token: Option.some(token),
    user: parseMeResponse(body, token),
    permissions: getPermissions(body),
    currentScope: currentScope
      ? Option.some({
          organizationId: currentScope.organizationId,
          roleId: currentScope.roleId,
        })
      : Option.none(),
  }
}

/** Pure: select single scope from scopes array (for auto-selection). */
export function selectSingleScope(
  scopes: readonly { org_id: string; role_id?: string }[],
): Option.Option<{ organizationId: string; roleId: string }> {
  if (!Array.isArray(scopes) || scopes.length !== 1) return Option.none()

  const scope = scopes[0]
  const orgId = scope.org_id
  const roleId = scope.role_id ?? ''

  return orgId && roleId
    ? Option.some({ organizationId: orgId, roleId })
    : Option.none()
}

// ============================================================================
// SECTION 2: HTTP Request Builders (Pure)
// ============================================================================

/** Pure: build authenticated request to /api/auth/* endpoint. */
export function buildAuthRequest(
  baseUrl: string,
  token: string,
  endpoint: string,
): HttpClientRequest.HttpClientRequest {
  return HttpClientRequest.get(`${baseUrl}${endpoint}`).pipe(
    HttpClientRequest.setHeader('Authorization', `Bearer ${token}`),
  )
}

/** Pure: build authenticated request with scope headers. */
export function buildAuthRequestWithScope(
  baseUrl: string,
  token: string,
  scope: { organizationId: string; roleId: string },
  endpoint: string,
): HttpClientRequest.HttpClientRequest {
  return buildAuthRequest(baseUrl, token, endpoint).pipe(
    HttpClientRequest.setHeader('x-organization-id', scope.organizationId),
    HttpClientRequest.setHeader('x-role-id', scope.roleId),
  )
}

/** Pure: build login request. */
export function buildLoginRequest(
  baseUrl: string,
  email: string,
  password: string,
): HttpClientRequest.HttpClientRequest {
  return HttpClientRequest.post(`${baseUrl}/api/auth/login`).pipe(
    HttpClientRequest.bodyUnsafeJson({ email, password }),
  )
}

/** Pure: build register request. */
export function buildRegisterRequest(
  baseUrl: string,
  email: string,
  password: string,
): HttpClientRequest.HttpClientRequest {
  return HttpClientRequest.post(`${baseUrl}/api/auth/register`).pipe(
    HttpClientRequest.bodyUnsafeJson({ email, password }),
  )
}

// ============================================================================
// SECTION 3: Schema Definitions
// ============================================================================

/** Schema for /api/auth/scopes response. */
export const ScopesResponseSchema = Schema.Struct({
  scopes: Schema.optional(
    Schema.Array(
      Schema.Struct({
        org_id: Schema.String,
        role_id: Schema.optional(Schema.String),
      }),
    ),
  ),
})

export type ScopesResponse = Schema.Schema.Type<typeof ScopesResponseSchema>

/** Schema for /api/auth/scopes response with display fields (org_name, role). */
export const ScopesResponseFullSchema = Schema.Struct({
  scopes: Schema.optional(
    Schema.Array(
      Schema.Struct({
        org_id: Schema.String,
        org_name: Schema.String,
        role_id: Schema.optional(Schema.String),
        role: Schema.String,
      }),
    ),
  ),
})

export type ScopesResponseFull = Schema.Schema.Type<
  typeof ScopesResponseFullSchema
>

// ============================================================================
// SECTION 4: Response Parsers (Effect, R = never)
// ============================================================================

/** Parse JSON from HTTP response with error mapping. */
export function parseJsonResponse<E>(
  response: { json: Effect.Effect<unknown, unknown, never> },
  toError: (e: unknown) => E,
): Effect.Effect<unknown, E, never> {
  return pipe(response.json, Effect.mapError(toError))
}

/** Parse /api/auth/me response body (never fails, returns empty object on error). */
export function parseAuthMeBody(
  rawBody: unknown,
): Effect.Effect<AuthMeBody, never, never> {
  return pipe(
    Schema.decodeUnknown(AuthMeBodySchema)(rawBody),
    Effect.catchAll(() => Effect.succeed({} as AuthMeBody)),
  )
}

/** Parse /api/auth/login response body. */
export function parseLoginResponse(
  rawBody: unknown,
): Effect.Effect<{ token?: string }, LoginFailedError, never> {
  return pipe(
    Schema.decodeUnknown(LoginResponseSchema)(rawBody),
    Effect.mapError(
      (e) =>
        new LoginFailedError({
          message: e.message ?? 'Invalid login response',
          cause: e,
        }),
    ),
  )
}

/** Parse error response body and extract error message. */
export function parseErrorResponse(
  rawBody: unknown,
  fallback: string,
): Effect.Effect<string, never, never> {
  return pipe(
    Schema.decodeUnknown(ApiErrorBodySchema)(rawBody),
    Effect.map((errBody) => extractApiErrorMessage(errBody, fallback)),
    Effect.catchAll(() => Effect.succeed(fallback)),
  )
}

/** Parse /api/auth/scopes response body. */
export function parseScopesResponse(
  rawBody: unknown,
): Effect.Effect<ScopesResponse, never, never> {
  return pipe(
    rawBody,
    Schema.decodeUnknown(ScopesResponseSchema),
    Effect.catchAll(() => Effect.succeed({ scopes: [] })),
  )
}

/** Parse /api/auth/scopes response body with full scope fields. */
export function parseScopesResponseFull(
  rawBody: unknown,
): Effect.Effect<ScopesResponseFull, never, never> {
  return pipe(
    rawBody,
    Schema.decodeUnknown(ScopesResponseFullSchema),
    Effect.catchAll(() => Effect.succeed({ scopes: [] })),
  )
}

// ============================================================================
// SECTION 5: HTTP Executors (Effect with HttpClient requirement)
// ============================================================================

/**
 * Fetch /api/auth/me and parse response.
 * Returns null on error or non-200 status.
 */
export function fetchAuthMe(
  baseUrl: string,
  token: string,
  scope?: { organizationId: string; roleId: string },
): Effect.Effect<AuthMeBody | null, never, HttpClient.HttpClient> {
  return Effect.gen(function* () {
    const client = yield* HttpClient.HttpClient

    const request = scope
      ? buildAuthRequestWithScope(baseUrl, token, scope, '/api/auth/me')
      : buildAuthRequest(baseUrl, token, '/api/auth/me')

    const response = yield* client
      .execute(request)
      .pipe(Effect.catchAll(() => Effect.succeed(null)))

    if (!response || response.status < 200 || response.status >= 300) {
      return null
    }

    const rawBody = yield* response.json.pipe(
      Effect.catchAll(() => Effect.succeed(null)),
    )

    if (!rawBody) return null

    return yield* parseAuthMeBody(rawBody)
  })
}

/**
 * Fetch /api/auth/scopes and parse response.
 * Returns empty array on error.
 */
export function fetchScopes(
  baseUrl: string,
  token: string,
): Effect.Effect<
  readonly { org_id: string; role_id?: string }[],
  never,
  HttpClient.HttpClient
> {
  return Effect.gen(function* () {
    const client = yield* HttpClient.HttpClient
    const request = buildAuthRequest(baseUrl, token, '/api/auth/scopes')

    const response = yield* client
      .execute(request)
      .pipe(Effect.catchAll(() => Effect.succeed(null)))

    if (!response || response.status < 200 || response.status >= 300) {
      return []
    }

    const rawBody = yield* response.json.pipe(
      Effect.catchAll(() => Effect.succeed(null)),
    )

    if (!rawBody) return []

    const scopesBody = yield* parseScopesResponse(rawBody)
    return scopesBody.scopes ?? []
  })
}

/**
 * Fetch /api/auth/scopes and parse as full Scope[] (org_name, role for display).
 */
export function fetchScopesFull(
  baseUrl: string,
  token: string,
): Effect.Effect<readonly Scope[], never, HttpClient.HttpClient> {
  return Effect.gen(function* () {
    const client = yield* HttpClient.HttpClient
    const request = buildAuthRequest(baseUrl, token, '/api/auth/scopes')

    const response = yield* client
      .execute(request)
      .pipe(Effect.catchAll(() => Effect.succeed(null)))

    if (!response || response.status < 200 || response.status >= 300) {
      return []
    }

    const rawBody = yield* response.json.pipe(
      Effect.catchAll(() => Effect.succeed(null)),
    )

    if (!rawBody) return []

    const scopesBody = yield* parseScopesResponseFull(rawBody)
    const list = scopesBody.scopes ?? []
    return list.map(
      (s) =>
        new Scope({
          org_id: s.org_id,
          org_name: s.org_name ?? s.org_id,
          role_id: s.role_id,
          role: s.role ?? '',
        }),
    )
  })
}

/**
 * Perform login request and return token.
 * Throws LoginFailedError or TokenMissingError on error.
 */
export function performLogin(
  baseUrl: string,
  email: string,
  password: string,
): Effect.Effect<
  { token: string },
  LoginFailedError | TokenMissingError | HttpClientError.HttpClientError,
  HttpClient.HttpClient
> {
  return Effect.gen(function* () {
    const client = yield* HttpClient.HttpClient
    const request = buildLoginRequest(baseUrl, email, password)

    const response = yield* pipe(
      client.execute(request),
      Effect.mapError(
        (e) =>
          new LoginFailedError({
            message: e instanceof Error ? e.message : String(e),
            cause: e,
          }),
      ),
    )

    const rawBody = yield* parseJsonResponse(
      response,
      (e) =>
        new LoginFailedError({
          message: e instanceof Error ? e.message : 'Invalid response body',
          cause: e,
        }),
    )

    if (response.status < 200 || response.status >= 300) {
      const message = yield* parseErrorResponse(rawBody, 'Login failed')
      return yield* Effect.fail(new LoginFailedError({ message }))
    }

    const decoded = yield* parseLoginResponse(rawBody)

    if (!decoded.token) {
      return yield* Effect.fail(
        new TokenMissingError({
          message: 'Login response missing token',
        }),
      )
    }

    return { token: decoded.token }
  })
}

/**
 * Perform register request.
 * Throws RegistrationFailedError on error.
 */
export function performRegister(
  baseUrl: string,
  email: string,
  password: string,
): Effect.Effect<
  void,
  RegistrationFailedError | HttpClientError.HttpClientError,
  HttpClient.HttpClient
> {
  return Effect.gen(function* () {
    const client = yield* HttpClient.HttpClient
    const request = buildRegisterRequest(baseUrl, email, password)

    const response = yield* pipe(
      client.execute(request),
      Effect.mapError(
        (e) =>
          new RegistrationFailedError({
            message: e instanceof Error ? e.message : String(e),
            cause: e,
          }),
      ),
    )

    const rawBody = yield* parseJsonResponse(
      response,
      (e) =>
        new RegistrationFailedError({
          message: e instanceof Error ? e.message : 'Invalid response body',
          cause: e,
        }),
    )

    if (response.status < 200 || response.status >= 300) {
      const message = yield* parseErrorResponse(rawBody, 'Registration failed')
      return yield* Effect.fail(new RegistrationFailedError({ message }))
    }
  })
}

// ============================================================================
// SECTION 6: Composition Functions
// ============================================================================

/**
 * Fetch /api/auth/me and build authentication state.
 * Returns null on error.
 */
export function fetchMeAndBuildState(
  baseUrl: string,
  token: string,
  scope?: { organizationId: string; roleId: string },
): Effect.Effect<AuthenticationState | null, never, HttpClient.HttpClient> {
  return Effect.gen(function* () {
    const body = yield* fetchAuthMe(baseUrl, token, scope)
    if (!body) return null
    return buildAuthState(token, body, scope)
  })
}

/**
 * Auto-select scope if exactly one scope is available.
 * Returns the selected scope or None if multiple/zero scopes.
 */
export function autoSelectScope(
  baseUrl: string,
  token: string,
  tokenStorage: TokenStorage,
): Effect.Effect<
  Option.Option<{ organizationId: string; roleId: string }>,
  never,
  HttpClient.HttpClient
> {
  return Effect.gen(function* () {
    const scopes = yield* fetchScopes(baseUrl, token)
    const selectedScope = selectSingleScope(scopes)

    yield* Option.match(selectedScope, {
      onNone: () => Effect.void,
      onSome: (scope) =>
        tokenStorage.setScope(scope.organizationId, scope.roleId),
    })

    return selectedScope
  })
}

/**
 * Update authentication state in store.
 */
export function updateAuthState(
  store: ReactiveStore<AuthenticationState>,
  state: AuthenticationState,
): Effect.Effect<void, never, never> {
  return Effect.gen(function* () {
    yield* Effect.logTrace('AuthenticationLive.updateAuthState')
    const hasUser = Option.isSome(state.user)
    yield* Effect.logDebug(
      `AuthenticationLive.updateAuthState: hasUser=${hasUser}, permissionsCount=${state.permissions.length}`,
    )
    yield* store.update(() => state)
  })
}

/**
 * Clear authentication state (token, scope, store).
 */
export function clearAuthState(
  store: ReactiveStore<AuthenticationState>,
  tokenStorage: TokenStorage,
): Effect.Effect<void, never, never> {
  return Effect.gen(function* () {
    yield* store.update(() => initialAuthenticationState)
    yield* tokenStorage.clearToken()
    yield* tokenStorage.clearScope()
  })
}

// ============================================================================
// SECTION 7: Layer Implementation
// ============================================================================

export const AuthenticationLive = Layer.effect(
  Authentication,
  Effect.gen(function* () {
    const client = yield* HttpClient.HttpClient
    const store = yield* AuthStoreTag
    const tokenStorage = yield* TokenStorage
    const baseUrl = getBaseUrl()

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
            onSome: (token) =>
              Effect.gen(function* () {
                const scopeOpt = yield* tokenStorage.getScope()
                // When we have a stored scope, fetch /me only with scope so the initial
                // state has permissions. Otherwise DashboardScopeGuard sees permissions.length === 0
                // and shows spinner forever after full page load (e2e and reloads).
                let state: AuthenticationState | null = null
                if (Option.isSome(scopeOpt)) {
                  state = yield* fetchMeAndBuildState(
                    baseUrl,
                    token,
                    scopeOpt.value,
                  ).pipe(
                    Effect.provide(
                      Layer.succeed(HttpClient.HttpClient, client),
                    ),
                  )
                  if (state) yield* updateAuthState(store, state)
                } else {
                  state = yield* fetchMeAndBuildState(baseUrl, token).pipe(
                    Effect.provide(Layer.succeed(HttpClient.HttpClient, client)),
                  )
                  if (state) yield* updateAuthState(store, state)
                }
                // Token was present but /me failed (e.g. 401): clear localStorage so we don't retry on next load.
                if (!state) {
                  yield* Effect.logDebug(
                    'AuthenticationLive.restoreSession: /me failed (e.g. 401), clearing stored token and scope',
                  )
                  yield* clearAuthState(store, tokenStorage)
                }
              }),
          })
        }).pipe(
          Effect.catchAll((error: unknown) =>
            Effect.gen(function* () {
              yield* Effect.logError(
                `Session restoration failed: ${error instanceof Error ? error.message : String(error)}`,
              )
              yield* clearAuthState(store, tokenStorage)
            }),
          ),
        ),

      getCurrentUser: (): Effect.Effect<
        Option.Option<AuthenticationUser>,
        InvalidTokenError | UserNotFoundError,
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
            yield* clearAuthState(store, tokenStorage)
            return Option.none()
          }

          const state = yield* fetchMeAndBuildState(baseUrl, token).pipe(
            Effect.provide(Layer.succeed(HttpClient.HttpClient, client)),
          )

          return state?.user ?? Option.none()
        }).pipe(
          Effect.catchAll((e: unknown) => {
            if (
              e instanceof InvalidTokenError ||
              e instanceof UserNotFoundError
            ) {
              return Effect.fail(e)
            }
            return Effect.fail(
              new InvalidTokenError({
                message: e instanceof Error ? e.message : String(e),
              }),
            )
          }),
        ),

      login: (
        email: string,
        password: string,
      ): Effect.Effect<
        AuthenticationUser,
        LoginFailedError | TokenMissingError | UserNotFoundError,
        never
      > =>
        pipe(
          Effect.gen(function* () {
            yield* Effect.logTrace('AuthenticationLive.login')

            // 1. Perform login and get token
            const { token } = yield* performLogin(
              baseUrl,
              email,
              password,
            ).pipe(
              Effect.provide(Layer.succeed(HttpClient.HttpClient, client)),
              Effect.catchAll((e) => {
                if (
                  e instanceof LoginFailedError ||
                  e instanceof TokenMissingError
                ) {
                  return Effect.fail(e)
                }
                return Effect.fail(
                  new LoginFailedError({
                    message: e instanceof Error ? e.message : String(e),
                    cause: e,
                  }),
                )
              }),
            )
            yield* tokenStorage.setToken(token)
            yield* Effect.logDebug(
              `AuthenticationLive.login: hasToken=${token != null}`,
            )

            // 2. Fetch initial user state
            const state = yield* fetchMeAndBuildState(baseUrl, token).pipe(
              Effect.provide(Layer.succeed(HttpClient.HttpClient, client)),
            )
            if (!state) {
              return yield* Effect.fail(
                new UserNotFoundError({
                  message: 'Failed to load user after login',
                }),
              )
            }
            yield* updateAuthState(store, state)
            yield* Effect.logDebug(
              `AuthenticationLive.login: userLoaded=${Option.isSome(state.user)}`,
            )

            // 3. Auto-select scope when user has exactly one scope
            const selectedScope = yield* autoSelectScope(
              baseUrl,
              token,
              tokenStorage,
            ).pipe(
              Effect.provide(Layer.succeed(HttpClient.HttpClient, client)),
            )

            // 4. Refetch with scope if auto-selected (single scope)
            if (Option.isSome(selectedScope)) {
              yield* Effect.logDebug(
                `AuthenticationLive.login: auto-selected scope orgId=${selectedScope.value.organizationId}, roleId=${selectedScope.value.roleId}`,
              )
              const updatedState = yield* fetchMeAndBuildState(
                baseUrl,
                token,
                selectedScope.value,
              ).pipe(
                Effect.provide(Layer.succeed(HttpClient.HttpClient, client)),
              )
              if (updatedState) yield* updateAuthState(store, updatedState)
            }

            return state.user.pipe(
              Option.getOrElse(() => {
                throw new UserNotFoundError({
                  message: 'User not found after login',
                })
              }),
            )
          }),
        ),

      register: (
        email: string,
        password: string,
      ): Effect.Effect<void, RegistrationFailedError, never> =>
        Effect.gen(function* () {
          yield* Effect.logTrace('AuthenticationLive.register')

          yield* performRegister(baseUrl, email, password).pipe(
            Effect.provide(Layer.succeed(HttpClient.HttpClient, client)),
          )
        }).pipe(
          Effect.catchAll((e: unknown) => {
            if (e instanceof RegistrationFailedError) {
              return Effect.fail(e)
            }
            return Effect.fail(
              new RegistrationFailedError({
                message: e instanceof Error ? e.message : String(e),
                cause: e,
              }),
            )
          }),
        ),

      logout: (): Effect.Effect<void, never, never> =>
        Effect.gen(function* () {
          yield* Effect.logTrace('AuthenticationLive.logout')
          yield* clearAuthState(store, tokenStorage)
        }),

      selectScope: (
        organizationId: string,
        roleId: string,
      ): Effect.Effect<void, ScopeError, never> =>
        Effect.gen(function* () {
          yield* Effect.logTrace('AuthenticationLive.selectScope')

          yield* tokenStorage.setScope(organizationId, roleId)

          // Optimistic update: set currentScope so the UI does not redirect
          // back to select-scope if the refetch fails.
          yield* Effect.logDebug(
            'AuthenticationLive.selectScope: optimistic store update (currentScope)',
          )
          yield* store.update((state) => ({
            ...state,
            currentScope: Option.some({
              organizationId,
              roleId,
            }),
          }))

          // Refetch user and permissions after scope selection
          const tokenOpt = yield* tokenStorage.getToken()
          if (Option.isSome(tokenOpt)) {
            const token = tokenOpt.value
            yield* pipe(
              Effect.gen(function* () {
                const state = yield* fetchMeAndBuildState(baseUrl, token, {
                  organizationId,
                  roleId,
                }).pipe(
                  Effect.provide(Layer.succeed(HttpClient.HttpClient, client)),
                )
                yield* Effect.logDebug(
                  `AuthenticationLive.selectScope: fetchMe status=${state ? 'success' : 'failed'}`,
                )

                if (!state) return

                yield* Effect.logDebug(
                  `AuthenticationLive.selectScope: permissions count=${state.permissions.length}`,
                )
                const stateForStore: AuthenticationState = {
                  ...state,
                  currentScope: Option.some({ organizationId, roleId }),
                }
                yield* updateAuthState(store, stateForStore)
              }),
              Effect.catchAll((error) =>
                Effect.gen(function* () {
                  yield* Effect.logDebug(
                    `AuthenticationLive.selectScope: failed to refetch permissions: ${String(error)}`,
                  )
                }),
              ),
              Effect.asVoid,
            )
          }
        }),

      getScopes: (): Effect.Effect<readonly Scope[], never, never> =>
        Effect.gen(function* () {
          const tokenOpt = yield* tokenStorage.getToken()
          const token = Option.getOrElse(tokenOpt, () => '')
          if (token === '') return []
          return yield* fetchScopesFull(baseUrl, token).pipe(
            Effect.provide(Layer.succeed(HttpClient.HttpClient, client)),
            Effect.catchAll(() => Effect.succeed([])),
          )
        }),
    }
  }),
)
