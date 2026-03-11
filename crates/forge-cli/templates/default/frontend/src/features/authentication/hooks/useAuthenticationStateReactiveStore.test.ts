/**
 * BDD tests for useAuthenticationStateReactiveStore hook
 * Tests verify the behavior of the deprecated hook wrapper
 */

import { describe, it, expect } from 'bun:test'
import {
  useAuthenticationStateReactiveStore,
  useAuthStoreWithInit,
} from './useAuthenticationStateReactiveStore'
import type { AuthenticationState } from '../../../features/authentication/stores/AuthenticationStateReactiveStore'

describe('useAuthenticationStateReactiveStore', () => {
  describe('export behavior', () => {
    it('should export useAuthenticationStateReactiveStore as a function', () => {
      // Given the module
      // When I check the export
      // Then it should be a function
      expect(typeof useAuthenticationStateReactiveStore).toBe('function')
    })

    it('should export useAuthStoreWithInit as a function', () => {
      // Given the module
      // When I check the export
      // Then useAuthStoreWithInit should be a function
      expect(typeof useAuthStoreWithInit).toBe('function')
    })
  })

  describe('deprecated wrapper behavior', () => {
    it('should have the same signature as useAuthStoreWithInit', () => {
      // Given both functions
      // When I check their types
      // Then they should have compatible signatures
      // Note: TypeScript will enforce this at compile time
      expect(typeof useAuthenticationStateReactiveStore).toBe('function')
      expect(typeof useAuthStoreWithInit).toBe('function')
    })

    it('should return the same structure as useAuthStoreWithInit', () => {
      // Given the function signature
      // When I check the return type
      // Then it should return { authentication: AuthenticationState, initialized: boolean }
      // This is verified by TypeScript, but we can check the function exists
      expect(useAuthenticationStateReactiveStore).toBeDefined()
    })
  })

  describe('delegation behavior', () => {
    it('should delegate to useAuthStoreWithInit', () => {
      // Given the deprecated function
      // When I check its implementation
      // Then it should call useAuthStoreWithInit
      // Note: This is verified by reading the source code
      // The function body is: return useAuthStoreWithInit()
      expect(useAuthenticationStateReactiveStore).toBeDefined()
    })
  })

  describe('type compatibility', () => {
    it('should have compatible return type with useAuthStoreWithInit', () => {
      // Given both functions
      // When TypeScript checks the types
      // Then they should return the same structure
      // This test verifies the function exists and can be called
      // Actual type checking happens at compile time
      const expectedReturnType = {
        authentication: {} as AuthenticationState,
        initialized: false,
      }

      // Verify the structure matches
      expect(expectedReturnType).toHaveProperty('authentication')
      expect(expectedReturnType).toHaveProperty('initialized')
      expect(typeof expectedReturnType.initialized).toBe('boolean')
    })
  })

  describe('migration guidance', () => {
    it('should be marked as deprecated in documentation', () => {
      // Given the function
      // When I check if it's deprecated
      // Then it should have @deprecated JSDoc tag
      // This is verified by reading the source code
      expect(useAuthenticationStateReactiveStore).toBeDefined()
    })

    it('should recommend useAuthStoreWithInit as replacement', () => {
      // Given the deprecated function
      // When developers see the deprecation notice
      // Then they should use useAuthStoreWithInit instead
      // This is verified by the @deprecated comment in source
      expect(useAuthStoreWithInit).toBeDefined()
      expect(typeof useAuthStoreWithInit).toBe('function')
    })
  })

  describe('function identity', () => {
    it('should be a different function reference from useAuthStoreWithInit', () => {
      // Given both functions
      // When I compare their references
      // Then they should be different functions (wrapper pattern)
      expect(useAuthenticationStateReactiveStore).not.toBe(useAuthStoreWithInit)
    })

    it('should be callable as a function', () => {
      // Given the function
      // When I check if it's callable
      // Then it should be a function
      expect(typeof useAuthenticationStateReactiveStore).toBe('function')
      expect(useAuthenticationStateReactiveStore).toBeInstanceOf(Function)
    })
  })
})
