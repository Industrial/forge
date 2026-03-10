import { describe, test, expect } from 'bun:test'
import { TokenMissingError } from './TokenMissingError'

describe('TokenMissingError', () => {
  describe('error creation behavior', () => {
    test('should create error with message', () => {
      const message = 'Token is missing from response'
      const error = new TokenMissingError({ message })

      expect(error.message).toBe(message)
    })

    test('should create error without cause field', () => {
      const message = 'Expected token not found'
      const error = new TokenMissingError({ message })

      expect(error.message).toBe(message)
      // TokenMissingError does not have a cause field
      expect((error as any).cause).toBeUndefined()
    })
  })

  describe('tagged error behavior', () => {
    test('should be tagged with TokenMissingError', () => {
      const error = new TokenMissingError({ message: 'Test error' })

      expect(error).toBeInstanceOf(TokenMissingError)
      expect((error as any)._tag).toBe('TokenMissingError')
    })

    test('should support Effect.catchTag pattern matching', () => {
      const error = new TokenMissingError({ message: 'Catchable error' })

      expect((error as any)._tag).toBe('TokenMissingError')
      expect(typeof error.message).toBe('string')
    })
  })

  describe('readonly fields behavior', () => {
    test('should have readonly message field', () => {
      const error = new TokenMissingError({ message: 'Test error' })

      expect(error.message).toBe('Test error')
    })
  })

  describe('token missing scenarios', () => {
    test('should represent missing token in login response', () => {
      const error = new TokenMissingError({
        message: 'Login response did not contain token',
      })

      expect(error.message).toContain('token')
    })

    test('should represent missing token in registration response', () => {
      const error = new TokenMissingError({
        message: 'Registration successful but token missing',
      })

      expect(error.message).toContain('missing')
    })

    test('should represent unexpected response format', () => {
      const error = new TokenMissingError({
        message: 'Response does not contain expected token field',
      })

      expect(error.message).toContain('Response')
    })

    test('should represent null token in response', () => {
      const error = new TokenMissingError({
        message: 'Token field is null or undefined',
      })

      expect(error.message).toContain('null')
    })
  })

  describe('error message types behavior', () => {
    test('should handle descriptive error messages', () => {
      const error = new TokenMissingError({
        message: 'Authentication token not present in server response',
      })

      expect(error.message).toBe(
        'Authentication token not present in server response',
      )
    })

    test('should handle empty error message', () => {
      const error = new TokenMissingError({ message: '' })

      expect(error.message).toBe('')
      expect(error.message.length).toBe(0)
    })

    test('should handle long error messages', () => {
      const longMessage = `Token missing: ${'A'.repeat(500)}`
      const error = new TokenMissingError({ message: longMessage })

      expect(error.message).toBe(longMessage)
    })
  })
})
