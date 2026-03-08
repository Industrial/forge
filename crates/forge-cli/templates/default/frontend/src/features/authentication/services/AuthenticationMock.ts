/**
 * Mock implementation of Authentication for testing.
 * Follows Effect.ts testing patterns - creates real mock objects, not vi.fn() mocks.
 *
 * This mock provides a controllable authentication service for testing without
 * requiring actual HTTP calls or persistent storage. It maintains in-memory state
 * and allows tests to configure behavior (success/failure responses).
 *
 * @example
 * ```typescript
 * import { createMockAuthentication } from './AuthenticationMock'
 * import { Effect, Layer } from 'effect'
 * import { Authentication } from './Authentication'
 *
 * describe('myFeature', () => {
 *   const { authentication, setUser, setLoginError } = createMockAuthentication()
 *   const mockLayer = Layer.succeed(Authentication, authentication)
 *
 *   it('should handle authenticated user', async () => {
 *     const user = new AuthenticationUser({ id: '1', email: 'test@example.com', token: 'token' })
 *     setUser(user)
 *
 *     const result = await Effect.runPromise(
 *       myFeature().pipe(Effect.provide(mockLayer))
 *     )
 *
 *     expect(result).toBeDefined()
 *   })
 * })
 * ```
 */

import { Effect, Option } from 'effect'
import { Authentication } from './Authentication'
import { AuthenticationUser } from '../domain/AuthenticationUser'
import { AuthenticationError } from '../errors/AuthenticationError'
import { ScopeError } from '../errors'

/** Internal state for the mock authentication service */
interface MockAuthenticationState {
  user: Option.Option<AuthenticationUser>
  scope: { organizationId: string; roleId: string } | null
  getCurrentUserError: Option.Option<AuthenticationError>
  loginError: Option.Option<AuthenticationError>
  registerError: Option.Option<AuthenticationError>
  selectScopeError: Option.Option<ScopeError>
}

/**
 * Creates a mock authentication service that implements the Authentication interface.
 * Provides full control over authentication state and error conditions for testing.
 *
 * @returns An object containing:
 *   - authentication: The mock Authentication service
 *   - state: Internal state (for inspection in tests)
 *   - Control functions to configure mock behavior
 */
export function createMockAuthentication(): {
  authentication: Authentication
  state: MockAuthenticationState
  setUser: (user: AuthenticationUser | null) => void
  clearUser: () => void
  setScope: (organizationId: string, roleId: string) => void
  clearScope: () => void
  setGetCurrentUserError: (error: AuthenticationError | null) => void
  setLoginError: (error: AuthenticationError | null) => void
  setRegisterError: (error: AuthenticationError | null) => void
  setSelectScopeError: (error: ScopeError | null) => void
} {
  // Internal state
  const state: MockAuthenticationState = {
    user: Option.none(),
    scope: null,
    getCurrentUserError: Option.none(),
    loginError: Option.none(),
    registerError: Option.none(),
    selectScopeError: Option.none(),
  }

  // Control functions
  const setUser = (user: AuthenticationUser | null) => {
    state.user = user ? Option.some(user) : Option.none()
  }

  const clearUser = () => {
    state.user = Option.none()
  }

  const setScope = (organizationId: string, roleId: string) => {
    state.scope = { organizationId, roleId }
  }

  const clearScope = () => {
    state.scope = null
  }

  const setGetCurrentUserError = (error: AuthenticationError | null) => {
    state.getCurrentUserError = error ? Option.some(error) : Option.none()
  }

  const setLoginError = (error: AuthenticationError | null) => {
    state.loginError = error ? Option.some(error) : Option.none()
  }

  const setRegisterError = (error: AuthenticationError | null) => {
    state.registerError = error ? Option.some(error) : Option.none()
  }

  const setSelectScopeError = (error: ScopeError | null) => {
    state.selectScopeError = error ? Option.some(error) : Option.none()
  }

  // Mock Authentication implementation
  const authentication: Authentication = {
    restoreSession: () => Effect.void,

    getCurrentUser: () =>
      Option.match(state.getCurrentUserError, {
        onNone: () => Effect.succeed(state.user),
        onSome: (e) => Effect.fail(e),
      }),

    login: (email: string, password: string) =>
      Option.match(state.loginError, {
        onNone: () => {
          const user = new AuthenticationUser({
            id: 'mock-id',
            email,
            token: `mock-token-${email}-${password}`,
          })
          state.user = Option.some(user)
          return Effect.succeed(user)
        },
        onSome: (e) => Effect.fail(e),
      }),

    register: (_email: string, _password: string) =>
      Option.match(state.registerError, {
        onNone: () => Effect.void,
        onSome: (e) => Effect.fail(e),
      }),

    logout: () =>
      Effect.sync(() => {
        state.user = Option.none()
        state.scope = null
      }),

    selectScope: (organizationId: string, roleId: string) =>
      Option.match(state.selectScopeError, {
        onNone: () =>
          Effect.sync(() => {
            state.scope = { organizationId, roleId }
          }),
        onSome: (e) => Effect.fail(e),
      }),
  }

  return {
    authentication,
    state,
    setUser,
    clearUser,
    setScope,
    clearScope,
    setGetCurrentUserError,
    setLoginError,
    setRegisterError,
    setSelectScopeError,
  }
}

/**
 * Default mock Authentication Layer for simple test cases.
 * For tests that need more control, use createMockAuthentication() and Layer.succeed().
 */
export const MockAuthenticationLayer = Effect.sync(
  () => createMockAuthentication().authentication,
).pipe(Effect.map((auth) => ({ [Authentication.key]: auth })))
