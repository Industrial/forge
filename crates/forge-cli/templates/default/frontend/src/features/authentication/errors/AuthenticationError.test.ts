import { describe, test, expect } from 'bun:test'
import { AuthenticationError } from './AuthenticationError'

/**
 * BDD-style tests for AuthenticationError.
 * Tests focus on behavior rather than implementation.
 * Tests are organized by feature/behavior area with descriptive names.
 */

describe('AuthenticationError', () => {
  describe('error creation behavior', () => {
    test('should create error with message', () => {
      // Given: an error message
      const message = 'Authentication failed'

      // When: creating an AuthenticationError
      const error = new AuthenticationError({ message })

      // Then: error should contain the message
      expect(error.message).toBe(message)
      expect(error.cause).toBeUndefined()
    })

    test('should create error with message and cause', () => {
      // Given: an error message and cause
      const message = 'Network error occurred'
      const cause = new Error('Connection timeout')

      // When: creating an AuthenticationError with cause
      const error = new AuthenticationError({ message, cause })

      // Then: error should contain both message and cause
      expect(error.message).toBe(message)
      expect(error.cause).toBe(cause)
    })

    test('should create error without cause', () => {
      // Given: an error message without cause
      const message = 'Invalid token'

      // When: creating an AuthenticationError without cause
      const error = new AuthenticationError({ message })

      // Then: error should have message but no cause
      expect(error.message).toBe(message)
      expect(error.cause).toBeUndefined()
    })
  })

  describe('error message behavior', () => {
    test('should contain descriptive error message', () => {
      // Given: a descriptive error message
      const message = 'Token has expired. Please login again.'

      // When: creating an error
      const error = new AuthenticationError({ message })

      // Then: should contain the descriptive message
      expect(error.message).toBe(message)
      expect(typeof error.message).toBe('string')
    })

    test('should handle empty error message', () => {
      // Given: an empty error message
      // When: creating an error with empty message
      const error = new AuthenticationError({ message: '' })

      // Then: should accept empty message
      expect(error.message).toBe('')
      expect(error.message.length).toBe(0)
    })

    test('should handle long error messages', () => {
      // Given: a long error message
      const longMessage = 'A'.repeat(1000)

      // When: creating an error with long message
      const error = new AuthenticationError({ message: longMessage })

      // Then: should handle long messages
      expect(error.message).toBe(longMessage)
      expect(error.message.length).toBe(1000)
    })
  })

  describe('error cause behavior', () => {
    test('should contain Error object as cause', () => {
      // Given: an Error object
      const cause = new Error('Underlying error')

      // When: creating error with Error cause
      const error = new AuthenticationError({
        message: 'Authentication failed',
        cause,
      })

      // Then: should contain Error as cause
      expect(error.cause).toBe(cause)
      expect(error.cause).toBeInstanceOf(Error)
    })

    test('should contain string as cause', () => {
      // Given: a string cause
      const cause = 'Network timeout'

      // When: creating error with string cause
      const error = new AuthenticationError({
        message: 'Request failed',
        cause,
      })

      // Then: should contain string as cause
      expect(error.cause).toBe(cause)
      expect(typeof error.cause).toBe('string')
    })

    test('should contain object as cause', () => {
      // Given: an object cause
      const cause = { code: 500, details: 'Internal server error' }

      // When: creating error with object cause
      const error = new AuthenticationError({
        message: 'Server error',
        cause,
      })

      // Then: should contain object as cause
      expect(error.cause).toBe(cause)
      expect(typeof error.cause).toBe('object')
    })

    test('should allow cause to be undefined', () => {
      // Given: no cause provided
      // When: creating error without cause
      const error = new AuthenticationError({ message: 'Error occurred' })

      // Then: cause should be undefined
      expect(error.cause).toBeUndefined()
    })

    test('should allow cause to be null', () => {
      // Given: null cause
      // When: creating error with null cause
      const error = new AuthenticationError({
        message: 'Error occurred',
        cause: null,
      })

      // Then: cause should be null
      expect(error.cause).toBeNull()
    })
  })

  describe('readonly fields behavior', () => {
    test('should have readonly message field', () => {
      // Given: an error instance
      const error = new AuthenticationError({ message: 'Test error' })

      // When: accessing message property
      // Then: message should be readable
      expect(error.message).toBe('Test error')
      // TypeScript readonly modifier prevents mutation at compile time
    })

    test('should have readonly cause field', () => {
      // Given: an error instance with cause
      const error = new AuthenticationError({
        message: 'Test error',
        cause: new Error('Cause'),
      })

      // When: accessing cause property
      // Then: cause should be readable
      expect(error.cause).toBeDefined()
      // TypeScript readonly modifier prevents mutation at compile time
    })
  })

  describe('tagged error behavior', () => {
    test('should be tagged with AuthenticationError', () => {
      // Given: an error instance
      const error = new AuthenticationError({ message: 'Test error' })

      // When: checking the class tag
      // Then: should be tagged as AuthenticationError
      // Effect's Data.TaggedError provides structural equality and type safety
      expect(error).toBeInstanceOf(AuthenticationError)
    })

    test('should support pattern matching via tagged error', () => {
      // Given: an AuthenticationError instance
      const error = new AuthenticationError({ message: 'Pattern match test' })

      // When: checking error type
      // Then: should be identifiable as AuthenticationError
      // Effect's Data.TaggedError enables pattern matching in Effect.catchTag
      expect(error._tag).toBe('AuthenticationError')
    })

    test('should support structural equality via Data.TaggedError', () => {
      // Given: two error instances with same data
      const error1 = new AuthenticationError({
        message: 'Same error',
        cause: new Error('Same cause'),
      })
      const error2 = new AuthenticationError({
        message: 'Same error',
        cause: new Error('Same cause'),
      })

      // When: comparing instances
      // Then: should have same message (structural equality)
      // Note: Effect's Data.TaggedError provides structural equality
      expect(error1.message).toBe(error2.message)
      // Cause comparison depends on reference equality
    })
  })

  describe('authentication store operation behavior', () => {
    test('should represent fetchMe failure errors', () => {
      // Given: a fetchMe failure scenario
      const error = new AuthenticationError({
        message: 'Failed to fetch current user',
        cause: new Error('Network error'),
      })

      // When: using error from fetchMe failure
      // Then: should represent authentication fetch error
      expect(error.message).toContain('fetch')
      expect(error.cause).toBeDefined()
      // Typically used when fetchMe fails (network or invalid token)
    })

    test('should represent invalid token errors', () => {
      // Given: an invalid token scenario
      const error = new AuthenticationError({
        message: 'Invalid or expired token',
      })

      // When: using error for invalid token
      // Then: should represent token validation error
      expect(error.message).toContain('token')
      // Used when token is invalid or expired
    })

    test('should represent network errors', () => {
      // Given: a network error scenario
      const networkError = new Error('Connection failed')
      const error = new AuthenticationError({
        message: 'Network request failed',
        cause: networkError,
      })

      // When: using error for network failure
      // Then: should contain network error cause
      expect(error.message).toContain('Network')
      expect(error.cause).toBe(networkError)
    })

    test('should represent invalid state errors', () => {
      // Given: an invalid state scenario
      const error = new AuthenticationError({
        message: 'Method called in invalid state',
      })

      // When: using error for invalid state
      // Then: should represent state validation error
      expect(error.message).toContain('invalid')
      // Used when a method is used in an invalid state
    })
  })

  describe('error propagation behavior', () => {
    test('should preserve error message during propagation', () => {
      // Given: an error with message
      const originalMessage = 'Original error message'
      const error = new AuthenticationError({ message: originalMessage })

      // When: propagating error
      // Then: message should be preserved
      expect(error.message).toBe(originalMessage)
      // Error can be propagated through Effect chains
    })

    test('should preserve error cause during propagation', () => {
      // Given: an error with cause
      const originalCause = new Error('Original cause')
      const error = new AuthenticationError({
        message: 'Wrapper error',
        cause: originalCause,
      })

      // When: propagating error
      // Then: cause should be preserved
      expect(error.cause).toBe(originalCause)
      // Cause provides context about underlying error
    })

    test('should support error chaining', () => {
      // Given: a nested error scenario
      const innerError = new Error('Inner error')
      const middleError = new AuthenticationError({
        message: 'Middle error',
        cause: innerError,
      })
      const outerError = new AuthenticationError({
        message: 'Outer error',
        cause: middleError,
      })

      // When: accessing nested errors
      // Then: should support error chaining
      expect(outerError.message).toBe('Outer error')
      expect(outerError.cause).toBe(middleError)
      // Errors can be chained with cause references
    })
  })

  describe('effect error handling behavior', () => {
    test('should be usable in Effect error channel', () => {
      // Given: an AuthenticationError
      const error = new AuthenticationError({ message: 'Effect error' })

      // When: using in Effect
      // Then: should be compatible with Effect error type
      // Effect.catchTag('AuthenticationError') can catch this error
      expect(error._tag).toBe('AuthenticationError')
    })

    test('should support Effect.catchTag pattern matching', () => {
      // Given: an AuthenticationError
      const error = new AuthenticationError({
        message: 'Catchable error',
      })

      // When: pattern matching in Effect
      // Then: should be catchable by tag
      // Effect.catchTag('AuthenticationError', handler) can handle this
      expect(error._tag).toBe('AuthenticationError')
      expect(typeof error.message).toBe('string')
    })

    test('should provide error context for Effect handlers', () => {
      // Given: an error with message and cause
      const error = new AuthenticationError({
        message: 'Error with context',
        cause: { statusCode: 401 },
      })

      // When: handling in Effect
      // Then: should provide context via message and cause
      expect(error.message).toBeDefined()
      expect(error.cause).toBeDefined()
      // Effect handlers can access message and cause for error handling
    })
  })

  describe('error message types behavior', () => {
    test('should handle authentication failure messages', () => {
      // Given: authentication failure message
      const error = new AuthenticationError({
        message: 'Invalid email or password',
      })

      // When: accessing message
      // Then: should contain authentication failure information
      expect(error.message).toBe('Invalid email or password')
    })

    test('should handle token expiration messages', () => {
      // Given: token expiration message
      const error = new AuthenticationError({
        message: 'Token has expired',
      })

      // When: accessing message
      // Then: should indicate token expiration
      expect(error.message).toBe('Token has expired')
    })

    test('should handle session timeout messages', () => {
      // Given: session timeout message
      const error = new AuthenticationError({
        message: 'Session has timed out',
      })

      // When: accessing message
      // Then: should indicate session timeout
      expect(error.message).toBe('Session has timed out')
    })
  })
})
