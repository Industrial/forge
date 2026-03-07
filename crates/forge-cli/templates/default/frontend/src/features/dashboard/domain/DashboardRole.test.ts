/**
 * BDD tests for DashboardRole domain
 * Tests verify the behavior of the DashboardRole Data.TaggedClass
 * DashboardRole represents a role summary for dropdowns (e.g. org-scoped role list)
 */

import { describe, test, expect } from 'bun:test'
import { DashboardRole } from './DashboardRole'

describe('DashboardRole domain', () => {
  describe('instance creation behavior', () => {
    test('should create an instance with all required properties', () => {
      // Given: dashboard role data with required properties
      const id = 'role-123'
      const name = 'viewer'
      const displayName = 'Viewer'

      // When: I create a DashboardRole instance
      const role = new DashboardRole({
        id,
        name,
        display_name: displayName,
      })

      // Then: it should have all required properties
      expect(role.id).toBe(id)
      expect(role.name).toBe(name)
      expect(role.display_name).toBe(displayName)
    })

    test('should create an instance with null display_name', () => {
      // Given: dashboard role data with null display_name
      const role = new DashboardRole({
        id: 'role-123',
        name: 'admin',
        display_name: null,
      })

      // Then: display_name should be null
      expect(role.display_name).toBeNull()
    })

    test('should create an instance with empty string display_name', () => {
      // Given: dashboard role data with empty string display_name
      const role = new DashboardRole({
        id: 'role-123',
        name: 'member',
        display_name: '',
      })

      // Then: display_name should be empty string
      expect(role.display_name).toBe('')
    })
  })

  describe('property access behavior', () => {
    test('should access id property', () => {
      // Given: a DashboardRole instance
      const role = new DashboardRole({
        id: 'role-456',
        name: 'viewer',
        display_name: 'Viewer',
      })

      // When: accessing id property
      // Then: should return the id
      expect(role.id).toBe('role-456')
    })

    test('should access name property', () => {
      // Given: a DashboardRole instance
      const role = new DashboardRole({
        id: 'role-123',
        name: 'admin',
        display_name: 'Administrator',
      })

      // When: accessing name property
      // Then: should return the name
      expect(role.name).toBe('admin')
    })

    test('should access display_name property', () => {
      // Given: a DashboardRole instance
      const role = new DashboardRole({
        id: 'role-123',
        name: 'owner',
        display_name: 'Owner',
      })

      // When: accessing display_name property
      // Then: should return the display_name
      expect(role.display_name).toBe('Owner')
    })

    test('should access display_name when null', () => {
      // Given: a DashboardRole instance with null display_name
      const role = new DashboardRole({
        id: 'role-123',
        name: 'member',
        display_name: null,
      })

      // When: accessing display_name property
      // Then: should return null
      expect(role.display_name).toBeNull()
    })
  })

  describe('tagged class behavior', () => {
    test('should have the DashboardRole tag', () => {
      // Given: a DashboardRole instance
      const role = new DashboardRole({
        id: 'role-123',
        name: 'viewer',
        display_name: 'Viewer',
      })

      // When: checking the tag
      // Then: it should be tagged as 'DashboardRole'
      // Data.TaggedClass provides a _tag property
      expect((role as any)._tag).toBe('DashboardRole')
    })

    test('should be identifiable as DashboardRole type', () => {
      // Given: a DashboardRole instance
      const role = new DashboardRole({
        id: 'role-123',
        name: 'viewer',
        display_name: 'Viewer',
      })

      // When: checking the instance
      // Then: it should be an instance of DashboardRole
      expect(role).toBeInstanceOf(DashboardRole)
    })
  })

  describe('readonly fields behavior', () => {
    test('should have readonly id field', () => {
      // Given: a DashboardRole instance
      const role = new DashboardRole({
        id: 'role-123',
        name: 'viewer',
        display_name: 'Viewer',
      })

      // When: accessing id property
      // Then: id should be readable
      expect(role.id).toBe('role-123')
      // TypeScript readonly modifier prevents mutation at compile time
    })

    test('should have readonly name field', () => {
      // Given: a DashboardRole instance
      const role = new DashboardRole({
        id: 'role-123',
        name: 'admin',
        display_name: 'Administrator',
      })

      // When: accessing name property
      // Then: name should be readable
      expect(role.name).toBe('admin')
      // TypeScript readonly modifier prevents mutation at compile time
    })

    test('should have readonly display_name field', () => {
      // Given: a DashboardRole instance
      const role = new DashboardRole({
        id: 'role-123',
        name: 'owner',
        display_name: 'Owner',
      })

      // When: accessing display_name property
      // Then: display_name should be readable
      expect(role.display_name).toBe('Owner')
      // TypeScript readonly modifier prevents mutation at compile time
    })
  })

  describe('name values behavior', () => {
    test('should support viewer name', () => {
      // Given: dashboard role with viewer name
      const role = new DashboardRole({
        id: 'role-123',
        name: 'viewer',
        display_name: 'Viewer',
      })

      // Then: name should be viewer
      expect(role.name).toBe('viewer')
    })

    test('should support admin name', () => {
      // Given: dashboard role with admin name
      const role = new DashboardRole({
        id: 'role-123',
        name: 'admin',
        display_name: 'Administrator',
      })

      // Then: name should be admin
      expect(role.name).toBe('admin')
    })

    test('should support owner name', () => {
      // Given: dashboard role with owner name
      const role = new DashboardRole({
        id: 'role-123',
        name: 'owner',
        display_name: 'Owner',
      })

      // Then: name should be owner
      expect(role.name).toBe('owner')
    })

    test('should support member name', () => {
      // Given: dashboard role with member name
      const role = new DashboardRole({
        id: 'role-123',
        name: 'member',
        display_name: 'Member',
      })

      // Then: name should be member
      expect(role.name).toBe('member')
    })

    test('should support custom role names', () => {
      // Given: dashboard role with custom name
      const role = new DashboardRole({
        id: 'role-123',
        name: 'custom-role',
        display_name: 'Custom Role',
      })

      // Then: name should be custom-role
      expect(role.name).toBe('custom-role')
    })
  })

  describe('display_name behavior', () => {
    test('should support display_name matching name', () => {
      // Given: dashboard role with display_name matching name
      const role = new DashboardRole({
        id: 'role-123',
        name: 'viewer',
        display_name: 'viewer',
      })

      // Then: display_name should match name
      expect(role.display_name).toBe('viewer')
    })

    test('should support display_name different from name', () => {
      // Given: dashboard role with display_name different from name
      const role = new DashboardRole({
        id: 'role-123',
        name: 'admin',
        display_name: 'Administrator',
      })

      // Then: display_name should be different from name
      expect(role.name).toBe('admin')
      expect(role.display_name).toBe('Administrator')
    })

    test('should support null display_name', () => {
      // Given: dashboard role with null display_name
      const role = new DashboardRole({
        id: 'role-123',
        name: 'viewer',
        display_name: null,
      })

      // Then: display_name should be null
      expect(role.display_name).toBeNull()
    })

    test('should support empty string display_name', () => {
      // Given: dashboard role with empty string display_name
      const role = new DashboardRole({
        id: 'role-123',
        name: 'viewer',
        display_name: '',
      })

      // Then: display_name should be empty string
      expect(role.display_name).toBe('')
    })
  })

  describe('edge cases', () => {
    test('should handle empty string id', () => {
      // Given: dashboard role with empty string id
      const role = new DashboardRole({
        id: '',
        name: 'viewer',
        display_name: 'Viewer',
      })

      // Then: should accept empty string id
      expect(role.id).toBe('')
    })

    test('should handle empty string name', () => {
      // Given: dashboard role with empty string name
      const role = new DashboardRole({
        id: 'role-123',
        name: '',
        display_name: 'Viewer',
      })

      // Then: should accept empty string name
      expect(role.name).toBe('')
    })

    test('should handle long display_name strings', () => {
      // Given: dashboard role with long display_name
      const longDisplayName = 'A'.repeat(100)
      const role = new DashboardRole({
        id: 'role-123',
        name: 'viewer',
        display_name: longDisplayName,
      })

      // Then: should accept long display_name
      expect(role.display_name).toBe(longDisplayName)
      expect(role.display_name?.length).toBe(100)
    })

    test('should handle UUID format id', () => {
      // Given: dashboard role with UUID format id
      const uuidId = '550e8400-e29b-41d4-a716-446655440000'
      const role = new DashboardRole({
        id: uuidId,
        name: 'viewer',
        display_name: 'Viewer',
      })

      // Then: should accept UUID format id
      expect(role.id).toBe(uuidId)
    })
  })
})
