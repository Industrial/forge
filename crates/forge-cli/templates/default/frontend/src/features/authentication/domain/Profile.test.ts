/**
 * BDD tests for Profile domain
 * Tests verify the behavior of the Profile Data.TaggedClass
 * Note: Profile is deprecated in favor of Scope, but kept for backwards compatibility
 */

import { describe, it, expect } from 'bun:test'
import { Profile } from './Profile'

describe('Profile domain', () => {
  describe('instance creation behavior', () => {
    it('should create an instance with org_id, org_name, and role', () => {
      // Given profile data
      const orgId = 'org-123'
      const orgName = 'Acme Corp'
      const role = 'owner'

      // When I create a Profile instance
      const profile = new Profile({
        org_id: orgId,
        org_name: orgName,
        role,
      })

      // Then it should have all required properties
      expect(profile.org_id).toBe(orgId)
      expect(profile.org_name).toBe(orgName)
      expect(profile.role).toBe(role)
    })

    it('should create an instance with optional role_id', () => {
      // Given profile data with role_id
      const orgId = 'org-123'
      const orgName = 'Acme Corp'
      const roleId = 'role-456'
      const role = 'viewer'

      // When I create a Profile instance with role_id
      const profile = new Profile({
        org_id: orgId,
        org_name: orgName,
        role_id: roleId,
        role,
      })

      // Then it should include the role_id
      expect(profile.org_id).toBe(orgId)
      expect(profile.org_name).toBe(orgName)
      expect(profile.role_id).toBe(roleId)
      expect(profile.role).toBe(role)
    })

    it('should create an instance without role_id', () => {
      // Given profile data without role_id
      const orgId = 'org-123'
      const orgName = 'Acme Corp'
      const role = 'admin'

      // When I create a Profile instance without role_id
      const profile = new Profile({
        org_id: orgId,
        org_name: orgName,
        role,
      })

      // Then role_id should be undefined
      expect(profile.org_id).toBe(orgId)
      expect(profile.org_name).toBe(orgName)
      expect(profile.role_id).toBeUndefined()
      expect(profile.role).toBe(role)
    })

    it('should accept empty strings for string properties', () => {
      // Given profile data with empty strings
      const profile = new Profile({
        org_id: '',
        org_name: '',
        role: '',
      })

      // Then it should accept empty strings
      expect(profile.org_id).toBe('')
      expect(profile.org_name).toBe('')
      expect(profile.role).toBe('')
    })

    it('should have readonly properties', () => {
      // Given a Profile instance
      const profile = new Profile({
        org_id: 'org-1',
        org_name: 'Test Org',
        role: 'viewer',
      })

      // When I try to modify properties (TypeScript should prevent this)
      // Then the properties should remain unchanged
      // Note: In runtime, readonly doesn't prevent modification, but TypeScript enforces it
      expect(profile.org_id).toBe('org-1')
      expect(profile.org_name).toBe('Test Org')
      expect(profile.role).toBe('viewer')
    })
  })

  describe('structural equality behavior', () => {
    it('should be equal to another instance with same values', () => {
      // Given two Profile instances with the same values
      const profile1 = new Profile({
        org_id: 'org-123',
        org_name: 'Acme Corp',
        role_id: 'role-456',
        role: 'owner',
      })
      const profile2 = new Profile({
        org_id: 'org-123',
        org_name: 'Acme Corp',
        role_id: 'role-456',
        role: 'owner',
      })

      // When I compare them
      // Then they should be structurally equal (Data.TaggedClass provides this)
      expect(profile1).toEqual(profile2)
    })

    it('should be equal when both have undefined role_id', () => {
      // Given two Profile instances without role_id
      const profile1 = new Profile({
        org_id: 'org-123',
        org_name: 'Acme Corp',
        role: 'viewer',
      })
      const profile2 = new Profile({
        org_id: 'org-123',
        org_name: 'Acme Corp',
        role: 'viewer',
      })

      // When I compare them
      // Then they should be equal
      expect(profile1).toEqual(profile2)
    })

    it('should not be equal when org_id differs', () => {
      // Given two Profile instances with different org_ids
      const profile1 = new Profile({
        org_id: 'org-123',
        org_name: 'Acme Corp',
        role: 'owner',
      })
      const profile2 = new Profile({
        org_id: 'org-456',
        org_name: 'Acme Corp',
        role: 'owner',
      })

      // When I compare them
      // Then they should not be equal
      expect(profile1).not.toEqual(profile2)
    })

    it('should not be equal when org_name differs', () => {
      // Given two Profile instances with different org_names
      const profile1 = new Profile({
        org_id: 'org-123',
        org_name: 'Acme Corp',
        role: 'owner',
      })
      const profile2 = new Profile({
        org_id: 'org-123',
        org_name: 'Beta Inc',
        role: 'owner',
      })

      // When I compare them
      // Then they should not be equal
      expect(profile1).not.toEqual(profile2)
    })

    it('should not be equal when role differs', () => {
      // Given two Profile instances with different roles
      const profile1 = new Profile({
        org_id: 'org-123',
        org_name: 'Acme Corp',
        role: 'owner',
      })
      const profile2 = new Profile({
        org_id: 'org-123',
        org_name: 'Acme Corp',
        role: 'viewer',
      })

      // When I compare them
      // Then they should not be equal
      expect(profile1).not.toEqual(profile2)
    })

    it('should not be equal when role_id differs', () => {
      // Given two Profile instances with different role_ids
      const profile1 = new Profile({
        org_id: 'org-123',
        org_name: 'Acme Corp',
        role_id: 'role-1',
        role: 'owner',
      })
      const profile2 = new Profile({
        org_id: 'org-123',
        org_name: 'Acme Corp',
        role_id: 'role-2',
        role: 'owner',
      })

      // When I compare them
      // Then they should not be equal
      expect(profile1).not.toEqual(profile2)
    })

    it('should not be equal when one has role_id and the other does not', () => {
      // Given two Profile instances where one has role_id and the other does not
      const profile1 = new Profile({
        org_id: 'org-123',
        org_name: 'Acme Corp',
        role_id: 'role-456',
        role: 'owner',
      })
      const profile2 = new Profile({
        org_id: 'org-123',
        org_name: 'Acme Corp',
        role: 'owner',
      })

      // When I compare them
      // Then they should not be equal
      expect(profile1).not.toEqual(profile2)
    })
  })

  describe('tagged class behavior', () => {
    it('should have the Profile tag', () => {
      // Given a Profile instance
      const profile = new Profile({
        org_id: 'org-1',
        org_name: 'Test Org',
        role: 'viewer',
      })

      // When I check the tag
      // Then it should be tagged as 'Profile'
      // Data.TaggedClass provides a _tag property
      expect((profile as any)._tag).toBe('Profile')
    })

    it('should be identifiable as Profile type', () => {
      // Given a Profile instance
      const profile = new Profile({
        org_id: 'org-1',
        org_name: 'Test Org',
        role: 'viewer',
      })

      // When I check the instance
      // Then it should be an instance of Profile
      expect(profile).toBeInstanceOf(Profile)
    })
  })

  describe('property access behavior', () => {
    it('should allow reading org_id property', () => {
      // Given a Profile instance
      const profile = new Profile({
        org_id: 'org-123',
        org_name: 'Acme Corp',
        role: 'owner',
      })

      // When I read the org_id property
      const orgId = profile.org_id

      // Then it should return the correct value
      expect(orgId).toBe('org-123')
    })

    it('should allow reading org_name property', () => {
      // Given a Profile instance
      const profile = new Profile({
        org_id: 'org-123',
        org_name: 'Acme Corporation',
        role: 'owner',
      })

      // When I read the org_name property
      const orgName = profile.org_name

      // Then it should return the correct value
      expect(orgName).toBe('Acme Corporation')
    })

    it('should allow reading role property', () => {
      // Given a Profile instance
      const profile = new Profile({
        org_id: 'org-123',
        org_name: 'Acme Corp',
        role: 'viewer',
      })

      // When I read the role property
      const role = profile.role

      // Then it should return the correct value
      expect(role).toBe('viewer')
    })

    it('should allow reading role_id property when present', () => {
      // Given a Profile instance with role_id
      const profile = new Profile({
        org_id: 'org-123',
        org_name: 'Acme Corp',
        role_id: 'role-789',
        role: 'admin',
      })

      // When I read the role_id property
      const roleId = profile.role_id

      // Then it should return the correct value
      expect(roleId).toBe('role-789')
    })

    it('should return undefined for role_id when not provided', () => {
      // Given a Profile instance without role_id
      const profile = new Profile({
        org_id: 'org-123',
        org_name: 'Acme Corp',
        role: 'admin',
      })

      // When I read the role_id property
      const roleId = profile.role_id

      // Then it should be undefined
      expect(roleId).toBeUndefined()
    })
  })

  describe('usage in organization and role context', () => {
    it('should represent an organization and role pair', () => {
      // Given organization and role data
      const orgId = '550e8400-e29b-41d4-a716-446655440000'
      const orgName = 'Engineering Team'
      const roleId = '123e4567-e89b-12d3-a456-426614174000'
      const role = 'developer'

      // When I create a Profile
      const profile = new Profile({
        org_id: orgId,
        org_name: orgName,
        role_id: roleId,
        role,
      })

      // Then it should represent a complete org + role pair
      expect(profile.org_id).toBe(orgId)
      expect(profile.org_name).toBe(orgName)
      expect(profile.role_id).toBe(roleId)
      expect(profile.role).toBe(role)
    })

    it('should work with UUID strings as org_id', () => {
      // Given a UUID string as org_id
      const uuid = '123e4567-e89b-12d3-a456-426614174000'

      // When I create a Profile with UUID org_id
      const profile = new Profile({
        org_id: uuid,
        org_name: 'Test Org',
        role: 'owner',
      })

      // Then it should accept and store the UUID
      expect(profile.org_id).toBe(uuid)
    })

    it('should work with common role names', () => {
      // Given common role names
      const roles = ['owner', 'admin', 'viewer', 'editor', 'developer']

      // When I create Profiles with these roles
      const profiles = roles.map((role) =>
        new Profile({
          org_id: 'org-1',
          org_name: 'Test Org',
          role,
        }),
      )

      // Then each should have the correct role
      profiles.forEach((profile, index) => {
        expect(profile.role).toBe(roles[index])
      })
    })
  })
})
