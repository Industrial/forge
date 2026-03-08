import { describe, test, expect } from 'bun:test'
import { RegistrationFailedError } from './RegistrationFailedError'

describe('RegistrationFailedError', () => {
  describe('error creation behavior', () => {
    test('should create error with message', () => {
      const message = 'Email already exists'
      const error = new RegistrationFailedError({ message })

      expect(error.message).toBe(message)
      expect(error.cause).toBeUndefined()
    })

    test('should create error with message and cause', () => {
      const message = 'Registration request failed'
      const cause = new Error('Network timeout')
      const error = new RegistrationFailedError({ message, cause })

      expect(error.message).toBe(message)
      expect(error.cause).toBe(cause)
    })

    test('should create error without cause', () => {
      const message = 'Invalid registration data'
      const error = new RegistrationFailedError({ message })

      expect(error.message).toBe(message)
      expect(error.cause).toBeUndefined()
    })
  })

  describe('tagged error behavior', () => {
    test('should be tagged with RegistrationFailedError', () => {
      const error = new RegistrationFailedError({ message: 'Test error' })

      expect(error).toBeInstanceOf(RegistrationFailedError)
      expect((error as any)._tag).toBe('RegistrationFailedError')
    })

    test('should support Effect.catchTag pattern matching', () => {
      const error = new RegistrationFailedError({ message: 'Catchable error' })

      expect((error as any)._tag).toBe('RegistrationFailedError')
      expect(typeof error.message).toBe('string')
    })
  })

  describe('readonly fields behavior', () => {
    test('should have readonly message field', () => {
      const error = new RegistrationFailedError({ message: 'Test error' })

      expect(error.message).toBe('Test error')
    })

    test('should have readonly cause field', () => {
      const error = new RegistrationFailedError({
        message: 'Test error',
        cause: new Error('Cause'),
      })

      expect(error.cause).toBeDefined()
    })
  })

  describe('registration failure scenarios', () => {
    test('should represent email already exists error', () => {
      const error = new RegistrationFailedError({
        message: 'Email already registered',
      })

      expect(error.message).toContain('Email')
    })

    test('should represent validation error during registration', () => {
      const error = new RegistrationFailedError({
        message: 'Password must be at least 8 characters',
      })

      expect(error.message).toContain('Password')
    })

    test('should represent network error during registration', () => {
      const networkError = new Error('Connection failed')
      const error = new RegistrationFailedError({
        message: 'Registration request failed due to network error',
        cause: networkError,
      })

      expect(error.message).toContain('network')
      expect(error.cause).toBe(networkError)
    })

    test('should represent server error during registration', () => {
      const serverError = { status: 500, message: 'Internal server error' }
      const error = new RegistrationFailedError({
        message: 'Registration failed: server error',
        cause: serverError,
      })

      expect(error.message).toContain('server')
      expect(error.cause).toBe(serverError)
    })
  })

  describe('cause types behavior', () => {
    test('should contain Error object as cause', () => {
      const cause = new Error('Underlying error')
      const error = new RegistrationFailedError({
        message: 'Registration failed',
        cause,
      })

      expect(error.cause).toBe(cause)
      expect(error.cause).toBeInstanceOf(Error)
    })

    test('should contain string as cause', () => {
      const cause = 'Network timeout'
      const error = new RegistrationFailedError({
        message: 'Request failed',
        cause,
      })

      expect(error.cause).toBe(cause)
      expect(typeof error.cause).toBe('string')
    })

    test('should contain object as cause', () => {
      const cause = { code: 409, details: 'Conflict' }
      const error = new RegistrationFailedError({
        message: 'Email already exists',
        cause,
      })

      expect(error.cause).toBe(cause)
      expect(typeof error.cause).toBe('object')
    })
  })
})
