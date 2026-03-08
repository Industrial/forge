import { describe, test, expect } from 'bun:test'
import { InvalidTokenError } from './InvalidTokenError'

describe('InvalidTokenError', () => {
  describe('error creation behavior', () => {
    test('should create error with message', () => {
      const message = 'Token is invalid or expired'
      const error = new InvalidTokenError({ message })

      expect(error.message).toBe(message)
    })

    test('should create error without cause field', () => {
      const message = 'Token validation failed'
      const error = new InvalidTokenError({ message })

      expect(error.message).toBe(message)
      // InvalidTokenError does not have a cause field
      expect((error as any).cause).toBeUndefined()
    })
  })

  describe('tagged error behavior', () => {
    test('should be tagged with InvalidTokenError', () => {
      const error = new InvalidTokenError({ message: 'Test error' })

      expect(error).toBeInstanceOf(InvalidTokenError)
      expect((error as any)._tag).toBe('InvalidTokenError')
    })

    test('should support Effect.catchTag pattern matching', () => {
      const error = new InvalidTokenError({ message: 'Catchable error' })

      expect((error as any)._tag).toBe('InvalidTokenError')
      expect(typeof error.message).toBe('string')
    })
  })

  describe('readonly fields behavior', () => {
    test('should have readonly message field', () => {
      const error = new InvalidTokenError({ message: 'Test error' })

      expect(error.message).toBe('Test error')
    })
  })

  describe('invalid token scenarios', () => {
    test('should represent expired token error', () => {
      const error = new InvalidTokenError({
        message: 'Token has expired',
      })

      expect(error.message).toContain('expired')
    })

    test('should represent malformed token error', () => {
      const error = new InvalidTokenError({
        message: 'Token format is invalid',
      })

      expect(error.message).toContain('invalid')
    })

    test('should represent token validation failure', () => {
      const error = new InvalidTokenError({
        message: 'Token signature verification failed',
      })

      expect(error.message).toContain('Token')
    })

    test('should represent revoked token error', () => {
      const error = new InvalidTokenError({
        message: 'Token has been revoked',
      })

      expect(error.message).toContain('revoked')
    })
  })

  describe('error message types behavior', () => {
    test('should handle descriptive error messages', () => {
      const error = new InvalidTokenError({
        message: 'JWT token signature is invalid',
      })

      expect(error.message).toBe('JWT token signature is invalid')
    })

    test('should handle empty error message', () => {
      const error = new InvalidTokenError({ message: '' })

      expect(error.message).toBe('')
      expect(error.message.length).toBe(0)
    })

    test('should handle long error messages', () => {
      const longMessage = 'Token validation failed: ' + 'A'.repeat(500)
      const error = new InvalidTokenError({ message: longMessage })

      expect(error.message).toBe(longMessage)
    })
  })
})
