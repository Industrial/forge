/**
 * Unit tests for AuthenticationLive helper functions.
 * Following Effect.ts testing patterns - no vi.mock() for Effect services.
 *
 * These tests focus on the pure helper functions and core business logic.
 * Integration tests for the full service should use AuthenticationMock.ts.
 */

import { Option } from 'effect'
import { describe, expect, it } from 'vitest'
import type { AuthMeBody } from '../../../api/types'
import {
  buildAuthState,
  extractApiErrorMessage,
  getPermissions,
  parseMeResponse,
  selectSingleScope,
} from './AuthenticationLive'

// ============================================================================
// Pure Helper Functions Tests
// ============================================================================

describe('Pure Helper Functions', () => {
  describe('parseMeResponse', () => {
    it('should parse valid user data with token', () => {
      const token = 'test-token-123'
      const body: AuthMeBody = {
        user: {
          id: 'user-1',
          email: 'test@example.com',
        },
      }

      const result = parseMeResponse(body, token)
      expect(Option.isSome(result)).toBe(true)
      const user = Option.getOrThrow(result)
      expect(user.id).toBe('user-1')
      expect(user.email).toBe('test@example.com')
      expect(user.token).toBe('test-token-123')
    })

    it('should return None for empty body', () => {
      const result = parseMeResponse({}, 'token')
      expect(Option.isNone(result)).toBe(true)
    })

    it('should return None for body without user', () => {
      const body: AuthMeBody = { someOtherField: 'value' } as any
      const result = parseMeResponse(body, 'token')
      expect(Option.isNone(result)).toBe(true)
    })

    it('should handle null user', () => {
      const body: AuthMeBody = { user: null as any }
      const result = parseMeResponse(body, 'token')
      expect(Option.isNone(result)).toBe(true)
    })

    it('should convert missing fields to empty strings', () => {
      const body: AuthMeBody = {
        user: {
          id: undefined as any,
          email: undefined as any,
        },
      }
      const result = parseMeResponse(body, 'token')
      expect(Option.isSome(result)).toBe(true)
      const user = Option.getOrThrow(result)
      expect(user.id).toBe('')
      expect(user.email).toBe('')
    })
  })

  describe('getPermissions', () => {
    it('should get permissions from top-level', () => {
      const body: AuthMeBody = { permissions: ['read', 'write', 'admin'] }
      const result = getPermissions(body)
      expect(result).toEqual(['read', 'write', 'admin'])
    })

    it('should get permissions from user.permissions', () => {
      const body: AuthMeBody = {
        user: {
          id: 'user-1',
          email: 'test@example.com',
          permissions: ['read', 'write'],
        },
      }
      const result = getPermissions(body)
      expect(result).toEqual(['read', 'write'])
    })

    it('should return empty array if no permissions', () => {
      const body: AuthMeBody = {}
      const result = getPermissions(body)
      expect(result).toEqual([])
    })

    it('should prefer top-level permissions over user.permissions', () => {
      const body: AuthMeBody = {
        permissions: ['admin'],
        user: {
          id: 'user-1',
          email: 'test@example.com',
          permissions: ['read'],
        },
      }
      const result = getPermissions(body)
      expect(result).toEqual(['admin'])
    })

    it('should filter out non-string values', () => {
      const body: AuthMeBody = {
        permissions: ['read', 123, null, undefined, 'write'] as any,
      }
      const result = getPermissions(body)
      expect(result).toEqual(['read', 'write'])
    })

    it('should handle non-array permissions', () => {
      const body: AuthMeBody = {
        permissions: 'admin' as any,
      }
      const result = getPermissions(body)
      expect(result).toEqual([])
    })
  })

  describe('extractApiErrorMessage', () => {
    it('should extract error message from message field', () => {
      const body = { message: 'Something went wrong' }
      const result = extractApiErrorMessage(body, 'Unknown error')
      expect(result).toBe('Something went wrong')
    })

    it('should extract error from string error field', () => {
      const body = { error: 'Invalid credentials' }
      const result = extractApiErrorMessage(body, 'Unknown error')
      expect(result).toBe('Invalid credentials')
    })

    it('should extract error from object error field', () => {
      const body = { error: { message: 'Nested error' } }
      const result = extractApiErrorMessage(body, 'Unknown error')
      expect(result).toBe('Nested error')
    })

    it('should return fallback if no error fields', () => {
      const body = {}
      const result = extractApiErrorMessage(body, 'Unknown error')
      expect(result).toBe('Unknown error')
    })

    it('should prefer message over error', () => {
      const body = { message: 'Error 1', error: 'Error 2' }
      const result = extractApiErrorMessage(body, 'Unknown error')
      expect(result).toBe('Error 1')
    })
  })

  describe('buildAuthState', () => {
    it('should build complete auth state with all fields', () => {
      const token = 'test-token-123'
      const body: AuthMeBody = {
        user: {
          id: 'user-1',
          email: 'test@example.com',
        },
        permissions: ['read', 'write'],
      }

      const result = buildAuthState(token, body)

      expect(Option.isSome(result.token)).toBe(true)
      expect(Option.getOrThrow(result.token)).toBe('test-token-123')
      expect(Option.isSome(result.user)).toBe(true)
      const user = Option.getOrThrow(result.user)
      expect(user.id).toBe('user-1')
      expect(user.email).toBe('test@example.com')
      expect(result.permissions).toEqual(['read', 'write'])
      expect(Option.isNone(result.currentScope)).toBe(true)
    })

    it('should set currentScope when provided', () => {
      const token = 'test-token-123'
      const body: AuthMeBody = {
        user: {
          id: 'user-1',
          email: 'test@example.com',
        },
      }

      const result = buildAuthState(token, body, {
        organizationId: 'org-1',
        roleId: 'role-1',
      })

      expect(Option.isSome(result.currentScope)).toBe(true)
      expect(Option.getOrThrow(result.currentScope)).toEqual({
        organizationId: 'org-1',
        roleId: 'role-1',
      })
    })

    it('should handle missing user', () => {
      const token = 'test-token-123'
      const body: AuthMeBody = {
        permissions: ['read'],
      }

      const result = buildAuthState(token, body)

      expect(Option.isSome(result.token)).toBe(true)
      expect(Option.isNone(result.user)).toBe(true)
      expect(result.permissions).toEqual(['read'])
      expect(Option.isNone(result.currentScope)).toBe(true)
    })
  })

  describe('selectSingleScope', () => {
    it('should return Some with single scope', () => {
      const scopes = [{ org_id: 'org-1', role_id: 'role-1' }]
      const result = selectSingleScope(scopes)
      expect(Option.isSome(result)).toBe(true)
      const scope = Option.getOrThrow(result)
      expect(scope.organizationId).toBe('org-1')
      expect(scope.roleId).toBe('role-1')
    })

    it('should return None for empty array', () => {
      const scopes: Array<{ org_id: string; role_id?: string }> = []
      const result = selectSingleScope(scopes)
      expect(Option.isNone(result)).toBe(true)
    })

    it('should return None for multiple scopes', () => {
      const scopes = [
        { org_id: 'org-1', role_id: 'role-1' },
        { org_id: 'org-2', role_id: 'role-2' },
      ]
      const result = selectSingleScope(scopes)
      expect(Option.isNone(result)).toBe(true)
    })

    it('should return None if org_id is missing', () => {
      const scopes = [{ org_id: '', role_id: 'role-1' }]
      const result = selectSingleScope(scopes)
      expect(Option.isNone(result)).toBe(true)
    })

    it('should return None if role_id is missing', () => {
      const scopes = [{ org_id: 'org-1', role_id: undefined }]
      const result = selectSingleScope(scopes)
      expect(Option.isNone(result)).toBe(true)
    })

    it('should handle role_id as empty string', () => {
      const scopes = [{ org_id: 'org-1', role_id: '' }]
      const result = selectSingleScope(scopes)
      expect(Option.isNone(result)).toBe(true)
    })
  })
})
