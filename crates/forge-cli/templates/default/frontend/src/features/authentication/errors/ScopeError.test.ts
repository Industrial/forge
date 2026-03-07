/**
 * BDD tests for ScopeError
 * Tests verify the behavior of the ScopeError Data.TaggedError
 */

import { describe, it, expect } from 'bun:test'
import { ScopeError } from './ScopeError'

describe('ScopeError', () => {
  describe('instance creation behavior', () => {
    it('should create an instance with a message', () => {
      // Given an error message
      const message = 'Scope selection failed'

      // When I create a ScopeError instance
      const error = new ScopeError({ message })

      // Then it should have the message property
      expect(error.message).toBe(message)
    })

    it('should create an instance with message and cause', () => {
      // Given an error message and cause
      const message = 'Network error'
      const cause = new Error('Connection timeout')

      // When I create a ScopeError instance with cause
      const error = new ScopeError({ message, cause })

      // Then it should have both message and cause
      expect(error.message).toBe(message)
      expect(error.cause).toBe(cause)
    })

    it('should create an instance without cause', () => {
      // Given an error message without cause
      const message = 'Invalid organization'

      // When I create a ScopeError instance without cause
      const error = new ScopeError({ message })

      // Then cause should be undefined
      expect(error.message).toBe(message)
      expect(error.cause).toBeUndefined()
    })

    it('should accept empty string as message', () => {
      // Given an empty message
      const message = ''

      // When I create a ScopeError instance
      const error = new ScopeError({ message })

      // Then it should accept empty string
      expect(error.message).toBe('')
    })

    it('should have readonly properties', () => {
      // Given a ScopeError instance
      const error = new ScopeError({
        message: 'Test error',
      })

      // When I try to modify properties (TypeScript should prevent this)
      // Then the properties should remain unchanged
      // Note: In runtime, readonly doesn't prevent modification, but TypeScript enforces it
      expect(error.message).toBe('Test error')
    })
  })

  describe('structural equality behavior', () => {
    it('should be equal to another instance with same message and no cause', () => {
      // Given two ScopeError instances with the same message
      const error1 = new ScopeError({ message: 'Scope failed' })
      const error2 = new ScopeError({ message: 'Scope failed' })

      // When I compare them
      // Then they should be structurally equal (Data.TaggedError provides this)
      expect(error1).toEqual(error2)
    })

    it('should be equal when both have undefined cause', () => {
      // Given two ScopeError instances without cause
      const error1 = new ScopeError({ message: 'Error' })
      const error2 = new ScopeError({ message: 'Error' })

      // When I compare them
      // Then they should be equal
      expect(error1).toEqual(error2)
    })

    it('should not be equal when message differs', () => {
      // Given two ScopeError instances with different messages
      const error1 = new ScopeError({ message: 'Error 1' })
      const error2 = new ScopeError({ message: 'Error 2' })

      // When I compare them
      // Then they should not be equal
      expect(error1).not.toEqual(error2)
    })

    it('should not be equal when cause differs', () => {
      // Given two ScopeError instances with different causes
      const error1 = new ScopeError({
        message: 'Error',
        cause: new Error('Cause 1'),
      })
      const error2 = new ScopeError({
        message: 'Error',
        cause: new Error('Cause 2'),
      })

      // When I compare them
      // Then they should not be equal
      expect(error1).not.toEqual(error2)
    })

    it('should not be equal when one has cause and the other does not', () => {
      // Given two ScopeError instances where one has cause and the other does not
      const error1 = new ScopeError({
        message: 'Error',
        cause: new Error('Cause'),
      })
      const error2 = new ScopeError({ message: 'Error' })

      // When I compare them
      // Then they should not be equal
      expect(error1).not.toEqual(error2)
    })

    it('should be equal when both have the same cause object reference', () => {
      // Given a shared cause object
      const cause = new Error('Shared cause')
      const error1 = new ScopeError({ message: 'Error', cause })
      const error2 = new ScopeError({ message: 'Error', cause })

      // When I compare them
      // Then they should be equal (same object reference)
      expect(error1).toEqual(error2)
    })
  })

  describe('tagged error behavior', () => {
    it('should have the ScopeError tag', () => {
      // Given a ScopeError instance
      const error = new ScopeError({ message: 'Test error' })

      // When I check the tag
      // Then it should be tagged as 'ScopeError'
      // Data.TaggedError provides a _tag property
      expect((error as any)._tag).toBe('ScopeError')
    })

    it('should be identifiable as ScopeError type', () => {
      // Given a ScopeError instance
      const error = new ScopeError({ message: 'Test error' })

      // When I check the instance
      // Then it should be an instance of ScopeError
      expect(error).toBeInstanceOf(ScopeError)
    })

    it('should be an Error instance', () => {
      // Given a ScopeError instance
      const error = new ScopeError({ message: 'Test error' })

      // When I check if it's an Error
      // Then it should be an instance of Error
      expect(error).toBeInstanceOf(Error)
    })
  })

  describe('property access behavior', () => {
    it('should allow reading message property', () => {
      // Given a ScopeError instance
      const error = new ScopeError({ message: 'Read test' })

      // When I read the message property
      const message = error.message

      // Then it should return the correct value
      expect(message).toBe('Read test')
    })

    it('should allow reading cause property when present', () => {
      // Given a ScopeError instance with cause
      const cause = new Error('Original error')
      const error = new ScopeError({ message: 'Wrapper', cause })

      // When I read the cause property
      const errorCause = error.cause

      // Then it should return the correct value
      expect(errorCause).toBe(cause)
    })

    it('should return undefined for cause when not provided', () => {
      // Given a ScopeError instance without cause
      const error = new ScopeError({ message: 'No cause' })

      // When I read the cause property
      const cause = error.cause

      // Then it should be undefined
      expect(cause).toBeUndefined()
    })
  })

  describe('error message behavior', () => {
    it('should have message property accessible', () => {
      // Given a ScopeError instance
      const error = new ScopeError({ message: 'Custom message' })

      // When I access the message
      // Then it should be accessible
      expect(error.message).toBe('Custom message')
    })

    it('should work with common scope error messages', () => {
      // Given common scope error messages
      const messages = [
        'Invalid organization',
        'Invalid role',
        'Scope selection failed',
        'Organization not found',
        'Role not found',
      ]

      // When I create ScopeError instances with these messages
      const errors = messages.map((msg) => new ScopeError({ message: msg }))

      // Then each should have the correct message
      errors.forEach((error, index) => {
        expect(error.message).toBe(messages[index])
      })
    })
  })

  describe('cause chaining behavior', () => {
    it('should allow chaining errors with cause', () => {
      // Given a nested error structure
      const originalError = new Error('Original error')
      const scopeError = new ScopeError({
        message: 'Scope selection failed',
        cause: originalError,
      })

      // When I access the cause
      // Then it should contain the original error
      expect(scopeError.cause).toBe(originalError)
    })

    it('should allow cause to be any unknown type', () => {
      // Given various cause types
      const stringCause = 'String error'
      const numberCause = 404
      const objectCause = { code: 'SCOPE_FAILED', details: 'Invalid' }

      // When I create ScopeError instances with these causes
      const error1 = new ScopeError({
        message: 'Error 1',
        cause: stringCause,
      })
      const error2 = new ScopeError({
        message: 'Error 2',
        cause: numberCause,
      })
      const error3 = new ScopeError({
        message: 'Error 3',
        cause: objectCause,
      })

      // Then each should store the cause correctly
      expect(error1.cause).toBe(stringCause)
      expect(error2.cause).toBe(numberCause)
      expect(error3.cause).toBe(objectCause)
    })
  })

  describe('usage in scope selection flow', () => {
    it('should represent invalid organization errors', () => {
      // Given an invalid organization scenario
      const scopeError = new ScopeError({
        message: 'Organization not found or access denied',
      })

      // When I use it in scope selection flow
      // Then it should represent the organization error
      expect(scopeError.message).toContain('Organization')
    })

    it('should represent invalid role errors', () => {
      // Given an invalid role scenario
      const scopeError = new ScopeError({
        message: 'Role not found or access denied',
      })

      // When I use it in scope selection flow
      // Then it should represent the role error
      expect(scopeError.message).toContain('Role')
    })

    it('should represent network errors during scope selection', () => {
      // Given a network error scenario
      const networkError = new Error('Failed to fetch')
      const scopeError = new ScopeError({
        message: 'Unable to select scope: network error',
        cause: networkError,
      })

      // When I use it in scope selection flow
      // Then it should represent the network error
      expect(scopeError.message).toContain('scope')
      expect(scopeError.cause).toBe(networkError)
    })

    it('should represent API errors during scope selection', () => {
      // Given an API error scenario
      const apiError = { status: 403, message: 'Forbidden' }
      const scopeError = new ScopeError({
        message: 'Scope selection failed',
        cause: apiError,
      })

      // When I use it in scope selection flow
      // Then it should represent the API error
      expect(scopeError.message).toContain('failed')
      expect(scopeError.cause).toBe(apiError)
    })
  })
})
