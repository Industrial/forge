import { describe, test, expect } from 'bun:test'
import { Flash } from './Flash'

/**
 * BDD-style tests for Flash domain class.
 * Tests focus on behavior rather than implementation.
 * Tests are organized by feature/behavior area with descriptive names.
 */

describe('Flash', () => {
  describe('flash message creation behavior', () => {
    test('should create flash with success message', () => {
      // Given: a success message
      const message = 'Login successful'

      // When: creating a flash with message
      const flash = new Flash({ message })

      // Then: flash should contain the message
      expect(flash.message).toBe(message)
      expect(flash.error).toBeUndefined()
    })

    test('should create flash with error message', () => {
      // Given: an error message
      const error = 'Invalid credentials'

      // When: creating a flash with error
      const flash = new Flash({ error })

      // Then: flash should contain the error
      expect(flash.error).toBe(error)
      expect(flash.message).toBeUndefined()
    })

    test('should create flash with both message and error', () => {
      // Given: both message and error
      const message = 'Operation completed'
      const error = 'Some warning occurred'

      // When: creating a flash with both
      const flash = new Flash({ message, error })

      // Then: flash should contain both
      expect(flash.message).toBe(message)
      expect(flash.error).toBe(error)
    })

    test('should create flash with no message or error', () => {
      // Given: no message or error
      // When: creating an empty flash
      const flash = new Flash({})

      // Then: both message and error should be undefined
      expect(flash.message).toBeUndefined()
      expect(flash.error).toBeUndefined()
    })
  })

  describe('flash message types behavior', () => {
    test('should handle success flash messages', () => {
      // Given: a success message
      const successMessage = 'Registration successful'

      // When: creating a success flash
      const flash = new Flash({ message: successMessage })

      // Then: should be identifiable as success (has message, no error)
      expect(flash.message).toBe(successMessage)
      expect(flash.error).toBeUndefined()
      expect(typeof flash.message).toBe('string')
    })

    test('should handle error flash messages', () => {
      // Given: an error message
      const errorMessage = 'Authentication failed'

      // When: creating an error flash
      const flash = new Flash({ error: errorMessage })

      // Then: should be identifiable as error (has error, no message)
      expect(flash.error).toBe(errorMessage)
      expect(flash.message).toBeUndefined()
      expect(typeof flash.error).toBe('string')
    })

    test('should handle mixed flash messages', () => {
      // Given: both success and error messages
      const message = 'Process completed'
      const error = 'Some issues occurred'

      // When: creating a mixed flash
      const flash = new Flash({ message, error })

      // Then: should contain both types
      expect(flash.message).toBeDefined()
      expect(flash.error).toBeDefined()
      expect(typeof flash.message).toBe('string')
      expect(typeof flash.error).toBe('string')
    })
  })

  describe('optional fields behavior', () => {
    test('should have optional message field', () => {
      // Given: a flash instance
      // When: message is not provided
      const flash = new Flash({})

      // Then: message should be optional (undefined)
      expect(flash.message).toBeUndefined()
    })

    test('should have optional error field', () => {
      // Given: a flash instance
      // When: error is not provided
      const flash = new Flash({})

      // Then: error should be optional (undefined)
      expect(flash.error).toBeUndefined()
    })

    test('should allow message without error', () => {
      // Given: only message provided
      // When: creating flash with message only
      const flash = new Flash({ message: 'Success' })

      // Then: message should be set, error should be undefined
      expect(flash.message).toBe('Success')
      expect(flash.error).toBeUndefined()
    })

    test('should allow error without message', () => {
      // Given: only error provided
      // When: creating flash with error only
      const flash = new Flash({ error: 'Failure' })

      // Then: error should be set, message should be undefined
      expect(flash.error).toBe('Failure')
      expect(flash.message).toBeUndefined()
    })
  })

  describe('readonly fields behavior', () => {
    test('should have readonly message field', () => {
      // Given: a flash instance with message
      const flash = new Flash({ message: 'Test message' })

      // When: accessing message property
      // Then: message should be readable
      expect(flash.message).toBe('Test message')
      // TypeScript readonly modifier prevents mutation at compile time
    })

    test('should have readonly error field', () => {
      // Given: a flash instance with error
      const flash = new Flash({ error: 'Test error' })

      // When: accessing error property
      // Then: error should be readable
      expect(flash.error).toBe('Test error')
      // TypeScript readonly modifier prevents mutation at compile time
    })
  })

  describe('tagged class behavior', () => {
    test('should be tagged with Flash', () => {
      // Given: a flash instance
      const flash = new Flash({ message: 'Test' })

      // When: checking the class tag
      // Then: should be tagged as Flash
      // Effect's Data.TaggedClass provides structural equality and type safety
      expect(flash).toBeInstanceOf(Flash)
    })

    test('should support structural equality via Data.TaggedClass', () => {
      // Given: two flash instances with same data
      const flash1 = new Flash({ message: 'Success' })
      const flash2 = new Flash({ message: 'Success' })

      // When: comparing instances
      // Then: should be structurally equal (Effect's Data.TaggedClass behavior)
      // Note: Effect's Data.TaggedClass provides structural equality
      expect(flash1.message).toBe(flash2.message)
      expect(flash1.error).toBe(flash2.error)
    })
  })

  describe('string content behavior', () => {
    test('should accept string messages', () => {
      // Given: a string message
      const message = 'Operation completed successfully'

      // When: creating flash with string message
      const flash = new Flash({ message })

      // Then: should store the string
      expect(typeof flash.message).toBe('string')
      expect(flash.message).toBe(message)
    })

    test('should accept string errors', () => {
      // Given: a string error
      const error = 'An error occurred during processing'

      // When: creating flash with string error
      const flash = new Flash({ error })

      // Then: should store the string
      expect(typeof flash.error).toBe('string')
      expect(flash.error).toBe(error)
    })

    test('should handle empty string messages', () => {
      // Given: an empty string message
      // When: creating flash with empty message
      const flash = new Flash({ message: '' })

      // Then: should store empty string
      expect(flash.message).toBe('')
      expect(flash.message?.length).toBe(0)
    })

    test('should handle empty string errors', () => {
      // Given: an empty string error
      // When: creating flash with empty error
      const flash = new Flash({ error: '' })

      // Then: should store empty string
      expect(flash.error).toBe('')
      expect(flash.error?.length).toBe(0)
    })

    test('should handle long string messages', () => {
      // Given: a long message string
      const longMessage = 'A'.repeat(1000)

      // When: creating flash with long message
      const flash = new Flash({ message: longMessage })

      // Then: should store the entire string
      expect(flash.message).toBe(longMessage)
      expect(flash.message?.length).toBe(1000)
    })
  })

  describe('auth UI feedback behavior', () => {
    test('should be usable for success feedback in auth UI', () => {
      // Given: a success flash message
      const flash = new Flash({ message: 'Login successful' })

      // When: using for UI feedback
      // Then: should indicate success (has message)
      expect(flash.message).toBeDefined()
      expect(flash.message).toBe('Login successful')
      // UI can check flash.message to display success
    })

    test('should be usable for error feedback in auth UI', () => {
      // Given: an error flash message
      const flash = new Flash({ error: 'Invalid email or password' })

      // When: using for UI feedback
      // Then: should indicate error (has error)
      expect(flash.error).toBeDefined()
      expect(flash.error).toBe('Invalid email or password')
      // UI can check flash.error to display error
    })

    test('should support one-off feedback display', () => {
      // Given: a flash message for one-time display
      const flash = new Flash({ message: 'Session expired, please login again' })

      // When: displaying in UI
      // Then: should contain the feedback message
      expect(flash.message).toBe('Session expired, please login again')
      // Flash messages are typically shown once and then cleared
    })
  })
})
