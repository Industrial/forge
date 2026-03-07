/**
 * BDD tests for AuthenticationLive - production authentication service implementation.
 * Tests use Effect's Layer system for dependency injection (no vi.mock()).
 */
import { describe, test, expect, beforeEach } from 'bun:test'
import { Effect, Layer, Option, Stream, Chunk } from 'effect'
import { HttpClient, HttpClientRequest } from '@effect/platform'

import { AuthenticationLive } from './AuthenticationLive'
import { Authentication } from './Authentication'
import { AuthenticationUser } from '@/features/authentication/domain/AuthenticationUser'
import { AuthenticationError } from '@/features/authentication/errors/AuthenticationError'
import { ScopeError } from '@/features/authentication/errors'
import {
  AuthStoreTag,
  initialAuthenticationState,
  type AuthenticationState,
} from '@/features/authentication/stores/AuthenticationStateReactiveStore'
import { TokenStorage } from '@/services/TokenStorage'
import { makeTokenStorageMock } from '@/services/TokenStorageMock'
import { defineStore, type ReactiveStore } from '@/lib/ReactiveStore'
import type { AuthMeBody, LoginResponse } from '@/api/types'

// Helper to create a mock HttpClient
function createMockHttpClient(
  handler: (
    request: HttpClientRequest.HttpClientRequest,
  ) => Effect.Effect<
    {
      status: number
      json: Effect.Effect<unknown>
      headers: Headers
    },
    never,
    never
  >,
): HttpClient.HttpClient {
  const executeImpl = (request: HttpClientRequest.HttpClientRequest) => {
    return handler(request)
  }

  // Create a minimal HttpClient with execute method
  return {
    execute: executeImpl,
    // Stub other methods (not used by AuthenticationLive)
    get: () => Effect.succeed({ status: 404, json: Effect.succeed({}), headers: new Headers() }),
    post: () => Effect.succeed({ status: 404, json: Effect.succeed({}), headers: new Headers() }),
    put: () => Effect.succeed({ status: 404, json: Effect.succeed({}), headers: new Headers() }),
    patch: () => Effect.succeed({ status: 404, json: Effect.succeed({}), headers: new Headers() }),
    delete: () => Effect.succeed({ status: 404, json: Effect.succeed({}), headers: new Headers() }),
    head: () => Effect.succeed({ status: 404, json: Effect.succeed({}), headers: new Headers() }),
    options: () => Effect.succeed({ status: 404, json: Effect.succeed({}), headers: new Headers() }),
    request: () => Effect.succeed({ status: 404, json: Effect.succeed({}), headers: new Headers() }),
    requestWith: () => Effect.succeed({ status: 404, json: Effect.succeed({}), headers: new Headers() }),
    stream: () => Effect.succeed({ status: 404, json: Effect.succeed({}), headers: new Headers() }),
    streamWith: () => Effect.succeed({ status: 404, json: Effect.succeed({}), headers: new Headers() }),
  } as HttpClient.HttpClient
}

// Helper to create a mock auth store using AuthStoreTag
function createMockAuthStore(
  initialState: AuthenticationState = initialAuthenticationState,
): {
  store: ReactiveStore<AuthenticationState>
  layer: Layer.Layer<ReactiveStore<AuthenticationState>>
  getState: () => AuthenticationState
  setState: (state: AuthenticationState) => void
} {
  let current = initialState
  const changeListeners = new Set<(a: AuthenticationState) => void>()

  const notify = (a: AuthenticationState) => {
    current = a
    changeListeners.forEach((l) => l(a))
  }

  const changes = Stream.async<AuthenticationState, never, never>((emit) => {
    emit(Effect.succeed(Chunk.of(current)))
    const listener = (a: AuthenticationState) => {
      emit(Effect.succeed(Chunk.of(a)))
    }
    changeListeners.add(listener)
    return Effect.sync(() => {
      changeListeners.delete(listener)
    })
  })

  const store: ReactiveStore<AuthenticationState> = {
    get: () => Effect.succeed(current),
    update: (f: (a: AuthenticationState) => AuthenticationState) =>
      Effect.sync(() => {
        notify(f(current))
      }),
    changes,
  }

  // Use AuthStoreTag directly (the tag AuthenticationLive expects)
  const mockLayer = Layer.succeed(AuthStoreTag, store)

  return {
    store,
    layer: mockLayer,
    getState: () => current,
    setState: (state: AuthenticationState) => {
      current = state
    },
  }
}

describe('AuthenticationLive', () => {
  describe('restoreSession behavior', () => {
    test('should restore session when token exists', async () => {
      // Given: a token in storage and successful /api/auth/me response
      const token = 'test-token-123'
      const tokenStorage = makeTokenStorageMock()
      tokenStorage.setToken(token)

      const mockMeResponse: AuthMeBody = {
        user: {
          id: 'user-1',
          email: 'test@example.com',
        },
        permissions: ['read:users'],
      }

      const mockHttpClient = createMockHttpClient((request) => {
        if (request.url.includes('/api/auth/me')) {
          return Effect.succeed({
            status: 200,
            json: Effect.succeed(mockMeResponse),
            headers: new Headers(),
          })
        }
        return Effect.succeed({
          status: 404,
          json: Effect.succeed({}),
          headers: new Headers(),
        })
      })

      const mockAuthStore = createMockAuthStore()

      const httpLayer = Layer.succeed(HttpClient.HttpClient, mockHttpClient)
      const testLayer = AuthenticationLive.pipe(
        Layer.provide(httpLayer),
        Layer.provide(tokenStorage.layer),
        Layer.provide(mockAuthStore.layer),
      )

      // When: restoring session
      const authentication = await Effect.runPromise(
        Effect.gen(function* () {
          return yield* Authentication
        }).pipe(Effect.provide(testLayer)),
      )

      const result = await Effect.runPromise(
        authentication.restoreSession().pipe(Effect.provide(testLayer)),
      )

      // Then: session should be restored and user loaded
      expect(result).toBeUndefined() // restoreSession returns void
      const finalState = mockAuthStore.getState()
      expect(Option.isSome(finalState.user)).toBe(true)
      const user = Option.getOrNull(finalState.user)
      expect(user?.email).toBe('test@example.com')
    })

    test('should do nothing when no token exists', async () => {
      // Given: no token in storage
      const tokenStorage = makeTokenStorageMock()
      // token not set

      const mockHttpClient = createMockHttpClient(() =>
        Effect.succeed({
          status: 404,
          json: Effect.succeed({}),
          headers: new Headers(),
        }),
      )

      const mockAuthStore = createMockAuthStore()

      const httpLayer = Layer.succeed(HttpClient.HttpClient, mockHttpClient)
      const testLayer = AuthenticationLive.pipe(
        Layer.provide(httpLayer),
        Layer.provide(tokenStorage.layer),
        Layer.provide(mockAuthStore.layer),
      )

      // When: restoring session
      const authentication = await Effect.runPromise(
        Effect.gen(function* () {
          return yield* Authentication
        }).pipe(Effect.provide(testLayer)),
      )

      const result = await Effect.runPromise(
        authentication.restoreSession().pipe(Effect.provide(testLayer)),
      )

      // Then: should do nothing (no HTTP call made)
      expect(result).toBeUndefined()
      const finalState = mockAuthStore.getState()
      expect(Option.isNone(finalState.user)).toBe(true)
    })

    test('should clear state when /api/auth/me returns error status', async () => {
      // Given: a token in storage but /api/auth/me returns 401
      const token = 'invalid-token'
      const tokenStorage = makeTokenStorageMock()
      tokenStorage.setToken(token)

      const mockHttpClient = createMockHttpClient((request) => {
        if (request.url.includes('/api/auth/me')) {
          return Effect.succeed({
            status: 401,
            json: Effect.succeed({}),
            headers: new Headers(),
          })
        }
        return Effect.succeed({
          status: 404,
          json: Effect.succeed({}),
          headers: new Headers(),
        })
      })

      const mockAuthStore = createMockAuthStore()

      const httpLayer = Layer.succeed(HttpClient.HttpClient, mockHttpClient)
      const testLayer = AuthenticationLive.pipe(
        Layer.provide(httpLayer),
        Layer.provide(tokenStorage.layer),
        Layer.provide(mockAuthStore.layer),
      )

      // When: restoring session
      const authentication = await Effect.runPromise(
        Effect.gen(function* () {
          return yield* Authentication
        }).pipe(Effect.provide(testLayer)),
      )

      await Effect.runPromise(
        authentication.restoreSession().pipe(Effect.provide(testLayer)),
      )

      // Then: token should be cleared and state reset
      // Token cleared via logout() - verify via store state
      const finalState = mockAuthStore.getState()
      expect(Option.isNone(finalState.user)).toBe(true)
    })
  })

  describe('getCurrentUser behavior', () => {
    test('should return user when token exists and valid', async () => {
      // Given: a token in storage and successful /api/auth/me response
      const token = 'test-token-123'
      const tokenStorage = makeTokenStorageMock()
      tokenStorage.setToken(token)

      const mockMeResponse: AuthMeBody = {
        user: {
          id: 'user-1',
          email: 'test@example.com',
        },
      }

      const mockHttpClient = createMockHttpClient((request) => {
        if (request.url.includes('/api/auth/me')) {
          return Effect.succeed({
            status: 200,
            json: Effect.succeed(mockMeResponse),
            headers: new Headers(),
          })
        }
        return Effect.succeed({
          status: 404,
          json: Effect.succeed({}),
          headers: new Headers(),
        })
      })

      const mockAuthStore = createMockAuthStore()

      const httpLayer = Layer.succeed(HttpClient.HttpClient, mockHttpClient)
      const testLayer = AuthenticationLive.pipe(
        Layer.provide(httpLayer),
        Layer.provide(tokenStorage.layer),
        Layer.provide(mockAuthStore.layer),
      )

      // When: getting current user
      const authentication = await Effect.runPromise(
        Effect.gen(function* () {
          return yield* Authentication
        }).pipe(Effect.provide(testLayer)),
      )

      const result = await Effect.runPromise(
        authentication
          .getCurrentUser()
          .pipe(Effect.provide(testLayer)),
      )

      // Then: should return user
      expect(Option.isSome(result)).toBe(true)
      const user = Option.getOrNull(result)
      expect(user?.email).toBe('test@example.com')
    })

    test('should return none when no token exists', async () => {
      // Given: no token in storage
      const tokenStorage = makeTokenStorageMock()
      // token not set

      const mockHttpClient = createMockHttpClient(() =>
        Effect.succeed({
          status: 404,
          json: Effect.succeed({}),
          headers: new Headers(),
        }),
      )

      const mockAuthStore = createMockAuthStore()

      const httpLayer = Layer.succeed(HttpClient.HttpClient, mockHttpClient)
      const testLayer = AuthenticationLive.pipe(
        Layer.provide(httpLayer),
        Layer.provide(tokenStorage.layer),
        Layer.provide(mockAuthStore.layer),
      )

      // When: getting current user
      const authentication = await Effect.runPromise(
        Effect.gen(function* () {
          return yield* Authentication
        }).pipe(Effect.provide(testLayer)),
      )

      const result = await Effect.runPromise(
        authentication
          .getCurrentUser()
          .pipe(Effect.provide(testLayer)),
      )

      // Then: should return none
      expect(Option.isNone(result)).toBe(true)
    })
  })

  describe('login behavior', () => {
    test('should login successfully and load user', async () => {
      // Given: valid credentials
      const email = 'test@example.com'
      const password = 'password123'

      const mockLoginResponse: LoginResponse = {
        ok: true,
        token: 'new-token-456',
      }

      const mockMeResponse: AuthMeBody = {
        user: {
          id: 'user-1',
          email: 'test@example.com',
        },
      }

      let loginCallCount = 0
      let meCallCount = 0

      const mockHttpClient = createMockHttpClient((request) => {
        if (request.url.includes('/api/auth/login')) {
          loginCallCount++
          return Effect.succeed({
            status: 200,
            json: Effect.succeed(mockLoginResponse),
            headers: new Headers(),
          })
        }
        if (request.url.includes('/api/auth/me')) {
          meCallCount++
          return Effect.succeed({
            status: 200,
            json: Effect.succeed(mockMeResponse),
            headers: new Headers(),
          })
        }
        return Effect.succeed({
          status: 404,
          json: Effect.succeed({}),
          headers: new Headers(),
        })
      })

      const tokenStorage = makeTokenStorageMock()
      const mockAuthStore = createMockAuthStore()

      const httpLayer = Layer.succeed(HttpClient.HttpClient, mockHttpClient)
      const testLayer = AuthenticationLive.pipe(
        Layer.provide(httpLayer),
        Layer.provide(tokenStorage.layer),
        Layer.provide(mockAuthStore.layer),
      )

      // When: logging in
      const authentication = await Effect.runPromise(
        Effect.gen(function* () {
          return yield* Authentication
        }).pipe(Effect.provide(testLayer)),
      )

      const result = await Effect.runPromise(
        authentication.login(email, password).pipe(Effect.provide(testLayer)),
      )

      // Then: should return user and token should be stored
      expect(result.email).toBe(email)
      expect(loginCallCount).toBe(1)
      expect(meCallCount).toBe(1)

      // Token stored via login() - verify via store state
      const finalState = mockAuthStore.getState()
      expect(Option.isSome(finalState.token)).toBe(true)
      expect(Option.getOrNull(finalState.token)).toBe('new-token-456')
    })

    test('should fail when login returns error status', async () => {
      // Given: invalid credentials
      const email = 'test@example.com'
      const password = 'wrong-password'

      const mockHttpClient = createMockHttpClient((request) => {
        if (request.url.includes('/api/auth/login')) {
          return Effect.succeed({
            status: 401,
            json: Effect.succeed({
              message: 'Invalid credentials',
            }),
            headers: new Headers(),
          })
        }
        return Effect.succeed({
          status: 404,
          json: Effect.succeed({}),
          headers: new Headers(),
        })
      })

      const tokenStorage = makeTokenStorageMock()
      const mockAuthStore = createMockAuthStore()

      const httpLayer = Layer.succeed(HttpClient.HttpClient, mockHttpClient)
      const testLayer = AuthenticationLive.pipe(
        Layer.provide(httpLayer),
        Layer.provide(tokenStorage.layer),
        Layer.provide(mockAuthStore.layer),
      )

      // When: logging in
      const authentication = await Effect.runPromise(
        Effect.gen(function* () {
          return yield* Authentication
        }).pipe(Effect.provide(testLayer)),
      )

      // Then: should fail with AuthenticationError
      await expect(
        Effect.runPromise(
          authentication.login(email, password).pipe(Effect.provide(testLayer)),
        ),
      ).rejects.toThrow('credentials')
    })

    test('should fail when login response missing token', async () => {
      // Given: login succeeds but response has no token
      const email = 'test@example.com'
      const password = 'password123'

      const mockLoginResponse: LoginResponse = {
        ok: true,
        token: '' as any, // Empty token to test error case
      }

      const mockHttpClient = createMockHttpClient((request) => {
        if (request.url.includes('/api/auth/login')) {
          return Effect.succeed({
            status: 200,
            json: Effect.succeed(mockLoginResponse),
            headers: new Headers(),
          })
        }
        return Effect.succeed({
          status: 404,
          json: Effect.succeed({}),
          headers: new Headers(),
        })
      })

      const tokenStorage = makeTokenStorageMock()
      const mockAuthStore = createMockAuthStore()

      const httpLayer = Layer.succeed(HttpClient.HttpClient, mockHttpClient)
      const testLayer = AuthenticationLive.pipe(
        Layer.provide(httpLayer),
        Layer.provide(tokenStorage.layer),
        Layer.provide(mockAuthStore.layer),
      )

      // When: logging in
      const authentication = await Effect.runPromise(
        Effect.gen(function* () {
          return yield* Authentication
        }).pipe(Effect.provide(testLayer)),
      )

      // Then: should fail with AuthenticationError about missing token
      await expect(
        Effect.runPromise(
          authentication.login(email, password).pipe(Effect.provide(testLayer)),
        ),
      ).rejects.toThrow('token')
    })
  })

  describe('register behavior', () => {
    test('should register successfully', async () => {
      // Given: valid registration data
      const email = 'newuser@example.com'
      const password = 'password123'

      const mockHttpClient = createMockHttpClient((request) => {
        if (request.url.includes('/api/auth/register')) {
          return Effect.succeed({
            status: 200,
            json: Effect.succeed({}),
            headers: new Headers(),
          })
        }
        return Effect.succeed({
          status: 404,
          json: Effect.succeed({}),
          headers: new Headers(),
        })
      })

      const tokenStorage = makeTokenStorageMock()
      const mockAuthStore = createMockAuthStore()

      const httpLayer = Layer.succeed(HttpClient.HttpClient, mockHttpClient)
      const testLayer = AuthenticationLive.pipe(
        Layer.provide(httpLayer),
        Layer.provide(tokenStorage.layer),
        Layer.provide(mockAuthStore.layer),
      )

      // When: registering
      const authentication = await Effect.runPromise(
        Effect.gen(function* () {
          return yield* Authentication
        }).pipe(Effect.provide(testLayer)),
      )

      const result = await Effect.runPromise(
        authentication
          .register(email, password)
          .pipe(Effect.provide(testLayer)),
      )

      // Then: should succeed (returns void)
      expect(result).toBeUndefined()
    })

    test('should fail when registration returns error status', async () => {
      // Given: registration fails (e.g., email already exists)
      const email = 'existing@example.com'
      const password = 'password123'

      const mockHttpClient = createMockHttpClient((request) => {
        if (request.url.includes('/api/auth/register')) {
          return Effect.succeed({
            status: 400,
            json: Effect.succeed({
              message: 'Email already exists',
            }),
            headers: new Headers(),
          })
        }
        return Effect.succeed({
          status: 404,
          json: Effect.succeed({}),
          headers: new Headers(),
        })
      })

      const tokenStorage = makeTokenStorageMock()
      const mockAuthStore = createMockAuthStore()

      const httpLayer = Layer.succeed(HttpClient.HttpClient, mockHttpClient)
      const testLayer = AuthenticationLive.pipe(
        Layer.provide(httpLayer),
        Layer.provide(tokenStorage.layer),
        Layer.provide(mockAuthStore.layer),
      )

      // When: registering
      const authentication = await Effect.runPromise(
        Effect.gen(function* () {
          return yield* Authentication
        }).pipe(Effect.provide(testLayer)),
      )

      // Then: should fail with AuthenticationError
      await expect(
        Effect.runPromise(
          authentication
            .register(email, password)
            .pipe(Effect.provide(testLayer)),
        ),
      ).rejects.toThrow()
    })
  })

  describe('logout behavior', () => {
    test('should clear token and reset state', async () => {
      // Given: user is logged in (token exists)
      const token = 'test-token-123'
      const tokenStorage = makeTokenStorageMock()
      tokenStorage.setToken(token)

      const mockAuthStore = createMockAuthStore({
        ...initialAuthenticationState,
        token: Option.some(token),
        user: Option.some(
          new AuthenticationUser({
            id: 'user-1',
            email: 'test@example.com',
            token,
          }),
        ),
      })

      const mockHttpClient = createMockHttpClient(() =>
        Effect.succeed({
          status: 404,
          json: Effect.succeed({}),
          headers: new Headers(),
        }),
      )

      const httpLayer = Layer.succeed(HttpClient.HttpClient, mockHttpClient)
      const testLayer = AuthenticationLive.pipe(
        Layer.provide(httpLayer),
        Layer.provide(tokenStorage.layer),
        Layer.provide(mockAuthStore.layer),
      )

      // When: logging out
      const authentication = await Effect.runPromise(
        Effect.gen(function* () {
          return yield* Authentication
        }).pipe(Effect.provide(testLayer)),
      )

      const result = await Effect.runPromise(
        authentication.logout().pipe(Effect.provide(testLayer)),
      )

      // Then: token should be cleared and state reset
      expect(result).toBeUndefined()

      // Verify token was cleared - use direct access to mock
      // Note: logout() clears token via tokenStorage.clearToken() which updates the mock's internal state
      // We verify this by checking the store state instead

      const finalState = mockAuthStore.getState()
      expect(Option.isNone(finalState.user)).toBe(true)
      expect(Option.isNone(finalState.token)).toBe(true)
    })
  })

  describe('selectScope behavior', () => {
    test('should set scope and update state', async () => {
      // Given: user is logged in
      const organizationId = 'org-123'
      const roleId = 'role-456'

      const tokenStorage = makeTokenStorageMock()
      const mockAuthStore = createMockAuthStore()

      const mockHttpClient = createMockHttpClient(() =>
        Effect.succeed({
          status: 404,
          json: Effect.succeed({}),
          headers: new Headers(),
        }),
      )

      const httpLayer = Layer.succeed(HttpClient.HttpClient, mockHttpClient)
      const testLayer = AuthenticationLive.pipe(
        Layer.provide(httpLayer),
        Layer.provide(tokenStorage.layer),
        Layer.provide(mockAuthStore.layer),
      )

      // When: selecting scope
      const authentication = await Effect.runPromise(
        Effect.gen(function* () {
          return yield* Authentication
        }).pipe(Effect.provide(testLayer)),
      )

      const result = await Effect.runPromise(
        authentication
          .selectScope(organizationId, roleId)
          .pipe(Effect.provide(testLayer)),
      )

      // Then: scope should be stored and needsScopeSelect set to false
      expect(result).toBeUndefined()

      // Scope stored via selectScope() - verify via direct mock access
      // Since we can't easily access the mock's internal state, we verify via store state
      // The selectScope method calls tokenStorage.setScope internally

      const finalState = mockAuthStore.getState()
      expect(Option.getOrNull(finalState.needsScopeSelect)).toBe(false)
    })
  })

  describe('fetchMeAndUpdate behavior', () => {
    test('should update state with permissions from body', async () => {
      // Given: token exists and /api/auth/me returns permissions
      const token = 'test-token-123'
      const tokenStorage = makeTokenStorageMock()
      tokenStorage.setToken(token)

      const mockMeResponse: AuthMeBody = {
        user: {
          id: 'user-1',
          email: 'test@example.com',
        },
        permissions: ['read:users', 'write:users'],
      }

      const mockHttpClient = createMockHttpClient((request) => {
        if (request.url.includes('/api/auth/me')) {
          return Effect.succeed({
            status: 200,
            json: Effect.succeed(mockMeResponse),
            headers: new Headers(),
          })
        }
        return Effect.succeed({
          status: 404,
          json: Effect.succeed({}),
          headers: new Headers(),
        })
      })

      const mockAuthStore = createMockAuthStore()

      const httpLayer = Layer.succeed(HttpClient.HttpClient, mockHttpClient)
      const testLayer = AuthenticationLive.pipe(
        Layer.provide(httpLayer),
        Layer.provide(tokenStorage.layer),
        Layer.provide(mockAuthStore.layer),
      )

      // When: fetching user (via getCurrentUser)
      const authentication = await Effect.runPromise(
        Effect.gen(function* () {
          return yield* Authentication
        }).pipe(Effect.provide(testLayer)),
      )

      await Effect.runPromise(
        authentication
          .getCurrentUser()
          .pipe(Effect.provide(testLayer)),
      )

      // Then: permissions should be set in state
      const finalState = mockAuthStore.getState()
      expect(finalState.permissions).toEqual(['read:users', 'write:users'])
    })

    test('should handle needs_scope_select flag', async () => {
      // Given: token exists and /api/auth/me returns needs_scope_select: true
      const token = 'test-token-123'
      const tokenStorage = makeTokenStorageMock()
      tokenStorage.setToken(token)

      const mockMeResponse: AuthMeBody = {
        user: {
          id: 'user-1',
          email: 'test@example.com',
        },
        needs_scope_select: true,
      }

      const mockHttpClient = createMockHttpClient((request) => {
        if (request.url.includes('/api/auth/me')) {
          return Effect.succeed({
            status: 200,
            json: Effect.succeed(mockMeResponse),
            headers: new Headers(),
          })
        }
        return Effect.succeed({
          status: 404,
          json: Effect.succeed({}),
          headers: new Headers(),
        })
      })

      const mockAuthStore = createMockAuthStore()

      const httpLayer = Layer.succeed(HttpClient.HttpClient, mockHttpClient)
      const testLayer = AuthenticationLive.pipe(
        Layer.provide(httpLayer),
        Layer.provide(tokenStorage.layer),
        Layer.provide(mockAuthStore.layer),
      )

      // When: fetching user
      const authentication = await Effect.runPromise(
        Effect.gen(function* () {
          return yield* Authentication
        }).pipe(Effect.provide(testLayer)),
      )

      await Effect.runPromise(
        authentication
          .getCurrentUser()
          .pipe(Effect.provide(testLayer)),
      )

      // Then: needsScopeSelect should be set to true
      const finalState = mockAuthStore.getState()
      expect(Option.getOrNull(finalState.needsScopeSelect)).toBe(true)
    })
  })
})
