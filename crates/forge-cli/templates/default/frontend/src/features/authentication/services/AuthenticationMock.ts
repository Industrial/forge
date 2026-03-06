/**
 * Mock implementation of Authentication for tests.
 * In-memory user and scope; no localStorage or real HTTP. Use Layer.succeed(Authentication, mock)
 * or createMockAuthentication() and provide via Effect.provide(mockLayer).
 *
 * @see effect.ts-testing – never use vi.mock() for Effect services; use this Layer instead.
 */
import { Effect, Option } from 'effect'
import { Layer } from 'effect'

import { Authentication } from '@/features/authentication/services/Authentication'
import { AuthenticationUser } from '@/features/authentication/domain/AuthenticationUser'
import { AuthenticationError } from '@/features/authentication/errors/AuthenticationError'
import { ScopeError } from '@/features/authentication/errors'

export interface MockAuthenticationState {
  user: Option.Option<AuthenticationUser>
  scope: { organizationId: string; roleId: string } | null
  /** If set, getCurrentUser fails with this error. */
  getCurrentUserError: Option.Option<AuthenticationError>
  /** If set, login fails with this error. */
  loginError: Option.Option<AuthenticationError>
  /** If set, selectScope fails with this error. */
  selectScopeError: Option.Option<ScopeError>
}

/**
 * Creates a mock Authentication service and control API for tests.
 * Each test can set user, scope, and error behaviour without module mocks.
 *
 * @example
 * const { authentication, setUser, clearUser, setLoginError } = createMockAuthentication()
 * const mockLayer = Layer.succeed(Authentication, authentication)
 * const result = await Effect.runPromise(
 *   myEffect.pipe(Effect.provide(mockLayer))
 * )
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
  setSelectScopeError: (error: ScopeError | null) => void
} {
  const state: MockAuthenticationState = {
    user: Option.none(),
    scope: null,
    getCurrentUserError: Option.none(),
    loginError: Option.none(),
    selectScopeError: Option.none(),
  }

  const setUser = (user: AuthenticationUser | null) => {
    state.user = user != null ? Option.some(user) : Option.none()
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
    state.getCurrentUserError =
      error != null ? Option.some(error) : Option.none()
  }

  const setLoginError = (error: AuthenticationError | null) => {
    state.loginError = error != null ? Option.some(error) : Option.none()
  }

  const setSelectScopeError = (error: ScopeError | null) => {
    state.selectScopeError = error != null ? Option.some(error) : Option.none()
  }

  const authentication: Authentication = {
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
    setSelectScopeError,
  }
}

/** Layer that provides a default mock Authentication. For per-test control use createMockAuthentication() and Layer.succeed(Authentication, result.authentication). */
export const AuthenticationMockLayer = Layer.succeed(
  Authentication,
  createMockAuthentication().authentication,
)
