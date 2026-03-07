/**
 * BDD tests for Assignment domain class
 * Tests verify the behavior of the Assignment Data.TaggedClass
 */

import { describe, it, expect } from 'bun:test'
import { Assignment } from './Assignment'

describe('Assignment', () => {
  describe('instance creation behavior', () => {
    it('should create an instance with required fields', () => {
      // Given required fields
      const scope = 'global'
      const roleName = 'admin'
      const permissionKey = 'read'

      // When I create an Assignment instance
      const assignment = new Assignment({
        scope,
        role_name: roleName,
        permission_key: permissionKey,
      })

      // Then it should have the correct properties
      expect(assignment.scope).toBe(scope)
      expect(assignment.role_name).toBe(roleName)
      expect(assignment.permission_key).toBe(permissionKey)
    })

    it('should create an instance with optional org_id', () => {
      // Given all fields including org_id
      const assignment = new Assignment({
        scope: 'org',
        role_name: 'user',
        permission_key: 'write',
        org_id: 'org-123',
      })

      // Then it should have org_id set
      expect(assignment.org_id).toBe('org-123')
    })

    it('should create an instance without org_id', () => {
      // Given fields without org_id
      const assignment = new Assignment({
        scope: 'global',
        role_name: 'admin',
        permission_key: 'read',
      })

      // Then org_id should be undefined
      expect(assignment.org_id).toBeUndefined()
    })

    it('should create an instance with null org_id', () => {
      // Given fields with null org_id
      const assignment = new Assignment({
        scope: 'org',
        role_name: 'user',
        permission_key: 'write',
        org_id: null,
      })

      // Then org_id should be null
      expect(assignment.org_id).toBeNull()
    })

    it('should accept empty strings for fields', () => {
      // Given empty string values
      const assignment = new Assignment({
        scope: '',
        role_name: '',
        permission_key: '',
      })

      // Then it should accept empty strings
      expect(assignment.scope).toBe('')
      expect(assignment.role_name).toBe('')
      expect(assignment.permission_key).toBe('')
    })
  })

  describe('property access behavior', () => {
    it('should allow reading scope property', () => {
      // Given an Assignment instance
      const assignment = new Assignment({
        scope: 'global',
        role_name: 'admin',
        permission_key: 'read',
      })

      // When I access the scope property
      // Then it should return the value
      expect(assignment.scope).toBe('global')
    })

    it('should allow reading role_name property', () => {
      // Given an Assignment instance
      const assignment = new Assignment({
        scope: 'global',
        role_name: 'admin',
        permission_key: 'read',
      })

      // When I access the role_name property
      // Then it should return the value
      expect(assignment.role_name).toBe('admin')
    })

    it('should allow reading permission_key property', () => {
      // Given an Assignment instance
      const assignment = new Assignment({
        scope: 'global',
        role_name: 'admin',
        permission_key: 'read',
      })

      // When I access the permission_key property
      // Then it should return the value
      expect(assignment.permission_key).toBe('read')
    })

    it('should allow reading org_id property when set', () => {
      // Given an Assignment instance with org_id
      const assignment = new Assignment({
        scope: 'org',
        role_name: 'user',
        permission_key: 'write',
        org_id: 'org-123',
      })

      // When I access the org_id property
      // Then it should return the value
      expect(assignment.org_id).toBe('org-123')
    })

    it('should return undefined for org_id when not set', () => {
      // Given an Assignment instance without org_id
      const assignment = new Assignment({
        scope: 'global',
        role_name: 'admin',
        permission_key: 'read',
      })

      // When I access the org_id property
      // Then it should return undefined
      expect(assignment.org_id).toBeUndefined()
    })

    it('should have readonly properties', () => {
      // Given an Assignment instance
      const assignment = new Assignment({
        scope: 'global',
        role_name: 'admin',
        permission_key: 'read',
      })

      // When I check the properties
      // Then they should be readonly (enforced by TypeScript)
      // Runtime check: properties exist and are accessible
      expect(assignment.scope).toBeDefined()
      expect(assignment.role_name).toBeDefined()
      expect(assignment.permission_key).toBeDefined()
    })
  })

  describe('tagged class behavior', () => {
    it('should have the Assignment tag', () => {
      // Given an Assignment instance
      const assignment = new Assignment({
        scope: 'global',
        role_name: 'admin',
        permission_key: 'read',
      })

      // When I check the tag
      // Then it should be tagged as 'Assignment'
      // Data.TaggedClass provides a _tag property
      expect((assignment as any)._tag).toBe('Assignment')
    })

    it('should be identifiable as Assignment type', () => {
      // Given an Assignment instance
      const assignment = new Assignment({
        scope: 'global',
        role_name: 'admin',
        permission_key: 'read',
      })

      // When I check the instance
      // Then it should be an instance of Assignment
      expect(assignment).toBeInstanceOf(Assignment)
    })
  })

  describe('structural equality behavior', () => {
    it('should be equal when all fields match', () => {
      // Given two Assignment instances with same data
      const assignment1 = new Assignment({
        scope: 'global',
        role_name: 'admin',
        permission_key: 'read',
      })
      const assignment2 = new Assignment({
        scope: 'global',
        role_name: 'admin',
        permission_key: 'read',
      })

      // When I compare them
      // Then they should be structurally equal
      // Effect's Data.TaggedClass provides structural equality
      expect(assignment1).toEqual(assignment2)
    })

    it('should be equal when org_id matches', () => {
      // Given two Assignment instances with same data including org_id
      const assignment1 = new Assignment({
        scope: 'org',
        role_name: 'user',
        permission_key: 'write',
        org_id: 'org-123',
      })
      const assignment2 = new Assignment({
        scope: 'org',
        role_name: 'user',
        permission_key: 'write',
        org_id: 'org-123',
      })

      // When I compare them
      // Then they should be equal
      expect(assignment1).toEqual(assignment2)
    })

    it('should not be equal when scope differs', () => {
      // Given two Assignment instances with different scope
      const assignment1 = new Assignment({
        scope: 'global',
        role_name: 'admin',
        permission_key: 'read',
      })
      const assignment2 = new Assignment({
        scope: 'org',
        role_name: 'admin',
        permission_key: 'read',
      })

      // When I compare them
      // Then they should not be equal
      expect(assignment1).not.toEqual(assignment2)
    })

    it('should not be equal when role_name differs', () => {
      // Given two Assignment instances with different role_name
      const assignment1 = new Assignment({
        scope: 'global',
        role_name: 'admin',
        permission_key: 'read',
      })
      const assignment2 = new Assignment({
        scope: 'global',
        role_name: 'user',
        permission_key: 'read',
      })

      // When I compare them
      // Then they should not be equal
      expect(assignment1).not.toEqual(assignment2)
    })

    it('should not be equal when permission_key differs', () => {
      // Given two Assignment instances with different permission_key
      const assignment1 = new Assignment({
        scope: 'global',
        role_name: 'admin',
        permission_key: 'read',
      })
      const assignment2 = new Assignment({
        scope: 'global',
        role_name: 'admin',
        permission_key: 'write',
      })

      // When I compare them
      // Then they should not be equal
      expect(assignment1).not.toEqual(assignment2)
    })

    it('should not be equal when one has org_id and the other does not', () => {
      // Given two Assignment instances where one has org_id
      const assignment1 = new Assignment({
        scope: 'org',
        role_name: 'user',
        permission_key: 'write',
        org_id: 'org-123',
      })
      const assignment2 = new Assignment({
        scope: 'org',
        role_name: 'user',
        permission_key: 'write',
      })

      // When I compare them
      // Then they should not be equal
      expect(assignment1).not.toEqual(assignment2)
    })

    it('should not be equal when org_id differs', () => {
      // Given two Assignment instances with different org_id
      const assignment1 = new Assignment({
        scope: 'org',
        role_name: 'user',
        permission_key: 'write',
        org_id: 'org-123',
      })
      const assignment2 = new Assignment({
        scope: 'org',
        role_name: 'user',
        permission_key: 'write',
        org_id: 'org-456',
      })

      // When I compare them
      // Then they should not be equal
      expect(assignment1).not.toEqual(assignment2)
    })

    it('should not be equal when one has null org_id and the other has undefined', () => {
      // Given two Assignment instances where one has null org_id
      const assignment1 = new Assignment({
        scope: 'org',
        role_name: 'user',
        permission_key: 'write',
        org_id: null,
      })
      const assignment2 = new Assignment({
        scope: 'org',
        role_name: 'user',
        permission_key: 'write',
      })

      // When I compare them
      // Then they should not be equal (null !== undefined)
      expect(assignment1).not.toEqual(assignment2)
    })

    it('should be equal when both have null org_id', () => {
      // Given two Assignment instances with null org_id
      const assignment1 = new Assignment({
        scope: 'org',
        role_name: 'user',
        permission_key: 'write',
        org_id: null,
      })
      const assignment2 = new Assignment({
        scope: 'org',
        role_name: 'user',
        permission_key: 'write',
        org_id: null,
      })

      // When I compare them
      // Then they should be equal
      expect(assignment1).toEqual(assignment2)
    })
  })

  describe('usage in Permissions service behavior', () => {
    it('should work with Permissions service add method', () => {
      // Given an Assignment instance
      const assignment = new Assignment({
        scope: 'global',
        role_name: 'admin',
        permission_key: 'read',
      })

      // When I use it in Permissions service context
      // Then it should have the correct structure
      // This verifies compatibility with Permissions.add body parameter
      expect(assignment.scope).toBe('global')
      expect(assignment.role_name).toBe('admin')
      expect(assignment.permission_key).toBe('read')
    })

    it('should work with Permissions service delete method', () => {
      // Given an Assignment instance
      const assignment = new Assignment({
        scope: 'org',
        role_name: 'user',
        permission_key: 'write',
        org_id: 'org-123',
      })

      // When I use it in Permissions service context
      // Then it should have the correct structure for matching
      // This verifies compatibility with Permissions.delete body parameter
      expect(assignment.scope).toBe('org')
      expect(assignment.role_name).toBe('user')
      expect(assignment.permission_key).toBe('write')
      expect(assignment.org_id).toBe('org-123')
    })
  })

  describe('scope values behavior', () => {
    it('should accept global scope', () => {
      // Given an Assignment with global scope
      const assignment = new Assignment({
        scope: 'global',
        role_name: 'admin',
        permission_key: 'read',
      })

      // Then scope should be 'global'
      expect(assignment.scope).toBe('global')
    })

    it('should accept org scope', () => {
      // Given an Assignment with org scope
      const assignment = new Assignment({
        scope: 'org',
        role_name: 'user',
        permission_key: 'write',
        org_id: 'org-123',
      })

      // Then scope should be 'org'
      expect(assignment.scope).toBe('org')
    })

    it('should accept any string as scope', () => {
      // Given an Assignment with custom scope
      const assignment = new Assignment({
        scope: 'custom-scope',
        role_name: 'admin',
        permission_key: 'read',
      })

      // Then scope should accept any string
      expect(assignment.scope).toBe('custom-scope')
    })
  })

  describe('edge cases behavior', () => {
    it('should handle very long strings', () => {
      // Given very long string values
      const longString = 'a'.repeat(1000)
      const assignment = new Assignment({
        scope: longString,
        role_name: longString,
        permission_key: longString,
      })

      // Then it should handle long strings
      expect(assignment.scope).toBe(longString)
      expect(assignment.role_name).toBe(longString)
      expect(assignment.permission_key).toBe(longString)
    })

    it('should handle special characters in strings', () => {
      // Given strings with special characters
      const assignment = new Assignment({
        scope: 'scope-with-dashes',
        role_name: 'role_name_with_underscores',
        permission_key: 'permission.key.with.dots',
      })

      // Then it should handle special characters
      expect(assignment.scope).toBe('scope-with-dashes')
      expect(assignment.role_name).toBe('role_name_with_underscores')
      expect(assignment.permission_key).toBe('permission.key.with.dots')
    })

    it('should handle unicode characters', () => {
      // Given strings with unicode characters
      const assignment = new Assignment({
        scope: 'scope-测试',
        role_name: 'role-テスト',
        permission_key: 'permission-тест',
      })

      // Then it should handle unicode characters
      expect(assignment.scope).toBe('scope-测试')
      expect(assignment.role_name).toBe('role-テスト')
      expect(assignment.permission_key).toBe('permission-тест')
    })
  })
})
