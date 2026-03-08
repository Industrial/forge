import { describe, test, expect } from 'bun:test'
import { ScopeSelectionFailedError } from './ScopeSelectionFailedError'

describe('ScopeSelectionFailedError', () => {
  describe('error creation behavior', () => {
    test('should create error with message', () => {
      const message = 'Scope selection failed'
      const error = new ScopeSelectionFailedError({ message })

      expect(error.message).toBe(message)
      expect(error.cause).toBeUndefined()
    })

    test('should create error with message and cause', () => {
      const message = 'Failed to select organization'
      const cause = new Error('Network timeout')
      const error = new ScopeSelectionFailedError({ message, cause })

      expect(error.message).toBe(message)
      expect(error.cause).toBe(cause)
    })

    test('should create error without cause', () => {
      const message = 'Invalid scope configuration'
      const error = new ScopeSelectionFailedError({ message })

      expect(error.message).toBe(message)
      expect(error.cause).toBeUndefined()
    })
  })

  describe('tagged error behavior', () => {
    test('should be tagged with ScopeSelectionFailedError', () => {
      const error = new ScopeSelectionFailedError({ message: 'Test error' })

      expect(error).toBeInstanceOf(ScopeSelectionFailedError)
      expect((error as any)._tag).toBe('ScopeSelectionFailedError')
    })

    test('should support Effect.catchTag pattern matching', () => {
      const error = new ScopeSelectionFailedError({
        message: 'Catchable error',
      })

      expect((error as any)._tag).toBe('ScopeSelectionFailedError')
      expect(typeof error.message).toBe('string')
    })
  })

  describe('readonly fields behavior', () => {
    test('should have readonly message field', () => {
      const error = new ScopeSelectionFailedError({ message: 'Test error' })

      expect(error.message).toBe('Test error')
    })

    test('should have readonly cause field', () => {
      const error = new ScopeSelectionFailedError({
        message: 'Test error',
        cause: new Error('Cause'),
      })

      expect(error.cause).toBeDefined()
    })
  })

  describe('scope selection failure scenarios', () => {
    test('should represent invalid organization error', () => {
      const error = new ScopeSelectionFailedError({
        message: 'Organization not found',
      })

      expect(error.message).toContain('Organization')
    })

    test('should represent invalid role error', () => {
      const error = new ScopeSelectionFailedError({
        message: 'Role not available for this organization',
      })

      expect(error.message).toContain('Role')
    })

    test('should represent API error during scope selection', () => {
      const apiError = { status: 403, message: 'Forbidden' }
      const error = new ScopeSelectionFailedError({
        message: 'Access denied to selected scope',
        cause: apiError,
      })

      expect(error.message).toContain('Access')
      expect(error.cause).toBe(apiError)
    })

    test('should represent network error during scope selection', () => {
      const networkError = new Error('Connection failed')
      const error = new ScopeSelectionFailedError({
        message: 'Unable to select scope due to network error',
        cause: networkError,
      })

      expect(error.message).toContain('network')
      expect(error.cause).toBe(networkError)
    })
  })

  describe('cause types behavior', () => {
    test('should contain Error object as cause', () => {
      const cause = new Error('Underlying error')
      const error = new ScopeSelectionFailedError({
        message: 'Scope selection failed',
        cause,
      })

      expect(error.cause).toBe(cause)
      expect(error.cause).toBeInstanceOf(Error)
    })

    test('should contain string as cause', () => {
      const cause = 'Network timeout'
      const error = new ScopeSelectionFailedError({
        message: 'Request failed',
        cause,
      })

      expect(error.cause).toBe(cause)
      expect(typeof error.cause).toBe('string')
    })

    test('should contain object as cause', () => {
      const cause = { code: 403, details: 'Access denied' }
      const error = new ScopeSelectionFailedError({
        message: 'Forbidden',
        cause,
      })

      expect(error.cause).toBe(cause)
      expect(typeof error.cause).toBe('object')
    })
  })
})
