/**
 * BDD tests for Role domain
 * Tests verify the behavior of the Role Data.TaggedClass
 * Role represents a full role entity used by the Roles service
 */

import { describe, it, expect } from 'bun:test'
import { Role } from './Role'

describe('Role domain', () => {
  describe('instance creation behavior', () => {
    it('should create an instance with required properties', () => {
      // Given role data with required properties
      const id = 'role-123'
      const orgId = 'org-456'
      const name = 'viewer'

      // When I create a Role instance
      const role = new Role({
        id,
        org_id: orgId,
        name,
        display_name: null,
      })

      // Then it should have all required properties
      expect(role.id).toBe(id)
      expect(role.org_id).toBe(orgId)
      expect(role.name).toBe(name)
      expect(role.display_name).toBeNull()
    })

    it('should create an instance with display_name', () => {
      // Given role data with display_name
      const role = new Role({
        id: 'role-123',
        org_id: 'org-456',
        name: 'owner',
        display_name: 'Owner',
      })

      // Then it should include the display_name
      expect(role.display_name).toBe('Owner')
    })

    it('should create an instance with optional created_at', () => {
      // Given role data with created_at
      const createdAt = '2024-01-01T00:00:00Z'
      const role = new Role({
        id: 'role-123',
        org_id: 'org-456',
        name: 'admin',
        display_name: 'Administrator',
        created_at: createdAt,
      })

      // Then it should include created_at
      expect(role.created_at).toBe(createdAt)
    })

    it('should create an instance with optional updated_at', () => {
      // Given role data with updated_at
      const updatedAt = '2024-01-02T00:00:00Z'
      const role = new Role({
        id: 'role-123',
        org_id: 'org-456',
        name: 'editor',
        display_name: 'Editor',
        updated_at: updatedAt,
      })

      // Then it should include updated_at
      expect(role.updated_at).toBe(updatedAt)
    })

    it('should create an instance without optional timestamp fields', () => {
      // Given role data without optional fields
      const role = new Role({
        id: 'role-123',
        org_id: 'org-456',
        name: 'viewer',
        display_name: null,
      })

      // Then optional fields should be undefined
      expect(role.created_at).toBeUndefined()
      expect(role.updated_at).toBeUndefined()
    })

    it('should accept null for display_name', () => {
      // Given role data with null display_name
      const role = new Role({
        id: 'role-123',
        org_id: 'org-456',
        name: 'custom',
        display_name: null,
      })

      // Then display_name should be null
      expect(role.display_name).toBeNull()
    })

    it('should accept empty strings for string properties', () => {
      // Given role data with empty strings
      const role = new Role({
        id: '',
        org_id: '',
        name: '',
        display_name: '',
      })

      // Then it should accept empty strings
      expect(role.id).toBe('')
      expect(role.org_id).toBe('')
      expect(role.name).toBe('')
      expect(role.display_name).toBe('')
    })

    it('should have readonly properties', () => {
      // Given a Role instance
      const role = new Role({
        id: 'role-1',
        org_id: 'org-1',
        name: 'viewer',
        display_name: 'Viewer',
      })

      // When I try to modify properties (TypeScript should prevent this)
      // Then the properties should remain unchanged
      // Note: In runtime, readonly doesn't prevent modification, but TypeScript enforces it
      expect(role.id).toBe('role-1')
      expect(role.org_id).toBe('org-1')
      expect(role.name).toBe('viewer')
      expect(role.display_name).toBe('Viewer')
    })
  })

  describe('structural equality behavior', () => {
    it('should be equal to another instance with same values', () => {
      // Given two Role instances with the same values
      const role1 = new Role({
        id: 'role-123',
        org_id: 'org-456',
        name: 'owner',
        display_name: 'Owner',
        created_at: '2024-01-01T00:00:00Z',
        updated_at: '2024-01-02T00:00:00Z',
      })
      const role2 = new Role({
        id: 'role-123',
        org_id: 'org-456',
        name: 'owner',
        display_name: 'Owner',
        created_at: '2024-01-01T00:00:00Z',
        updated_at: '2024-01-02T00:00:00Z',
      })

      // When I compare them
      // Then they should be structurally equal (Data.TaggedClass provides this)
      expect(role1).toEqual(role2)
    })

    it('should be equal when both have null display_name', () => {
      // Given two Role instances with null display_name
      const role1 = new Role({
        id: 'role-123',
        org_id: 'org-456',
        name: 'viewer',
        display_name: null,
      })
      const role2 = new Role({
        id: 'role-123',
        org_id: 'org-456',
        name: 'viewer',
        display_name: null,
      })

      // When I compare them
      // Then they should be equal
      expect(role1).toEqual(role2)
    })

    it('should be equal when both have undefined optional fields', () => {
      // Given two Role instances without optional fields
      const role1 = new Role({
        id: 'role-123',
        org_id: 'org-456',
        name: 'admin',
        display_name: null,
      })
      const role2 = new Role({
        id: 'role-123',
        org_id: 'org-456',
        name: 'admin',
        display_name: null,
      })

      // When I compare them
      // Then they should be equal
      expect(role1).toEqual(role2)
    })

    it('should not be equal when id differs', () => {
      // Given two Role instances with different ids
      const role1 = new Role({
        id: 'role-123',
        org_id: 'org-456',
        name: 'owner',
        display_name: 'Owner',
      })
      const role2 = new Role({
        id: 'role-789',
        org_id: 'org-456',
        name: 'owner',
        display_name: 'Owner',
      })

      // When I compare them
      // Then they should not be equal
      expect(role1).not.toEqual(role2)
    })

    it('should not be equal when org_id differs', () => {
      // Given two Role instances with different org_ids
      const role1 = new Role({
        id: 'role-123',
        org_id: 'org-456',
        name: 'owner',
        display_name: 'Owner',
      })
      const role2 = new Role({
        id: 'role-123',
        org_id: 'org-789',
        name: 'owner',
        display_name: 'Owner',
      })

      // When I compare them
      // Then they should not be equal
      expect(role1).not.toEqual(role2)
    })

    it('should not be equal when name differs', () => {
      // Given two Role instances with different names
      const role1 = new Role({
        id: 'role-123',
        org_id: 'org-456',
        name: 'owner',
        display_name: 'Owner',
      })
      const role2 = new Role({
        id: 'role-123',
        org_id: 'org-456',
        name: 'viewer',
        display_name: 'Owner',
      })

      // When I compare them
      // Then they should not be equal
      expect(role1).not.toEqual(role2)
    })

    it('should not be equal when display_name differs', () => {
      // Given two Role instances with different display_names
      const role1 = new Role({
        id: 'role-123',
        org_id: 'org-456',
        name: 'owner',
        display_name: 'Owner',
      })
      const role2 = new Role({
        id: 'role-123',
        org_id: 'org-456',
        name: 'owner',
        display_name: 'Administrator',
      })

      // When I compare them
      // Then they should not be equal
      expect(role1).not.toEqual(role2)
    })

    it('should not be equal when one has display_name and the other is null', () => {
      // Given two Role instances where one has display_name and the other is null
      const role1 = new Role({
        id: 'role-123',
        org_id: 'org-456',
        name: 'owner',
        display_name: 'Owner',
      })
      const role2 = new Role({
        id: 'role-123',
        org_id: 'org-456',
        name: 'owner',
        display_name: null,
      })

      // When I compare them
      // Then they should not be equal
      expect(role1).not.toEqual(role2)
    })

    it('should not be equal when created_at differs', () => {
      // Given two Role instances with different created_at values
      const role1 = new Role({
        id: 'role-123',
        org_id: 'org-456',
        name: 'owner',
        display_name: 'Owner',
        created_at: '2024-01-01T00:00:00Z',
      })
      const role2 = new Role({
        id: 'role-123',
        org_id: 'org-456',
        name: 'owner',
        display_name: 'Owner',
        created_at: '2024-01-02T00:00:00Z',
      })

      // When I compare them
      // Then they should not be equal
      expect(role1).not.toEqual(role2)
    })

    it('should not be equal when one has created_at and the other does not', () => {
      // Given two Role instances where one has created_at and the other does not
      const role1 = new Role({
        id: 'role-123',
        org_id: 'org-456',
        name: 'owner',
        display_name: 'Owner',
        created_at: '2024-01-01T00:00:00Z',
      })
      const role2 = new Role({
        id: 'role-123',
        org_id: 'org-456',
        name: 'owner',
        display_name: 'Owner',
      })

      // When I compare them
      // Then they should not be equal
      expect(role1).not.toEqual(role2)
    })
  })

  describe('tagged class behavior', () => {
    it('should have the Role tag', () => {
      // Given a Role instance
      const role = new Role({
        id: 'role-1',
        org_id: 'org-1',
        name: 'viewer',
        display_name: 'Viewer',
      })

      // When I check the tag
      // Then it should be tagged as 'Role'
      // Data.TaggedClass provides a _tag property
      expect((role as any)._tag).toBe('Role')
    })

    it('should be identifiable as Role type', () => {
      // Given a Role instance
      const role = new Role({
        id: 'role-1',
        org_id: 'org-1',
        name: 'viewer',
        display_name: 'Viewer',
      })

      // When I check the instance
      // Then it should be an instance of Role
      expect(role).toBeInstanceOf(Role)
    })
  })

  describe('property access behavior', () => {
    it('should allow reading id property', () => {
      // Given a Role instance
      const role = new Role({
        id: 'role-123',
        org_id: 'org-456',
        name: 'owner',
        display_name: 'Owner',
      })

      // When I read the id property
      const id = role.id

      // Then it should return the correct value
      expect(id).toBe('role-123')
    })

    it('should allow reading org_id property', () => {
      // Given a Role instance
      const role = new Role({
        id: 'role-123',
        org_id: 'org-456',
        name: 'owner',
        display_name: 'Owner',
      })

      // When I read the org_id property
      const orgId = role.org_id

      // Then it should return the correct value
      expect(orgId).toBe('org-456')
    })

    it('should allow reading name property', () => {
      // Given a Role instance
      const role = new Role({
        id: 'role-123',
        org_id: 'org-456',
        name: 'admin',
        display_name: 'Administrator',
      })

      // When I read the name property
      const name = role.name

      // Then it should return the correct value
      expect(name).toBe('admin')
    })

    it('should allow reading display_name property when present', () => {
      // Given a Role instance with display_name
      const role = new Role({
        id: 'role-123',
        org_id: 'org-456',
        name: 'viewer',
        display_name: 'Viewer Role',
      })

      // When I read the display_name property
      const displayName = role.display_name

      // Then it should return the correct value
      expect(displayName).toBe('Viewer Role')
    })

    it('should return null for display_name when null', () => {
      // Given a Role instance with null display_name
      const role = new Role({
        id: 'role-123',
        org_id: 'org-456',
        name: 'custom',
        display_name: null,
      })

      // When I read the display_name property
      const displayName = role.display_name

      // Then it should be null
      expect(displayName).toBeNull()
    })

    it('should allow reading created_at property when present', () => {
      // Given a Role instance with created_at
      const createdAt = '2024-01-01T00:00:00Z'
      const role = new Role({
        id: 'role-123',
        org_id: 'org-456',
        name: 'owner',
        display_name: 'Owner',
        created_at: createdAt,
      })

      // When I read the created_at property
      const timestamp = role.created_at

      // Then it should return the correct value
      expect(timestamp).toBe(createdAt)
    })

    it('should return undefined for created_at when not provided', () => {
      // Given a Role instance without created_at
      const role = new Role({
        id: 'role-123',
        org_id: 'org-456',
        name: 'viewer',
        display_name: 'Viewer',
      })

      // When I read the created_at property
      const timestamp = role.created_at

      // Then it should be undefined
      expect(timestamp).toBeUndefined()
    })

    it('should allow reading updated_at property when present', () => {
      // Given a Role instance with updated_at
      const updatedAt = '2024-01-02T00:00:00Z'
      const role = new Role({
        id: 'role-123',
        org_id: 'org-456',
        name: 'admin',
        display_name: 'Administrator',
        updated_at: updatedAt,
      })

      // When I read the updated_at property
      const timestamp = role.updated_at

      // Then it should return the correct value
      expect(timestamp).toBe(updatedAt)
    })

    it('should return undefined for updated_at when not provided', () => {
      // Given a Role instance without updated_at
      const role = new Role({
        id: 'role-123',
        org_id: 'org-456',
        name: 'editor',
        display_name: 'Editor',
      })

      // When I read the updated_at property
      const timestamp = role.updated_at

      // Then it should be undefined
      expect(timestamp).toBeUndefined()
    })
  })

  describe('usage in roles service context', () => {
    it('should represent a full role entity for CRUD operations', () => {
      // Given role data for CRUD operations
      const role = new Role({
        id: '550e8400-e29b-41d4-a716-446655440000',
        org_id: '123e4567-e89b-12d3-a456-426614174000',
        name: 'developer',
        display_name: 'Developer',
        created_at: '2024-01-01T00:00:00Z',
        updated_at: '2024-01-02T00:00:00Z',
      })

      // Then it should represent a complete role entity
      expect(role.id).toBeTruthy()
      expect(role.org_id).toBeTruthy()
      expect(role.name).toBe('developer')
      expect(role.display_name).toBe('Developer')
    })

    it('should work with UUID strings as id and org_id', () => {
      // Given UUID strings
      const roleId = '123e4567-e89b-12d3-a456-426614174000'
      const orgId = '550e8400-e29b-41d4-a716-446655440000'

      // When I create a Role with UUID strings
      const role = new Role({
        id: roleId,
        org_id: orgId,
        name: 'owner',
        display_name: 'Owner',
      })

      // Then it should accept and store the UUIDs
      expect(role.id).toBe(roleId)
      expect(role.org_id).toBe(orgId)
    })

    it('should work with common role names', () => {
      // Given common role names
      const roleNames = ['owner', 'admin', 'viewer', 'editor', 'developer']

      // When I create Roles with these names
      const roles = roleNames.map((name) =>
        new Role({
          id: `role-${name}`,
          org_id: 'org-1',
          name,
          display_name: name.charAt(0).toUpperCase() + name.slice(1),
        }),
      )

      // Then each should have the correct name
      roles.forEach((role, index) => {
        expect(role.name).toBe(roleNames[index])
      })
    })

    it('should support roles with and without display names', () => {
      // Given roles with different display_name configurations
      const roleWithDisplay = new Role({
        id: 'role-1',
        org_id: 'org-1',
        name: 'owner',
        display_name: 'Organization Owner',
      })
      const roleWithoutDisplay = new Role({
        id: 'role-2',
        org_id: 'org-1',
        name: 'viewer',
        display_name: null,
      })

      // Then both should be valid
      expect(roleWithDisplay.display_name).toBe('Organization Owner')
      expect(roleWithoutDisplay.display_name).toBeNull()
    })

    it('should support ISO 8601 timestamp strings', () => {
      // Given ISO 8601 timestamp strings
      const createdAt = '2024-01-01T12:00:00Z'
      const updatedAt = '2024-01-02T15:30:00Z'

      // When I create a Role with timestamps
      const role = new Role({
        id: 'role-123',
        org_id: 'org-456',
        name: 'admin',
        display_name: 'Administrator',
        created_at: createdAt,
        updated_at: updatedAt,
      })

      // Then it should store the timestamps correctly
      expect(role.created_at).toBe(createdAt)
      expect(role.updated_at).toBe(updatedAt)
    })
  })
})
