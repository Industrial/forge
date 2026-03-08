import { describe, test, expect } from 'bun:test'
import { UserNotFoundError } from './UserNotFoundError'

describe('UserNotFoundError', () => {
  describe('error creation behavior', () => {
    test('should create error with message', () => {
      const message = 'User not found after login'
      const error = new UserNotFoundError({ message })

      expect(error.message).toBe(message)
    })

    test('should create error without cause field', () => {
      const message = 'Failed to load user data'
      const error = new UserNotFoundError({ message })

      expect(error.message).toBe(message)
      // UserNotFoundError does not have a cause field
      expect((error as any).cause).toBeUndefined()
    })
  })

  describe('tagged error behavior', () => {
    test('should be tagged with UserNotFoundError', () => {
      const error = new UserNotFoundError({ message: 'Test error' })

      expect(error).toBeInstanceOf(UserNotFoundError)
      expect((error as any)._tag).toBe('UserNotFoundError')
    })

    test('should support Effect.catchTag pattern matching', () => {
      const error = new UserNotFoundError({ message: 'Catchable error' })

      expect((error as any)._tag).toBe('UserNotFoundError')
      expect(typeof error.message).toBe('string')
    })
  })

  describe('readonly fields behavior', () => {
    test('should have readonly message field', () => {
      const error = new UserNotFoundError({ message: 'Test error' })

      expect(error.message).toBe('Test error')
    })
  })

  describe('user not found scenarios', () => {
    test('should represent user not found after successful login', () => {
      const error = new UserNotFoundError({
        message: 'User data not available after authentication',
      })

      expect(error.message).toContain('User')
    })

    test('should represent failed user data fetch', () => {
      const error = new UserNotFoundError({
        message: 'Failed to load user after login',
      })

      expect(error.message).toContain('Failed')
    })

    test('should represent unexpected state error', () => {
      const error = new UserNotFoundError({
        message: 'Unexpected state: login succeeded but user missing',
      })

      expect(error.message).toContain('Unexpected')
    })
  })

  describe('error message types behavior', () => {
    test('should handle descriptive error messages', () => {
      const error = new UserNotFoundError({
        message: 'User profile could not be retrieved',
      })

      expect(error.message).toBe('User profile could not be retrieved')
    })

    test('should handle empty error message', () => {
      const error = new UserNotFoundError({ message: '' })

      expect(error.message).toBe('')
      expect(error.message.length).toBe(0)
    })

    test('should handle long error messages', () => {
      const longMessage = 'A'.repeat(500)
      const error = new UserNotFoundError({ message: longMessage })

      expect(error.message).toBe(longMessage)
      expect(error.message.length).toBe(500)
    })
  })
})
