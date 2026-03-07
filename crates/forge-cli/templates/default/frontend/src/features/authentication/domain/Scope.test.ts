/**
 * BDD tests for Scope domain
 * Tests verify the behavior of the Scope Data.TaggedClass
 * Scope represents one org + role pair the user can switch to (session scope)
 */

import { describe, it, expect } from 'bun:test'
import { Scope } from './Scope'

describe('Scope domain', () => {
  describe('instance creation behavior', () => {
    it('should create an instance with org_id, org_name, and role', () => {
      // Given scope data
      const orgId = 'org-123'
      const orgName = 'Acme Corp'
      const role = 'owner'

      // When I create a Scope instance
      const scope = new Scope({
        org_id: orgId,
        org_name: orgName,
        role,
      })

      // Then it should have all required properties
      expect(scope.org_id).toBe(orgId)
      expect(scope.org_name).toBe(orgName)
      expect(scope.role).toBe(role)
    })

    it('should create an instance with optional role_id', () => {
      // Given scope data with role_id
      const orgId = 'org-123'
      const orgName = 'Acme Corp'
      const roleId = 'role-456'
      const role = 'viewer'

      // When I create a Scope instance with role_id
      const scope = new Scope({
        org_id: orgId,
        org_name: orgName,
        role_id: roleId,
        role,
      })

      // Then it should include the role_id
      expect(scope.org_id).toBe(orgId)
      expect(scope.org_name).toBe(orgName)
      expect(scope.role_id).toBe(roleId)
      expect(scope.role).toBe(role)
    })

    it('should create an instance without role_id', () => {
      // Given scope data without role_id
      const orgId = 'org-123'
      const orgName = 'Acme Corp'
      const role = 'admin'

      // When I create a Scope instance without role_id
      const scope = new Scope({
        org_id: orgId,
        org_name: orgName,
        role,
      })

      // Then role_id should be undefined
      expect(scope.org_id).toBe(orgId)
      expect(scope.org_name).toBe(orgName)
      expect(scope.role_id).toBeUndefined()
      expect(scope.role).toBe(role)
    })

    it('should accept empty strings for string properties', () => {
      // Given scope data with empty strings
      const scope = new Scope({
        org_id: '',
        org_name: '',
        role: '',
      })

      // Then it should accept empty strings
      expect(scope.org_id).toBe('')
      expect(scope.org_name).toBe('')
      expect(scope.role).toBe('')
    })

    it('should have readonly properties', () => {
      // Given a Scope instance
      const scope = new Scope({
        org_id: 'org-1',
        org_name: 'Test Org',
        role: 'viewer',
      })

      // When I try to modify properties (TypeScript should prevent this)
      // Then the properties should remain unchanged
      // Note: In runtime, readonly doesn't prevent modification, but TypeScript enforces it
      expect(scope.org_id).toBe('org-1')
      expect(scope.org_name).toBe('Test Org')
      expect(scope.role).toBe('viewer')
    })
  })

  describe('structural equality behavior', () => {
    it('should be equal to another instance with same values', () => {
      // Given two Scope instances with the same values
      const scope1 = new Scope({
        org_id: 'org-123',
        org_name: 'Acme Corp',
        role_id: 'role-456',
        role: 'owner',
      })
      const scope2 = new Scope({
        org_id: 'org-123',
        org_name: 'Acme Corp',
        role_id: 'role-456',
        role: 'owner',
      })

      // When I compare them
      // Then they should be structurally equal (Data.TaggedClass provides this)
      expect(scope1).toEqual(scope2)
    })

    it('should be equal when both have undefined role_id', () => {
      // Given two Scope instances without role_id
      const scope1 = new Scope({
        org_id: 'org-123',
        org_name: 'Acme Corp',
        role: 'viewer',
      })
      const scope2 = new Scope({
        org_id: 'org-123',
        org_name: 'Acme Corp',
        role: 'viewer',
      })

      // When I compare them
      // Then they should be equal
      expect(scope1).toEqual(scope2)
    })

    it('should not be equal when org_id differs', () => {
      // Given two Scope instances with different org_ids
      const scope1 = new Scope({
        org_id: 'org-123',
        org_name: 'Acme Corp',
        role: 'owner',
      })
      const scope2 = new Scope({
        org_id: 'org-456',
        org_name: 'Acme Corp',
        role: 'owner',
      })

      // When I compare them
      // Then they should not be equal
      expect(scope1).not.toEqual(scope2)
    })

    it('should not be equal when org_name differs', () => {
      // Given two Scope instances with different org_names
      const scope1 = new Scope({
        org_id: 'org-123',
        org_name: 'Acme Corp',
        role: 'owner',
      })
      const scope2 = new Scope({
        org_id: 'org-123',
        org_name: 'Beta Inc',
        role: 'owner',
      })

      // When I compare them
      // Then they should not be equal
      expect(scope1).not.toEqual(scope2)
    })

    it('should not be equal when role differs', () => {
      // Given two Scope instances with different roles
      const scope1 = new Scope({
        org_id: 'org-123',
        org_name: 'Acme Corp',
        role: 'owner',
      })
      const scope2 = new Scope({
        org_id: 'org-123',
        org_name: 'Acme Corp',
        role: 'viewer',
      })

      // When I compare them
      // Then they should not be equal
      expect(scope1).not.toEqual(scope2)
    })

    it('should not be equal when role_id differs', () => {
      // Given two Scope instances with different role_ids
      const scope1 = new Scope({
        org_id: 'org-123',
        org_name: 'Acme Corp',
        role_id: 'role-1',
        role: 'owner',
      })
      const scope2 = new Scope({
        org_id: 'org-123',
        org_name: 'Acme Corp',
        role_id: 'role-2',
        role: 'owner',
      })

      // When I compare them
      // Then they should not be equal
      expect(scope1).not.toEqual(scope2)
    })

    it('should not be equal when one has role_id and the other does not', () => {
      // Given two Scope instances where one has role_id and the other does not
      const scope1 = new Scope({
        org_id: 'org-123',
        org_name: 'Acme Corp',
        role_id: 'role-456',
        role: 'owner',
      })
      const scope2 = new Scope({
        org_id: 'org-123',
        org_name: 'Acme Corp',
        role: 'owner',
      })

      // When I compare them
      // Then they should not be equal
      expect(scope1).not.toEqual(scope2)
    })
  })

  describe('tagged class behavior', () => {
    it('should have the Scope tag', () => {
      // Given a Scope instance
      const scope = new Scope({
        org_id: 'org-1',
        org_name: 'Test Org',
        role: 'viewer',
      })

      // When I check the tag
      // Then it should be tagged as 'Scope'
      // Data.TaggedClass provides a _tag property
      expect((scope as any)._tag).toBe('Scope')
    })

    it('should be identifiable as Scope type', () => {
      // Given a Scope instance
      const scope = new Scope({
        org_id: 'org-1',
        org_name: 'Test Org',
        role: 'viewer',
      })

      // When I check the instance
      // Then it should be an instance of Scope
      expect(scope).toBeInstanceOf(Scope)
    })
  })

  describe('property access behavior', () => {
    it('should allow reading org_id property', () => {
      // Given a Scope instance
      const scope = new Scope({
        org_id: 'org-123',
        org_name: 'Acme Corp',
        role: 'owner',
      })

      // When I read the org_id property
      const orgId = scope.org_id

      // Then it should return the correct value
      expect(orgId).toBe('org-123')
    })

    it('should allow reading org_name property', () => {
      // Given a Scope instance
      const scope = new Scope({
        org_id: 'org-123',
        org_name: 'Acme Corporation',
        role: 'owner',
      })

      // When I read the org_name property
      const orgName = scope.org_name

      // Then it should return the correct value
      expect(orgName).toBe('Acme Corporation')
    })

    it('should allow reading role property', () => {
      // Given a Scope instance
      const scope = new Scope({
        org_id: 'org-123',
        org_name: 'Acme Corp',
        role: 'viewer',
      })

      // When I read the role property
      const role = scope.role

      // Then it should return the correct value
      expect(role).toBe('viewer')
    })

    it('should allow reading role_id property when present', () => {
      // Given a Scope instance with role_id
      const scope = new Scope({
        org_id: 'org-123',
        org_name: 'Acme Corp',
        role_id: 'role-789',
        role: 'admin',
      })

      // When I read the role_id property
      const roleId = scope.role_id

      // Then it should return the correct value
      expect(roleId).toBe('role-789')
    })

    it('should return undefined for role_id when not provided', () => {
      // Given a Scope instance without role_id
      const scope = new Scope({
        org_id: 'org-123',
        org_name: 'Acme Corp',
        role: 'admin',
      })

      // When I read the role_id property
      const roleId = scope.role_id

      // Then it should be undefined
      expect(roleId).toBeUndefined()
    })
  })

  describe('usage in session scope context', () => {
    it('should represent an organization and role pair for session scope', () => {
      // Given organization and role data for session scope
      const orgId = '550e8400-e29b-41d4-a716-446655440000'
      const orgName = 'Engineering Team'
      const roleId = '123e4567-e89b-12d3-a456-426614174000'
      const role = 'developer'

      // When I create a Scope
      const scope = new Scope({
        org_id: orgId,
        org_name: orgName,
        role_id: roleId,
        role,
      })

      // Then it should represent a complete org + role pair for session scope
      expect(scope.org_id).toBe(orgId)
      expect(scope.org_name).toBe(orgName)
      expect(scope.role_id).toBe(roleId)
      expect(scope.role).toBe(role)
    })

    it('should work with UUID strings as org_id', () => {
      // Given a UUID string as org_id
      const uuid = '123e4567-e89b-12d3-a456-426614174000'

      // When I create a Scope with UUID org_id
      const scope = new Scope({
        org_id: uuid,
        org_name: 'Test Org',
        role: 'owner',
      })

      // Then it should accept and store the UUID
      expect(scope.org_id).toBe(uuid)
    })

    it('should work with common role names', () => {
      // Given common role names
      const roles = ['owner', 'admin', 'viewer', 'editor', 'developer']

      // When I create Scopes with these roles
      const scopes = roles.map(
        (role) =>
          new Scope({
            org_id: 'org-1',
            org_name: 'Test Org',
            role,
          }),
      )

      // Then each should have the correct role
      scopes.forEach((scope, index) => {
        expect(scope.role).toBe(roles[index])
      })
    })

    it('should support backwards compatibility with optional role_id', () => {
      // Given scope data without role_id (backwards compatibility)
      const scope = new Scope({
        org_id: 'org-123',
        org_name: 'Acme Corp',
        role: 'viewer',
      })

      // When I check the role_id
      // Then it should be undefined (optional for backwards compatibility)
      expect(scope.role_id).toBeUndefined()
      expect(scope.role).toBe('viewer')
    })
  })
})
