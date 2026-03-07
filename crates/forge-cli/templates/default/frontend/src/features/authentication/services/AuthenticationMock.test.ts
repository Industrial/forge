/**
 * BDD tests for AuthenticationMock
 * Tests verify the behavior of the mock Authentication implementation for testing
 */

import { describe, it, expect, beforeEach } from 'bun:test'
import { Effect, Option, Layer } from 'effect'
import {
  createMockAuthentication,
  AuthenticationMockLayer,
  type MockAuthenticationState,
} from './AuthenticationMock'
import { Authentication } from './Authentication'
import { AuthenticationUser } from '@/features/authentication/domain/AuthenticationUser'
import { AuthenticationError } from '@/features/authentication/errors/AuthenticationError'
import { ScopeError } from '@/features/authentication/errors'

describe('AuthenticationMock', () => {
  describe('createMockAuthentication behavior', () => {
    it('should return an object with authentication, state, and control functions', () => {
      // Given createMockAuthentication function
      // When I call it
      const mock = createMockAuthentication()

      // Then it should return the expected structure
      expect(mock).toBeDefined()
      expect(mock.authentication).toBeDefined()
      expect(mock.state).toBeDefined()
      expect(mock.setUser).toBeDefined()
      expect(mock.clearUser).toBeDefined()
      expect(mock.setScope).toBeDefined()
      expect(mock.clearScope).toBeDefined()
      expect(mock.setGetCurrentUserError).toBeDefined()
      expect(mock.setLoginError).toBeDefined()
      expect(mock.setSelectScopeError).toBeDefined()
    })

    it('should initialize state with empty values', () => {
      // Given createMockAuthentication function
      // When I call it
      const { state } = createMockAuthentication()

      // Then initial state should be empty
      expect(Option.isNone(state.user)).toBe(true)
      expect(state.scope).toBeNull()
      expect(Option.isNone(state.getCurrentUserError)).toBe(true)
      expect(Option.isNone(state.loginError)).toBe(true)
      expect(Option.isNone(state.selectScopeError)).toBe(true)
    })

    it('should return an Authentication service implementation', () => {
      // Given createMockAuthentication function
      // When I call it
      const { authentication } = createMockAuthentication()

      // Then authentication should implement Authentication interface
      expect(authentication.restoreSession).toBeDefined()
      expect(authentication.getCurrentUser).toBeDefined()
      expect(authentication.login).toBeDefined()
      expect(authentication.register).toBeDefined()
      expect(authentication.logout).toBeDefined()
      expect(authentication.selectScope).toBeDefined()
    })
  })

  describe('state management behavior', () => {
    let mock: ReturnType<typeof createMockAuthentication>

    beforeEach(() => {
      mock = createMockAuthentication()
    })

    describe('setUser behavior', () => {
      it('should set user when provided', () => {
        // Given a mock and a user
        const user = new AuthenticationUser({
          id: 'test-id',
          email: 'test@example.com',
          token: 'test-token',
        })

        // When I set the user
        mock.setUser(user)

        // Then state should contain the user
        expect(Option.isSome(mock.state.user)).toBe(true)
        expect(Option.getOrUndefined(mock.state.user)).toEqual(user)
      })

      it('should clear user when set to null', () => {
        // Given a mock with a user set
        const user = new AuthenticationUser({
          id: 'test-id',
          email: 'test@example.com',
          token: 'test-token',
        })
        mock.setUser(user)

        // When I set user to null
        mock.setUser(null)

        // Then user should be cleared
        expect(Option.isNone(mock.state.user)).toBe(true)
      })
    })

    describe('clearUser behavior', () => {
      it('should clear user from state', () => {
        // Given a mock with a user set
        const user = new AuthenticationUser({
          id: 'test-id',
          email: 'test@example.com',
          token: 'test-token',
        })
        mock.setUser(user)

        // When I clear the user
        mock.clearUser()

        // Then user should be cleared
        expect(Option.isNone(mock.state.user)).toBe(true)
      })
    })

    describe('setScope behavior', () => {
      it('should set scope when provided', () => {
        // Given a mock
        // When I set scope
        mock.setScope('org-123', 'role-456')

        // Then state should contain the scope
        expect(mock.state.scope).toEqual({
          organizationId: 'org-123',
          roleId: 'role-456',
        })
      })

      it('should overwrite existing scope', () => {
        // Given a mock with existing scope
        mock.setScope('org-123', 'role-456')

        // When I set a new scope
        mock.setScope('org-789', 'role-012')

        // Then scope should be updated
        expect(mock.state.scope).toEqual({
          organizationId: 'org-789',
          roleId: 'role-012',
        })
      })
    })

    describe('clearScope behavior', () => {
      it('should clear scope from state', () => {
        // Given a mock with scope set
        mock.setScope('org-123', 'role-456')

        // When I clear the scope
        mock.clearScope()

        // Then scope should be null
        expect(mock.state.scope).toBeNull()
      })
    })

    describe('setGetCurrentUserError behavior', () => {
      it('should set error when provided', () => {
        // Given a mock and an error
        const error = new AuthenticationError({ message: 'Test error' })

        // When I set the error
        mock.setGetCurrentUserError(error)

        // Then state should contain the error
        expect(Option.isSome(mock.state.getCurrentUserError)).toBe(true)
        expect(Option.getOrUndefined(mock.state.getCurrentUserError)).toEqual(
          error,
        )
      })

      it('should clear error when set to null', () => {
        // Given a mock with error set
        const error = new AuthenticationError({ message: 'Test error' })
        mock.setGetCurrentUserError(error)

        // When I set error to null
        mock.setGetCurrentUserError(null)

        // Then error should be cleared
        expect(Option.isNone(mock.state.getCurrentUserError)).toBe(true)
      })
    })

    describe('setLoginError behavior', () => {
      it('should set error when provided', () => {
        // Given a mock and an error
        const error = new AuthenticationError({ message: 'Login failed' })

        // When I set the error
        mock.setLoginError(error)

        // Then state should contain the error
        expect(Option.isSome(mock.state.loginError)).toBe(true)
        expect(Option.getOrUndefined(mock.state.loginError)).toEqual(error)
      })

      it('should clear error when set to null', () => {
        // Given a mock with error set
        const error = new AuthenticationError({ message: 'Login failed' })
        mock.setLoginError(error)

        // When I set error to null
        mock.setLoginError(null)

        // Then error should be cleared
        expect(Option.isNone(mock.state.loginError)).toBe(true)
      })
    })

    describe('setSelectScopeError behavior', () => {
      it('should set error when provided', () => {
        // Given a mock and an error
        const error = new ScopeError({ message: 'Scope selection failed' })

        // When I set the error
        mock.setSelectScopeError(error)

        // Then state should contain the error
        expect(Option.isSome(mock.state.selectScopeError)).toBe(true)
        expect(Option.getOrUndefined(mock.state.selectScopeError)).toEqual(
          error,
        )
      })

      it('should clear error when set to null', () => {
        // Given a mock with error set
        const error = new ScopeError({ message: 'Scope selection failed' })
        mock.setSelectScopeError(error)

        // When I set error to null
        mock.setSelectScopeError(null)

        // Then error should be cleared
        expect(Option.isNone(mock.state.selectScopeError)).toBe(true)
      })
    })
  })

  describe('restoreSession behavior', () => {
    it('should return void Effect', async () => {
      // Given a mock Authentication
      const { authentication } = createMockAuthentication()
      const layer = Layer.succeed(Authentication, authentication)

      // When I call restoreSession
      const program = Effect.gen(function* () {
        const auth = yield* Authentication
        return yield* auth.restoreSession()
      })

      // Then it should complete successfully
      await expect(
        Effect.runPromise(program.pipe(Effect.provide(layer))),
      ).resolves.toBeUndefined()
    })
  })

  describe('getCurrentUser behavior', () => {
    let mock: ReturnType<typeof createMockAuthentication>
    let layer: Layer.Layer<Authentication>

    beforeEach(() => {
      mock = createMockAuthentication()
      layer = Layer.succeed(Authentication, mock.authentication)
    })

    it('should return Option.none() when no user is set', async () => {
      // Given a mock with no user
      // When I call getCurrentUser
      const program = Effect.gen(function* () {
        const auth = yield* Authentication
        return yield* auth.getCurrentUser()
      })

      // Then it should return Option.none()
      const result = await Effect.runPromise(
        program.pipe(Effect.provide(layer)),
      )
      expect(Option.isNone(result)).toBe(true)
    })

    it('should return Option.some(user) when user is set', async () => {
      // Given a mock with a user set
      const user = new AuthenticationUser({
        id: 'test-id',
        email: 'test@example.com',
        token: 'test-token',
      })
      mock.setUser(user)

      // When I call getCurrentUser
      const program = Effect.gen(function* () {
        const auth = yield* Authentication
        return yield* auth.getCurrentUser()
      })

      // Then it should return Option.some(user)
      const result = await Effect.runPromise(
        program.pipe(Effect.provide(layer)),
      )
      expect(Option.isSome(result)).toBe(true)
      expect(Option.getOrUndefined(result)).toEqual(user)
    })

    it('should fail with error when error is set', async () => {
      // Given a mock with error set
      const error = new AuthenticationError({ message: 'Get user failed' })
      mock.setGetCurrentUserError(error)

      // When I call getCurrentUser
      const program = Effect.gen(function* () {
        const auth = yield* Authentication
        return yield* auth.getCurrentUser()
      })

      // Then it should fail with the error
      await expect(
        Effect.runPromise(program.pipe(Effect.provide(layer))),
      ).rejects.toThrow('Get user failed')
    })
  })

  describe('login behavior', () => {
    let mock: ReturnType<typeof createMockAuthentication>
    let layer: Layer.Layer<Authentication>

    beforeEach(() => {
      mock = createMockAuthentication()
      layer = Layer.succeed(Authentication, mock.authentication)
    })

    it('should create and return a user on success', async () => {
      // Given a mock without login error
      const email = 'test@example.com'
      const password = 'password123'

      // When I call login
      const program = Effect.gen(function* () {
        const auth = yield* Authentication
        return yield* auth.login(email, password)
      })

      // Then it should return a user with correct properties
      const result = await Effect.runPromise(
        program.pipe(Effect.provide(layer)),
      )
      expect(result).toBeInstanceOf(AuthenticationUser)
      expect(result.email).toBe(email)
      expect(result.token).toBe(`mock-token-${email}-${password}`)
      expect(result.id).toBe('mock-id')
    })

    it('should set user in state after successful login', async () => {
      // Given a mock without login error
      const email = 'test@example.com'
      const password = 'password123'

      // When I call login
      const program = Effect.gen(function* () {
        const auth = yield* Authentication
        return yield* auth.login(email, password)
      })
      await Effect.runPromise(program.pipe(Effect.provide(layer)))

      // Then user should be set in state
      expect(Option.isSome(mock.state.user)).toBe(true)
      const user = Option.getOrUndefined(mock.state.user)!
      expect(user.email).toBe(email)
    })

    it('should fail with error when error is set', async () => {
      // Given a mock with login error set
      const error = new AuthenticationError({ message: 'Login failed' })
      mock.setLoginError(error)

      // When I call login
      const program = Effect.gen(function* () {
        const auth = yield* Authentication
        return yield* auth.login('test@example.com', 'password')
      })

      // Then it should fail with the error
      await expect(
        Effect.runPromise(program.pipe(Effect.provide(layer))),
      ).rejects.toThrow('Login failed')
    })

    it('should not set user in state when login fails', async () => {
      // Given a mock with login error set
      const error = new AuthenticationError({ message: 'Login failed' })
      mock.setLoginError(error)

      // When I call login and it fails
      const program = Effect.gen(function* () {
        const auth = yield* Authentication
        return yield* auth.login('test@example.com', 'password')
      })

      try {
        await Effect.runPromise(program.pipe(Effect.provide(layer)))
      } catch {
        // Expected to fail
      }

      // Then user should not be set in state
      expect(Option.isNone(mock.state.user)).toBe(true)
    })
  })

  describe('register behavior', () => {
    it('should return void Effect', async () => {
      // Given a mock Authentication
      const { authentication } = createMockAuthentication()
      const layer = Layer.succeed(Authentication, authentication)

      // When I call register
      const program = Effect.gen(function* () {
        const auth = yield* Authentication
        return yield* auth.register('test@example.com', 'password')
      })

      // Then it should complete successfully
      await expect(
        Effect.runPromise(program.pipe(Effect.provide(layer))),
      ).resolves.toBeUndefined()
    })
  })

  describe('logout behavior', () => {
    let mock: ReturnType<typeof createMockAuthentication>
    let layer: Layer.Layer<Authentication>

    beforeEach(() => {
      mock = createMockAuthentication()
      layer = Layer.succeed(Authentication, mock.authentication)
    })

    it('should clear user and scope from state', async () => {
      // Given a mock with user and scope set
      const user = new AuthenticationUser({
        id: 'test-id',
        email: 'test@example.com',
        token: 'test-token',
      })
      mock.setUser(user)
      mock.setScope('org-123', 'role-456')

      // When I call logout
      const program = Effect.gen(function* () {
        const auth = yield* Authentication
        return yield* auth.logout()
      })
      await Effect.runPromise(program.pipe(Effect.provide(layer)))

      // Then user and scope should be cleared
      expect(Option.isNone(mock.state.user)).toBe(true)
      expect(mock.state.scope).toBeNull()
    })

    it('should return void Effect', async () => {
      // Given a mock Authentication
      // When I call logout
      const program = Effect.gen(function* () {
        const auth = yield* Authentication
        return yield* auth.logout()
      })

      // Then it should complete successfully
      await expect(
        Effect.runPromise(program.pipe(Effect.provide(layer))),
      ).resolves.toBeUndefined()
    })
  })

  describe('selectScope behavior', () => {
    let mock: ReturnType<typeof createMockAuthentication>
    let layer: Layer.Layer<Authentication>

    beforeEach(() => {
      mock = createMockAuthentication()
      layer = Layer.succeed(Authentication, mock.authentication)
    })

    it('should set scope in state on success', async () => {
      // Given a mock without selectScope error
      const organizationId = 'org-123'
      const roleId = 'role-456'

      // When I call selectScope
      const program = Effect.gen(function* () {
        const auth = yield* Authentication
        return yield* auth.selectScope(organizationId, roleId)
      })
      await Effect.runPromise(program.pipe(Effect.provide(layer)))

      // Then scope should be set in state
      expect(mock.state.scope).toEqual({
        organizationId,
        roleId,
      })
    })

    it('should return void Effect on success', async () => {
      // Given a mock without selectScope error
      // When I call selectScope
      const program = Effect.gen(function* () {
        const auth = yield* Authentication
        return yield* auth.selectScope('org-123', 'role-456')
      })

      // Then it should complete successfully
      await expect(
        Effect.runPromise(program.pipe(Effect.provide(layer))),
      ).resolves.toBeUndefined()
    })

    it('should fail with error when error is set', async () => {
      // Given a mock with selectScope error set
      const error = new ScopeError({ message: 'Scope selection failed' })
      mock.setSelectScopeError(error)

      // When I call selectScope
      const program = Effect.gen(function* () {
        const auth = yield* Authentication
        return yield* auth.selectScope('org-123', 'role-456')
      })

      // Then it should fail with the error
      await expect(
        Effect.runPromise(program.pipe(Effect.provide(layer))),
      ).rejects.toThrow('Scope selection failed')
    })

    it('should not set scope in state when selectScope fails', async () => {
      // Given a mock with selectScope error set
      const error = new ScopeError({ message: 'Scope selection failed' })
      mock.setSelectScopeError(error)

      // When I call selectScope and it fails
      const program = Effect.gen(function* () {
        const auth = yield* Authentication
        return yield* auth.selectScope('org-123', 'role-456')
      })

      try {
        await Effect.runPromise(program.pipe(Effect.provide(layer)))
      } catch {
        // Expected to fail
      }

      // Then scope should not be set in state
      expect(mock.state.scope).toBeNull()
    })
  })

  describe('AuthenticationMockLayer behavior', () => {
    it('should provide a valid Authentication service', async () => {
      // Given AuthenticationMockLayer
      // When I use it in an Effect program
      const program = Effect.gen(function* () {
        const auth = yield* Authentication
        return auth
      })

      // Then it should provide a valid Authentication service
      const result = await Effect.runPromise(
        program.pipe(Effect.provide(AuthenticationMockLayer)),
      )
      expect(result).toBeDefined()
      expect(result.restoreSession).toBeDefined()
      expect(result.getCurrentUser).toBeDefined()
      expect(result.login).toBeDefined()
      expect(result.register).toBeDefined()
      expect(result.logout).toBeDefined()
      expect(result.selectScope).toBeDefined()
    })

    it('should allow calling methods', async () => {
      // Given AuthenticationMockLayer
      // When I call methods on the service
      const program = Effect.gen(function* () {
        const auth = yield* Authentication
        yield* auth.restoreSession()
        const user = yield* auth.getCurrentUser()
        yield* auth.logout()
        return user
      })

      // Then it should execute successfully
      const result = await Effect.runPromise(
        program.pipe(Effect.provide(AuthenticationMockLayer)),
      )
      expect(Option.isNone(result)).toBe(true)
    })
  })

  describe('integration behavior', () => {
    it('should support full authentication flow', async () => {
      // Given a mock Authentication
      const mock = createMockAuthentication()
      const layer = Layer.succeed(Authentication, mock.authentication)

      // When I perform a full authentication flow
      const program = Effect.gen(function* () {
        const auth = yield* Authentication

        // Login
        const user = yield* auth.login('test@example.com', 'password123')

        // Get current user
        const currentUser = yield* auth.getCurrentUser()

        // Select scope
        yield* auth.selectScope('org-123', 'role-456')

        // Logout
        yield* auth.logout()

        return { user, currentUser }
      })

      // Then it should complete successfully
      const result = await Effect.runPromise(
        program.pipe(Effect.provide(layer)),
      )
      expect(result.user).toBeInstanceOf(AuthenticationUser)
      expect(Option.isSome(result.currentUser)).toBe(true)
      expect(mock.state.scope).toBeNull() // Cleared by logout
    })

    it('should allow per-test state control', async () => {
      // Given a mock Authentication with pre-set user
      const mock = createMockAuthentication()
      const user = new AuthenticationUser({
        id: 'pre-set-id',
        email: 'preset@example.com',
        token: 'preset-token',
      })
      mock.setUser(user)
      mock.setScope('org-999', 'role-888')

      const layer = Layer.succeed(Authentication, mock.authentication)

      // When I get current user and scope
      const program = Effect.gen(function* () {
        const auth = yield* Authentication
        const currentUser = yield* auth.getCurrentUser()
        return currentUser
      })

      // Then it should return the pre-set user
      const result = await Effect.runPromise(
        program.pipe(Effect.provide(layer)),
      )
      expect(Option.isSome(result)).toBe(true)
      expect(Option.getOrUndefined(result)).toEqual(user)
      expect(mock.state.scope).toEqual({
        organizationId: 'org-999',
        roleId: 'role-888',
      })
    })
  })
})
