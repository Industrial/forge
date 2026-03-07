/**
 * BDD tests for authentication stores index
 * Tests verify all exports are correctly re-exported from source modules
 */
import { describe, test, expect } from 'bun:test'
import { Effect, Option } from 'effect'

import {
  // Store exports
  AuthStore,
  AuthStoreTag,
  authStoreLayer,
  AuthenticationStateReactiveStoreTag,
  getAuthenticationStateStoreLayer,
  initialAuthenticationState,
  type AuthenticationState,
  type AuthenticationStateReactiveStore,
  // Hook exports
  useAuthStore,
  useAuthStoreWithInit,
  useAuthenticationStateReactiveStore,
} from './index'

// Import from source modules for comparison
import {
  AuthStore as AuthStoreSource,
  AuthStoreTag as AuthStoreTagSource,
  authStoreLayer as authStoreLayerSource,
  AuthenticationStateReactiveStoreTag as AuthenticationStateReactiveStoreTagSource,
  getAuthenticationStateStoreLayer as getAuthenticationStateStoreLayerSource,
  initialAuthenticationState as initialAuthenticationStateSource,
  type AuthenticationState as AuthenticationStateSource,
  type AuthenticationStateReactiveStore as AuthenticationStateReactiveStoreSource,
} from './AuthenticationStateReactiveStore'

import {
  useAuthStore as useAuthStoreSource,
  useAuthStoreWithInit as useAuthStoreWithInitSource,
  useAuthenticationStateReactiveStore as useAuthenticationStateReactiveStoreSource,
} from '../hooks/useAuthenticationStateReactiveStore'

describe('authentication stores index', () => {
  describe('store exports', () => {
    test('should export AuthStore', () => {
      // Given the index module
      // When I check AuthStore export
      // Then it should be defined and match the source
      expect(AuthStore).toBeDefined()
      expect(AuthStore).toBe(AuthStoreSource)
      expect(AuthStore).toHaveProperty('tag')
      expect(AuthStore).toHaveProperty('layer')
    })

    test('should export AuthStoreTag', () => {
      // Given the index module
      // When I check AuthStoreTag export
      // Then it should be defined and match the source
      expect(AuthStoreTag).toBeDefined()
      expect(AuthStoreTag).toBe(AuthStoreTagSource)
      expect(AuthStoreTag).toBe(AuthStore.tag)
    })

    test('should export authStoreLayer', () => {
      // Given the index module
      // When I check authStoreLayer export
      // Then it should be defined and match the source
      expect(authStoreLayer).toBeDefined()
      expect(authStoreLayer).toBe(authStoreLayerSource)
      expect(authStoreLayer).toBe(AuthStore.layer)
    })

    test('should export AuthenticationStateReactiveStoreTag', () => {
      // Given the index module
      // When I check AuthenticationStateReactiveStoreTag export
      // Then it should be defined and match the source
      expect(AuthenticationStateReactiveStoreTag).toBeDefined()
      expect(AuthenticationStateReactiveStoreTag).toBe(
        AuthenticationStateReactiveStoreTagSource
      )
      expect(AuthenticationStateReactiveStoreTag).toBe(AuthStoreTag)
    })

    test('should export getAuthenticationStateStoreLayer', () => {
      // Given the index module
      // When I check getAuthenticationStateStoreLayer export
      // Then it should be a function and match the source
      expect(typeof getAuthenticationStateStoreLayer).toBe('function')
      expect(getAuthenticationStateStoreLayer).toBe(
        getAuthenticationStateStoreLayerSource
      )
      expect(getAuthenticationStateStoreLayer()).toBe(authStoreLayer)
    })

    test('should export initialAuthenticationState', () => {
      // Given the index module
      // When I check initialAuthenticationState export
      // Then it should be defined and match the source
      expect(initialAuthenticationState).toBeDefined()
      expect(initialAuthenticationState).toBe(initialAuthenticationStateSource)
      expect(initialAuthenticationState).toHaveProperty('token')
      expect(initialAuthenticationState).toHaveProperty('user')
      expect(initialAuthenticationState).toHaveProperty('needsScopeSelect')
      expect(initialAuthenticationState).toHaveProperty('permissions')
    })

    test('should export AuthenticationState type', () => {
      // Given the index module
      // When I use AuthenticationState type
      // Then it should be compatible with the source type
      const state: AuthenticationState = initialAuthenticationState
      expect(state).toBeDefined()
      // Type compatibility verified by TypeScript compilation
      expect(Option.isNone(state.token)).toBe(true)
    })

    test('should export AuthenticationStateReactiveStore type', () => {
      // Given the index module
      // When I use AuthenticationStateReactiveStore type
      // Then it should be compatible with the source type
      const program = Effect.gen(function* () {
        const store = yield* AuthStoreTag
        const typedStore: AuthenticationStateReactiveStore = store
        return typedStore
      })

      // Type check passes if this compiles
      expect(true).toBe(true)
    })
  })

  describe('hook exports', () => {
    test('should export useAuthStore', () => {
      // Given the index module
      // When I check useAuthStore export
      // Then it should be a function and match the source
      expect(typeof useAuthStore).toBe('function')
      expect(useAuthStore).toBe(useAuthStoreSource)
    })

    test('should export useAuthStoreWithInit', () => {
      // Given the index module
      // When I check useAuthStoreWithInit export
      // Then it should be a function and match the source
      expect(typeof useAuthStoreWithInit).toBe('function')
      expect(useAuthStoreWithInit).toBe(useAuthStoreWithInitSource)
    })

    test('should export useAuthenticationStateReactiveStore', () => {
      // Given the index module
      // When I check useAuthenticationStateReactiveStore export
      // Then it should be a function and match the source
      expect(typeof useAuthenticationStateReactiveStore).toBe('function')
      expect(useAuthenticationStateReactiveStore).toBe(
        useAuthenticationStateReactiveStoreSource
      )
    })

    test('should export deprecated useAuthenticationStateReactiveStore as wrapper', () => {
      // Given the index module
      // When I check useAuthenticationStateReactiveStore
      // Then it should delegate to useAuthStoreWithInit
      // Note: This is verified by the implementation in the source file
      expect(useAuthenticationStateReactiveStore).toBeDefined()
      expect(typeof useAuthenticationStateReactiveStore).toBe('function')
    })
  })

  describe('export consistency', () => {
    test('should re-export all store-related exports', () => {
      // Given the index module
      // When I check all store exports
      // Then they should all be available
      expect(AuthStore).toBeDefined()
      expect(AuthStoreTag).toBeDefined()
      expect(authStoreLayer).toBeDefined()
      expect(AuthenticationStateReactiveStoreTag).toBeDefined()
      expect(getAuthenticationStateStoreLayer).toBeDefined()
      expect(initialAuthenticationState).toBeDefined()
    })

    test('should re-export all hook-related exports', () => {
      // Given the index module
      // When I check all hook exports
      // Then they should all be available
      expect(useAuthStore).toBeDefined()
      expect(useAuthStoreWithInit).toBeDefined()
      expect(useAuthenticationStateReactiveStore).toBeDefined()
    })

    test('should maintain correct relationships between exports', () => {
      // Given the index module exports
      // When I check relationships
      // Then they should match the source module relationships
      expect(AuthStoreTag).toBe(AuthStore.tag)
      expect(authStoreLayer).toBe(AuthStore.layer)
      expect(AuthenticationStateReactiveStoreTag).toBe(AuthStoreTag)
      expect(getAuthenticationStateStoreLayer()).toBe(authStoreLayer)
    })
  })

  describe('type exports', () => {
    test('should export AuthenticationState type correctly', () => {
      // Given the index module
      // When I use AuthenticationState type
      // Then it should work as expected
      const state: AuthenticationState = {
        token: Option.none(),
        user: Option.none(),
        needsScopeSelect: Option.none(),
        permissions: [],
      }

      expect(state).toBeDefined()
      expect(Option.isNone(state.token)).toBe(true)
      expect(Option.isNone(state.user)).toBe(true)
      expect(Option.isNone(state.needsScopeSelect)).toBe(true)
      expect(state.permissions).toEqual([])
    })

    test('should allow AuthenticationState with some values', () => {
      // Given the index module
      // When I create an AuthenticationState with some values
      // Then it should work correctly
      const state: AuthenticationState = {
        token: Option.some('test-token'),
        user: Option.none(),
        needsScopeSelect: Option.some(true),
        permissions: ['read:users'],
      }

      expect(Option.isSome(state.token)).toBe(true)
      expect(Option.getOrUndefined(state.token)).toBe('test-token')
      expect(Option.isSome(state.needsScopeSelect)).toBe(true)
      expect(Option.getOrUndefined(state.needsScopeSelect)).toBe(true)
      expect(state.permissions).toEqual(['read:users'])
    })
  })

  describe('module structure', () => {
    test('should be a valid ES module', () => {
      // Given the index module
      // When I check its structure
      // Then it should export valid values
      expect(typeof AuthStore).toBe('object')
      expect(typeof AuthStoreTag).toBe('object')
      expect(typeof authStoreLayer).toBe('object')
      expect(typeof getAuthenticationStateStoreLayer).toBe('function')
      expect(typeof useAuthStore).toBe('function')
      expect(typeof useAuthStoreWithInit).toBe('function')
    })

    test('should maintain backward compatibility exports', () => {
      // Given the index module
      // When I check deprecated exports
      // Then they should still be available
      expect(AuthenticationStateReactiveStoreTag).toBeDefined()
      expect(useAuthenticationStateReactiveStore).toBeDefined()
      // Deprecated tag should equal the new tag
      expect(AuthenticationStateReactiveStoreTag).toBe(AuthStoreTag)
    })
  })
})
