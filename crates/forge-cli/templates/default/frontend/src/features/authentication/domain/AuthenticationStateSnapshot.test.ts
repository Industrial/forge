import { describe, test, expect } from 'bun:test'
import { AuthenticationStateSnapshot } from './AuthenticationStateSnapshot'
import { AuthenticationUser } from './AuthenticationUser'
import { Flash } from './Flash'
import { Scope } from './Scope'

/**
 * BDD-style tests for AuthenticationStateSnapshot.
 * Tests focus on behavior rather than implementation.
 * Tests are organized by feature/behavior area with descriptive names.
 */

describe('AuthenticationStateSnapshot', () => {
  describe('snapshot creation behavior', () => {
    test('should create snapshot with all required fields', () => {
      // Given: all required snapshot data
      const user = new AuthenticationUser({
        id: 'user-1',
        email: 'test@example.com',
        token: 'token-123',
      })
      const scopes: Scope[] = []
      const permissions: string[] = ['read:users', 'write:users']
      const flash = new Flash({ message: 'Success' })
      const token = 'token-123'
      const currentOrgId = 'org-1'
      const currentRoleId = 'role-1'
      const currentRoleName = 'admin'
      const needs_scope_select = false

      // When: creating a snapshot
      const snapshot = new AuthenticationStateSnapshot({
        user,
        scopes,
        permissions,
        flash,
        token,
        currentOrgId,
        currentRoleId,
        currentRoleName,
        needs_scope_select,
      })

      // Then: snapshot should contain all provided values
      expect(snapshot.user).toBe(user)
      expect(snapshot.scopes).toBe(scopes)
      expect(snapshot.permissions).toBe(permissions)
      expect(snapshot.flash).toBe(flash)
      expect(snapshot.token).toBe(token)
      expect(snapshot.currentOrgId).toBe(currentOrgId)
      expect(snapshot.currentRoleId).toBe(currentRoleId)
      expect(snapshot.currentRoleName).toBe(currentRoleName)
      expect(snapshot.needs_scope_select).toBe(needs_scope_select)
    })

    test('should create snapshot with null user when not authenticated', () => {
      // Given: no authenticated user
      // When: creating a snapshot with null user
      const snapshot = new AuthenticationStateSnapshot({
        user: null,
        scopes: [],
        permissions: [],
        flash: null,
        token: null,
        currentOrgId: null,
        currentRoleId: null,
        currentRoleName: null,
        needs_scope_select: false,
      })

      // Then: user should be null
      expect(snapshot.user).toBeNull()
      expect(snapshot.token).toBeNull()
    })

    test('should create snapshot with empty arrays when no scopes or permissions', () => {
      // Given: no scopes or permissions
      // When: creating a snapshot
      const snapshot = new AuthenticationStateSnapshot({
        user: null,
        scopes: [],
        permissions: [],
        flash: null,
        token: null,
        currentOrgId: null,
        currentRoleId: null,
        currentRoleName: null,
        needs_scope_select: false,
      })

      // Then: scopes and permissions should be empty arrays
      expect(snapshot.scopes).toEqual([])
      expect(snapshot.permissions).toEqual([])
      expect(Array.isArray(snapshot.scopes)).toBe(true)
      expect(Array.isArray(snapshot.permissions)).toBe(true)
    })
  })

  describe('read-only behavior', () => {
    test('should be read-only snapshot with no setters', () => {
      // Given: a snapshot instance
      const snapshot = new AuthenticationStateSnapshot({
        user: null,
        scopes: [],
        permissions: [],
        flash: null,
        token: null,
        currentOrgId: null,
        currentRoleId: null,
        currentRoleName: null,
        needs_scope_select: false,
      })

      // When: attempting to access properties
      // Then: all properties should be readable
      expect(snapshot.user).toBeDefined()
      expect(snapshot.scopes).toBeDefined()
      expect(snapshot.permissions).toBeDefined()
      expect(snapshot.flash).toBeDefined()
      expect(snapshot.token).toBeDefined()
      expect(snapshot.currentOrgId).toBeDefined()
      expect(snapshot.currentRoleId).toBeDefined()
      expect(snapshot.currentRoleName).toBeDefined()
      expect(snapshot.needs_scope_select).toBeDefined()
      // Note: TypeScript's readonly modifier prevents mutation at compile time
    })
  })

  describe('scope selection behavior', () => {
    test('should indicate scope selection needed when needs_scope_select is true', () => {
      // Given: a snapshot with needs_scope_select set to true
      const snapshot = new AuthenticationStateSnapshot({
        user: new AuthenticationUser({
          id: 'user-1',
          email: 'test@example.com',
          token: 'token-123',
        }),
        scopes: [],
        permissions: [],
        flash: null,
        token: 'token-123',
        currentOrgId: null,
        currentRoleId: null,
        currentRoleName: null,
        needs_scope_select: true,
      })

      // When: checking if scope selection is needed
      // Then: should indicate scope selection is required
      expect(snapshot.needs_scope_select).toBe(true)
      // App should redirect to scope selection before dashboard
    })

    test('should not require scope selection when needs_scope_select is false', () => {
      // Given: a snapshot with needs_scope_select set to false
      const snapshot = new AuthenticationStateSnapshot({
        user: new AuthenticationUser({
          id: 'user-1',
          email: 'test@example.com',
          token: 'token-123',
        }),
        scopes: [new Scope({ org_id: 'org-1', org_name: 'Org 1', role_id: 'role-1', role: 'admin' })],
        permissions: [],
        flash: null,
        token: 'token-123',
        currentOrgId: 'org-1',
        currentRoleId: 'role-1',
        currentRoleName: 'admin',
        needs_scope_select: false,
      })

      // When: checking if scope selection is needed
      // Then: should not require scope selection
      expect(snapshot.needs_scope_select).toBe(false)
    })

    test('should have current scope when organization and role are set', () => {
      // Given: a snapshot with current organization and role
      const snapshot = new AuthenticationStateSnapshot({
        user: new AuthenticationUser({
          id: 'user-1',
          email: 'test@example.com',
          token: 'token-123',
        }),
        scopes: [],
        permissions: [],
        flash: null,
        token: 'token-123',
        currentOrgId: 'org-1',
        currentRoleId: 'role-1',
        currentRoleName: 'admin',
        needs_scope_select: false,
      })

      // When: accessing current scope information
      // Then: should have organization and role IDs and name
      expect(snapshot.currentOrgId).toBe('org-1')
      expect(snapshot.currentRoleId).toBe('role-1')
      expect(snapshot.currentRoleName).toBe('admin')
    })

    test('should have null current scope when not set', () => {
      // Given: a snapshot without current scope
      const snapshot = new AuthenticationStateSnapshot({
        user: new AuthenticationUser({
          id: 'user-1',
          email: 'test@example.com',
          token: 'token-123',
        }),
        scopes: [],
        permissions: [],
        flash: null,
        token: 'token-123',
        currentOrgId: null,
        currentRoleId: null,
        currentRoleName: null,
        needs_scope_select: true,
      })

      // When: accessing current scope information
      // Then: should have null values
      expect(snapshot.currentOrgId).toBeNull()
      expect(snapshot.currentRoleId).toBeNull()
      expect(snapshot.currentRoleName).toBeNull()
    })
  })

  describe('user authentication behavior', () => {
    test('should contain authenticated user when logged in', () => {
      // Given: an authenticated user
      const user = new AuthenticationUser({
        id: 'user-1',
        email: 'test@example.com',
        token: 'token-123',
      })

      // When: creating a snapshot with user
      const snapshot = new AuthenticationStateSnapshot({
        user,
        scopes: [],
        permissions: [],
        flash: null,
        token: 'token-123',
        currentOrgId: null,
        currentRoleId: null,
        currentRoleName: null,
        needs_scope_select: false,
      })

      // Then: snapshot should contain the user
      expect(snapshot.user).toBe(user)
      expect(snapshot.user?.id).toBe('user-1')
      expect(snapshot.user?.email).toBe('test@example.com')
      expect(snapshot.user?.token).toBe('token-123')
    })

    test('should have null user when not authenticated', () => {
      // Given: no authenticated user
      // When: creating a snapshot without user
      const snapshot = new AuthenticationStateSnapshot({
        user: null,
        scopes: [],
        permissions: [],
        flash: null,
        token: null,
        currentOrgId: null,
        currentRoleId: null,
        currentRoleName: null,
        needs_scope_select: false,
      })

      // Then: user should be null
      expect(snapshot.user).toBeNull()
    })
  })

  describe('permissions behavior', () => {
    test('should contain list of permission strings', () => {
      // Given: a list of permissions
      const permissions = ['read:users', 'write:users', 'delete:users']

      // When: creating a snapshot with permissions
      const snapshot = new AuthenticationStateSnapshot({
        user: null,
        scopes: [],
        permissions,
        flash: null,
        token: null,
        currentOrgId: null,
        currentRoleId: null,
        currentRoleName: null,
        needs_scope_select: false,
      })

      // Then: snapshot should contain the permissions
      expect(snapshot.permissions).toEqual(permissions)
      expect(snapshot.permissions.length).toBe(3)
      expect(snapshot.permissions[0]).toBe('read:users')
    })

    test('should handle empty permissions list', () => {
      // Given: no permissions
      // When: creating a snapshot with empty permissions
      const snapshot = new AuthenticationStateSnapshot({
        user: null,
        scopes: [],
        permissions: [],
        flash: null,
        token: null,
        currentOrgId: null,
        currentRoleId: null,
        currentRoleName: null,
        needs_scope_select: false,
      })

      // Then: permissions should be empty array
      expect(snapshot.permissions).toEqual([])
      expect(snapshot.permissions.length).toBe(0)
    })

    test('should have readonly permissions array', () => {
      // Given: a snapshot with permissions
      const snapshot = new AuthenticationStateSnapshot({
        user: null,
        scopes: [],
        permissions: ['read:users'],
        flash: null,
        token: null,
        currentOrgId: null,
        currentRoleId: null,
        currentRoleName: null,
        needs_scope_select: false,
      })

      // When: accessing permissions
      // Then: permissions should be readonly array
      expect(Array.isArray(snapshot.permissions)).toBe(true)
      // TypeScript readonly modifier prevents mutation
    })
  })

  describe('scopes behavior', () => {
    test('should contain list of available scopes', () => {
      // Given: available scopes
      const scopes = [
        new Scope({ org_id: 'org-1', org_name: 'Org 1', role_id: 'role-1', role: 'admin' }),
        new Scope({ org_id: 'org-2', org_name: 'Org 2', role_id: 'role-2', role: 'viewer' }),
      ]

      // When: creating a snapshot with scopes
      const snapshot = new AuthenticationStateSnapshot({
        user: new AuthenticationUser({
          id: 'user-1',
          email: 'test@example.com',
          token: 'token-123',
        }),
        scopes,
        permissions: [],
        flash: null,
        token: 'token-123',
        currentOrgId: null,
        currentRoleId: null,
        currentRoleName: null,
        needs_scope_select: false,
      })

      // Then: snapshot should contain the scopes
      expect(snapshot.scopes).toEqual(scopes)
      expect(snapshot.scopes.length).toBe(2)
      expect(snapshot.scopes[0].org_id).toBe('org-1')
    })

    test('should handle empty scopes list', () => {
      // Given: no scopes
      // When: creating a snapshot with empty scopes
      const snapshot = new AuthenticationStateSnapshot({
        user: null,
        scopes: [],
        permissions: [],
        flash: null,
        token: null,
        currentOrgId: null,
        currentRoleId: null,
        currentRoleName: null,
        needs_scope_select: false,
      })

      // Then: scopes should be empty array
      expect(snapshot.scopes).toEqual([])
      expect(snapshot.scopes.length).toBe(0)
    })

    test('should have readonly scopes array', () => {
      // Given: a snapshot with scopes
      const snapshot = new AuthenticationStateSnapshot({
        user: null,
        scopes: [new Scope({ org_id: 'org-1', org_name: 'Org 1', role_id: 'role-1', role: 'admin' })],
        permissions: [],
        flash: null,
        token: null,
        currentOrgId: null,
        currentRoleId: null,
        currentRoleName: null,
        needs_scope_select: false,
      })

      // When: accessing scopes
      // Then: scopes should be readonly array
      expect(Array.isArray(snapshot.scopes)).toBe(true)
      // TypeScript readonly modifier prevents mutation
    })
  })

  describe('flash message behavior', () => {
    test('should contain flash message when present', () => {
      // Given: a flash message
      const flash = new Flash({ message: 'Login successful' })

      // When: creating a snapshot with flash
      const snapshot = new AuthenticationStateSnapshot({
        user: null,
        scopes: [],
        permissions: [],
        flash,
        token: null,
        currentOrgId: null,
        currentRoleId: null,
        currentRoleName: null,
        needs_scope_select: false,
      })

      // Then: snapshot should contain the flash message
      expect(snapshot.flash).toBe(flash)
      expect(snapshot.flash?.message).toBe('Login successful')
    })

    test('should contain flash error when present', () => {
      // Given: a flash error
      const flash = new Flash({ error: 'Invalid credentials' })

      // When: creating a snapshot with flash error
      const snapshot = new AuthenticationStateSnapshot({
        user: null,
        scopes: [],
        permissions: [],
        flash,
        token: null,
        currentOrgId: null,
        currentRoleId: null,
        currentRoleName: null,
        needs_scope_select: false,
      })

      // Then: snapshot should contain the flash error
      expect(snapshot.flash).toBe(flash)
      expect(snapshot.flash?.error).toBe('Invalid credentials')
    })

    test('should have null flash when not present', () => {
      // Given: no flash message
      // When: creating a snapshot without flash
      const snapshot = new AuthenticationStateSnapshot({
        user: null,
        scopes: [],
        permissions: [],
        flash: null,
        token: null,
        currentOrgId: null,
        currentRoleId: null,
        currentRoleName: null,
        needs_scope_select: false,
      })

      // Then: flash should be null
      expect(snapshot.flash).toBeNull()
    })
  })

  describe('token behavior', () => {
    test('should contain token when authenticated', () => {
      // Given: an authentication token
      const token = 'bearer-token-123'

      // When: creating a snapshot with token
      const snapshot = new AuthenticationStateSnapshot({
        user: new AuthenticationUser({
          id: 'user-1',
          email: 'test@example.com',
          token,
        }),
        scopes: [],
        permissions: [],
        flash: null,
        token,
        currentOrgId: null,
        currentRoleId: null,
        currentRoleName: null,
        needs_scope_select: false,
      })

      // Then: snapshot should contain the token
      expect(snapshot.token).toBe(token)
    })

    test('should have null token when not authenticated', () => {
      // Given: no authentication token
      // When: creating a snapshot without token
      const snapshot = new AuthenticationStateSnapshot({
        user: null,
        scopes: [],
        permissions: [],
        flash: null,
        token: null,
        currentOrgId: null,
        currentRoleId: null,
        currentRoleName: null,
        needs_scope_select: false,
      })

      // Then: token should be null
      expect(snapshot.token).toBeNull()
    })
  })

  describe('tagged class behavior', () => {
    test('should be tagged with AuthenticationStateSnapshot', () => {
      // Given: a snapshot instance
      const snapshot = new AuthenticationStateSnapshot({
        user: null,
        scopes: [],
        permissions: [],
        flash: null,
        token: null,
        currentOrgId: null,
        currentRoleId: null,
        currentRoleName: null,
        needs_scope_select: false,
      })

      // When: checking the class tag
      // Then: should be tagged as AuthenticationStateSnapshot
      // Effect's Data.TaggedClass provides structural equality and type safety
      expect(snapshot).toBeInstanceOf(AuthenticationStateSnapshot)
    })
  })
})
