/**
 * BDD tests for authentication errors index
 * Tests verify that error exports from the index file work correctly
 */

import { describe, it, expect } from 'bun:test'
import { ScopeError } from './index'
import { ScopeError as ScopeErrorDirect } from './ScopeError'

describe('authentication errors index', () => {
  describe('export behavior', () => {
    it('should export ScopeError from index', () => {
      // Given the errors index module
      // When I import ScopeError from the index
      // Then it should be available and be the same as the direct import
      expect(ScopeError).toBe(ScopeErrorDirect)
    })

    it('should export ScopeError as a class', () => {
      // Given the errors index module
      // When I check the exported ScopeError
      // Then it should be a class/constructor function
      expect(typeof ScopeError).toBe('function')
    })

    it('should allow creating ScopeError instances from index export', () => {
      // Given the ScopeError exported from index
      // When I create an instance
      const error = new ScopeError({ message: 'Test error' })

      // Then it should be a valid ScopeError instance
      expect(error).toBeInstanceOf(ScopeError)
      expect(error.message).toBe('Test error')
    })

    it('should export ScopeError with correct tag', () => {
      // Given a ScopeError instance created from index export
      const error = new ScopeError({ message: 'Test' })

      // When I check the tag
      // Then it should have the correct tag
      expect((error as any)._tag).toBe('ScopeError')
    })
  })

  describe('re-export behavior', () => {
    it('should re-export ScopeError correctly', () => {
      // Given both direct and index imports
      const errorFromIndex = new ScopeError({ message: 'Test' })
      const errorFromDirect = new ScopeErrorDirect({ message: 'Test' })

      // When I compare them
      // Then they should be structurally equal
      expect(errorFromIndex).toEqual(errorFromDirect)
      expect(errorFromIndex).toBeInstanceOf(ScopeErrorDirect)
      expect(errorFromDirect).toBeInstanceOf(ScopeError)
    })

    it('should maintain ScopeError functionality through index export', () => {
      // Given a ScopeError created from index export
      const error = new ScopeError({
        message: 'Scope selection failed',
        cause: new Error('Original error'),
      })

      // When I access properties
      // Then all functionality should work correctly
      expect(error.message).toBe('Scope selection failed')
      expect(error.cause).toBeInstanceOf(Error)
      expect((error.cause as Error).message).toBe('Original error')
    })
  })

  describe('module structure behavior', () => {
    it('should provide a clean export interface', () => {
      // Given the errors index module
      // When I check what is exported
      // Then it should export ScopeError
      // Note: This test verifies the module structure
      expect(ScopeError).toBeDefined()
    })

    it('should allow importing ScopeError with named import', () => {
      // Given the errors index module
      // When I import ScopeError as a named export
      // Then it should work correctly
      const testError = new ScopeError({ message: 'Named import test' })
      expect(testError.message).toBe('Named import test')
    })
  })
})
