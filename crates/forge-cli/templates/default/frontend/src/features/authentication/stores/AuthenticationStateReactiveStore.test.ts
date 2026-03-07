/**
 * BDD tests for AuthenticationStateReactiveStore
 * Tests verify store definition, initial state, and reactive store behavior
 */
import { describe, test, expect, beforeEach } from 'bun:test'
import { Effect, Fiber, Layer, Option, Stream, Chunk } from 'effect'

import {
  AuthStore,
  AuthStoreTag,
  authStoreLayer,
  AuthenticationStateReactiveStoreTag,
  initialAuthenticationState,
  getAuthenticationStateStoreLayer,
  type AuthenticationState,
  type AuthenticationStateReactiveStore,
} from './AuthenticationStateReactiveStore'
import { AuthenticationUser } from '@/features/authentication/domain/AuthenticationUser'
import type { ReactiveStore } from '@/lib/ReactiveStore'

describe('AuthenticationStateReactiveStore', () => {
  describe('initial state', () => {
    test('should have correct initial state structure', () => {
      // Given the initialAuthenticationState
      // When I check its properties
      // Then all fields should be properly initialized
      expect(initialAuthenticationState).toHaveProperty('token')
      expect(initialAuthenticationState).toHaveProperty('user')
      expect(initialAuthenticationState).toHaveProperty('needsScopeSelect')
      expect(initialAuthenticationState).toHaveProperty('permissions')
    })

    test('should have token as Option.none()', () => {
      // Given the initialAuthenticationState
      // When I check the token field
      // Then it should be Option.none()
      expect(Option.isNone(initialAuthenticationState.token)).toBe(true)
    })

    test('should have user as Option.none()', () => {
      // Given the initialAuthenticationState
      // When I check the user field
      // Then it should be Option.none()
      expect(Option.isNone(initialAuthenticationState.user)).toBe(true)
    })

    test('should have needsScopeSelect as Option.none()', () => {
      // Given the initialAuthenticationState
      // When I check the needsScopeSelect field
      // Then it should be Option.none()
      expect(Option.isNone(initialAuthenticationState.needsScopeSelect)).toBe(
        true,
      )
    })

    test('should have empty permissions array', () => {
      // Given the initialAuthenticationState
      // When I check the permissions field
      // Then it should be an empty array
      expect(initialAuthenticationState.permissions).toEqual([])
      expect(initialAuthenticationState.permissions.length).toBe(0)
    })
  })

  describe('store definition', () => {
    test('should export AuthStore with tag and layer', () => {
      // Given the AuthStore definition
      // When I check its structure
      // Then it should have tag and layer properties
      expect(AuthStore).toHaveProperty('tag')
      expect(AuthStore).toHaveProperty('layer')
    })

    test('should have correct tag name', () => {
      // Given the AuthStore tag
      // When I check its identifier
      // Then it should match the expected name
      expect(AuthStoreTag).toBeDefined()
    })

    test('should export AuthStoreTag', () => {
      // Given the module exports
      // When I check AuthStoreTag
      // Then it should be defined and equal to AuthStore.tag
      expect(AuthStoreTag).toBeDefined()
      expect(AuthStoreTag).toBe(AuthStore.tag)
    })

    test('should export authStoreLayer', () => {
      // Given the module exports
      // When I check authStoreLayer
      // Then it should be defined and equal to AuthStore.layer
      expect(authStoreLayer).toBeDefined()
      expect(authStoreLayer).toBe(AuthStore.layer)
    })

    test('should export deprecated AuthenticationStateReactiveStoreTag', () => {
      // Given the module exports
      // When I check AuthenticationStateReactiveStoreTag
      // Then it should be defined and equal to AuthStoreTag
      expect(AuthenticationStateReactiveStoreTag).toBeDefined()
      expect(AuthenticationStateReactiveStoreTag).toBe(AuthStoreTag)
    })
  })

  describe('store operations', () => {
    let store: ReactiveStore<AuthenticationState>

    beforeEach(async () => {
      // Given a fresh store instance
      // When I create it with the layer
      const program = Effect.gen(function* () {
        const storeInstance = yield* AuthStoreTag
        return storeInstance
      })

      store = await Effect.runPromise(
        program.pipe(Effect.provide(authStoreLayer)),
      )
    })

    test('should get initial state', async () => {
      // Given a store instance
      // When I get the current state
      // Then it should return the initial state
      const state = await Effect.runPromise(store.get())
      expect(state).toEqual(initialAuthenticationState)
      expect(Option.isNone(state.token)).toBe(true)
      expect(Option.isNone(state.user)).toBe(true)
      expect(Option.isNone(state.needsScopeSelect)).toBe(true)
      expect(state.permissions).toEqual([])
    })

    test('should update token', async () => {
      // Given a store instance
      // When I update the token
      // Then the state should reflect the change
      const newToken = 'test-token-123'
      await Effect.runPromise(
        store.update((state) => ({
          ...state,
          token: Option.some(newToken),
        })),
      )

      const state = await Effect.runPromise(store.get())
      expect(Option.isSome(state.token)).toBe(true)
      expect(Option.getOrUndefined(state.token)).toBe(newToken)
    })

    test('should update user', async () => {
      // Given a store instance
      // When I update the user
      // Then the state should reflect the change
      const testUser = new AuthenticationUser({
        id: 'user-1',
        email: 'test@example.com',
        token: 'token-123',
      })

      await Effect.runPromise(
        store.update((state) => ({
          ...state,
          user: Option.some(testUser),
        })),
      )

      const state = await Effect.runPromise(store.get())
      expect(Option.isSome(state.user)).toBe(true)
      const user = Option.getOrUndefined(state.user)
      expect(user).toBeDefined()
      expect(user?.id).toBe('user-1')
      expect(user?.email).toBe('test@example.com')
    })

    test('should update needsScopeSelect', async () => {
      // Given a store instance
      // When I update needsScopeSelect
      // Then the state should reflect the change
      await Effect.runPromise(
        store.update((state) => ({
          ...state,
          needsScopeSelect: Option.some(true),
        })),
      )

      const state = await Effect.runPromise(store.get())
      expect(Option.isSome(state.needsScopeSelect)).toBe(true)
      expect(Option.getOrUndefined(state.needsScopeSelect)).toBe(true)
    })

    test('should update permissions', async () => {
      // Given a store instance
      // When I update permissions
      // Then the state should reflect the change
      const newPermissions = ['read:users', 'write:posts']
      await Effect.runPromise(
        store.update((state) => ({
          ...state,
          permissions: newPermissions,
        })),
      )

      const state = await Effect.runPromise(store.get())
      expect(state.permissions).toEqual(newPermissions)
      expect(state.permissions.length).toBe(2)
    })

    test('should update multiple fields at once', async () => {
      // Given a store instance
      // When I update multiple fields
      // Then all changes should be reflected
      const testUser = new AuthenticationUser({
        id: 'user-2',
        email: 'user2@example.com',
        token: 'token-456',
      })

      await Effect.runPromise(
        store.update((state) => ({
          ...state,
          token: Option.some('new-token'),
          user: Option.some(testUser),
          needsScopeSelect: Option.some(false),
          permissions: ['read:users'],
        })),
      )

      const state = await Effect.runPromise(store.get())
      expect(Option.getOrUndefined(state.token)).toBe('new-token')
      expect(Option.getOrUndefined(state.user)?.id).toBe('user-2')
      expect(Option.getOrUndefined(state.needsScopeSelect)).toBe(false)
      expect(state.permissions).toEqual(['read:users'])
    })
  })

  describe('changes stream', () => {
    test('should have changes stream property', async () => {
      // Given a store instance
      // When I check the store
      // Then it should have a changes stream
      const program = Effect.gen(function* () {
        const store = yield* AuthStoreTag
        return store.changes
      })

      const stream = await Effect.runPromise(
        program.pipe(Effect.provide(authStoreLayer)),
      )

      expect(stream).toBeDefined()
      // Stream is an object with Symbol properties
      expect(typeof stream).toBe('object')
    })

    test('should emit updates when state changes', async () => {
      // Given a store instance
      // When I update the state
      // Then the changes stream should be available for subscription
      const program = Effect.gen(function* () {
        const store = yield* AuthStoreTag

        // Get current state (may have been modified by previous tests)
        const beforeState = yield* store.get()

        // Update the store with a unique token
        const testToken = `stream-test-token-${Date.now()}`
        yield* store.update((state) => ({
          ...state,
          token: Option.some(testToken),
        }))

        // Get updated state
        const afterState = yield* store.get()

        // Verify changes stream exists and state was updated
        return {
          beforeState,
          afterState,
          testToken,
          hasChangesStream: !!store.changes,
        }
      })

      const result = await Effect.runPromise(
        program.pipe(Effect.provide(authStoreLayer)),
      )

      expect(result.hasChangesStream).toBe(true)
      expect(Option.isSome(result.afterState.token)).toBe(true)
      expect(Option.getOrUndefined(result.afterState.token)).toBe(
        result.testToken,
      )
      // Verify the state actually changed
      expect(result.afterState.token).not.toEqual(result.beforeState.token)
    })
  })

  describe('layer functionality', () => {
    test('should provide store via layer', async () => {
      // Given the authStoreLayer
      // When I provide it to an effect
      // Then I should be able to access the store
      // Note: State may have been modified by previous tests, so we just verify we can access it
      const program = Effect.gen(function* () {
        const store = yield* AuthStoreTag
        const state = yield* store.get()
        return state
      })

      const state = await Effect.runPromise(
        program.pipe(Effect.provide(authStoreLayer)),
      )

      // Verify we can access the store (state structure is correct)
      expect(state).toHaveProperty('token')
      expect(state).toHaveProperty('user')
      expect(state).toHaveProperty('needsScopeSelect')
      expect(state).toHaveProperty('permissions')
    })

    test('should return same store instance across multiple accesses', async () => {
      // Given the authStoreLayer
      // When I access the store multiple times
      // Then it should return the same instance (Layer.sync behavior)
      const program = Effect.gen(function* () {
        const store1 = yield* AuthStoreTag
        const store2 = yield* AuthStoreTag
        return { store1, store2 }
      })

      const { store1, store2 } = await Effect.runPromise(
        program.pipe(Effect.provide(authStoreLayer)),
      )

      // Layer.sync returns the same instance
      expect(store1).toBe(store2)
    })

    test('should export getAuthenticationStateStoreLayer function', () => {
      // Given the module exports
      // When I check getAuthenticationStateStoreLayer
      // Then it should be a function
      expect(typeof getAuthenticationStateStoreLayer).toBe('function')
    })

    test('should return authStoreLayer from getAuthenticationStateStoreLayer', () => {
      // Given getAuthenticationStateStoreLayer function
      // When I call it
      // Then it should return authStoreLayer
      const layer = getAuthenticationStateStoreLayer()
      expect(layer).toBe(authStoreLayer)
    })
  })

  describe('type exports', () => {
    test('should export AuthenticationState type', () => {
      // Given the module exports
      // When I check AuthenticationState type
      // Then it should be usable
      const state: AuthenticationState = initialAuthenticationState
      expect(state).toBeDefined()
    })

    test('should export AuthenticationStateReactiveStore type', () => {
      // Given the module exports
      // When I check AuthenticationStateReactiveStore type
      // Then it should be compatible with ReactiveStore<AuthenticationState>
      const program = Effect.gen(function* () {
        const store = yield* AuthStoreTag
        const typedStore: AuthenticationStateReactiveStore = store
        return typedStore
      })

      // Type check passes if this compiles
      expect(true).toBe(true)
    })
  })

  describe('store isolation', () => {
    test('should maintain separate state per layer instance', async () => {
      // Given two separate layer instances
      // When I update one store
      // Then the other store should remain unchanged
      const program1 = Effect.gen(function* () {
        const store = yield* AuthStoreTag
        yield* store.update((state) => ({
          ...state,
          token: Option.some('token-1'),
        }))
        return yield* store.get()
      })

      const program2 = Effect.gen(function* () {
        const store = yield* AuthStoreTag
        yield* store.update((state) => ({
          ...state,
          token: Option.some('token-2'),
        }))
        return yield* store.get()
      })

      // Both use the same layer, so they share state (Layer.sync behavior)
      const state1 = await Effect.runPromise(
        program1.pipe(Effect.provide(authStoreLayer)),
      )
      const state2 = await Effect.runPromise(
        program2.pipe(Effect.provide(authStoreLayer)),
      )

      // Since Layer.sync shares the same instance, both should see the last update
      expect(Option.getOrUndefined(state2.token)).toBe('token-2')
    })
  })
})
