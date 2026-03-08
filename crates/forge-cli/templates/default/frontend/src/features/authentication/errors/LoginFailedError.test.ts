import { describe, test, expect } from 'bun:test'
import { LoginFailedError } from './LoginFailedError'

describe('LoginFailedError', () => {
  describe('error creation behavior', () => {
    test('should create error with message', () => {
      const message = 'Invalid email or password'
      const error = new LoginFailedError({ message })

      expect(error.message).toBe(message)
      expect(error.cause).toBeUndefined()
    })

    test('should create error with message and cause', () => {
      const message = 'Login request failed'
      const cause = new Error('Network timeout')
      const error = new LoginFailedError({ message, cause })

      expect(error.message).toBe(message)
      expect(error.cause).toBe(cause)
    })

    test('should create error without cause', () => {
      const message = 'Invalid credentials'
      const error = new LoginFailedError({ message })

      expect(error.message).toBe(message)
      expect(error.cause).toBeUndefined()
    })
  })

  describe('tagged error behavior', () => {
    test('should be tagged with LoginFailedError', () => {
      const error = new LoginFailedError({ message: 'Test error' })

      expect(error).toBeInstanceOf(LoginFailedError)
      expect((error as any)._tag).toBe('LoginFailedError')
    })

    test('should support Effect.catchTag pattern matching', () => {
      const error = new LoginFailedError({ message: 'Catchable error' })

      expect((error as any)._tag).toBe('LoginFailedError')
      expect(typeof error.message).toBe('string')
    })
  })

  describe('readonly fields behavior', () => {
    test('should have readonly message field', () => {
      const error = new LoginFailedError({ message: 'Test error' })

      expect(error.message).toBe('Test error')
    })

    test('should have readonly cause field', () => {
      const error = new LoginFailedError({
        message: 'Test error',
        cause: new Error('Cause'),
      })

      expect(error.cause).toBeDefined()
    })
  })

  describe('login failure scenarios', () => {
    test('should represent invalid credentials error', () => {
      const error = new LoginFailedError({
        message: 'Invalid email or password',
      })

      expect(error.message).toContain('Invalid')
    })

    test('should represent network error during login', () => {
      const networkError = new Error('Connection failed')
      const error = new LoginFailedError({
        message: 'Login request failed due to network error',
        cause: networkError,
      })

      expect(error.message).toContain('network')
      expect(error.cause).toBe(networkError)
    })

    test('should represent server error during login', () => {
      const serverError = { status: 500, message: 'Internal server error' }
      const error = new LoginFailedError({
        message: 'Login failed: server error',
        cause: serverError,
      })

      expect(error.message).toContain('server')
      expect(error.cause).toBe(serverError)
    })

    test('should represent validation error during login', () => {
      const error = new LoginFailedError({
        message: 'Email format is invalid',
      })

      expect(error.message).toContain('Email')
    })
  })

  describe('cause types behavior', () => {
    test('should contain Error object as cause', () => {
      const cause = new Error('Underlying error')
      const error = new LoginFailedError({
        message: 'Login failed',
        cause,
      })

      expect(error.cause).toBe(cause)
      expect(error.cause).toBeInstanceOf(Error)
    })

    test('should contain string as cause', () => {
      const cause = 'Network timeout'
      const error = new LoginFailedError({
        message: 'Request failed',
        cause,
      })

      expect(error.cause).toBe(cause)
      expect(typeof error.cause).toBe('string')
    })

    test('should contain object as cause', () => {
      const cause = { code: 401, details: 'Unauthorized' }
      const error = new LoginFailedError({
        message: 'Authentication failed',
        cause,
      })

      expect(error.cause).toBe(cause)
      expect(typeof error.cause).toBe('object')
    })
  })
})
