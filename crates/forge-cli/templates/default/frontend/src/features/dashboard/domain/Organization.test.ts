/**
 * BDD tests for Organization domain class
 * Tests verify the behavior of the Organization Data.TaggedClass
 */

import { describe, it, expect } from 'bun:test'
import { Organization } from './Organization'

describe('Organization', () => {
  describe('instance creation behavior', () => {
    it('should create an instance with required fields', () => {
      // Given required fields
      const id = 'org-123'
      const name = 'Acme Corporation'
      const slug = 'acme-corp'

      // When I create an Organization instance
      const organization = new Organization({
        id,
        name,
        slug,
      })

      // Then it should have the correct properties
      expect(organization.id).toBe(id)
      expect(organization.name).toBe(name)
      expect(organization.slug).toBe(slug)
    })

    it('should create an instance with optional created_at', () => {
      // Given all fields including created_at
      const organization = new Organization({
        id: 'org-123',
        name: 'Acme Corporation',
        slug: 'acme-corp',
        created_at: '2024-01-01T00:00:00Z',
      })

      // Then it should have created_at set
      expect(organization.created_at).toBe('2024-01-01T00:00:00Z')
    })

    it('should create an instance with optional updated_at', () => {
      // Given all fields including updated_at
      const organization = new Organization({
        id: 'org-123',
        name: 'Acme Corporation',
        slug: 'acme-corp',
        updated_at: '2024-01-02T00:00:00Z',
      })

      // Then it should have updated_at set
      expect(organization.updated_at).toBe('2024-01-02T00:00:00Z')
    })

    it('should create an instance with both optional timestamps', () => {
      // Given all fields including both timestamps
      const organization = new Organization({
        id: 'org-123',
        name: 'Acme Corporation',
        slug: 'acme-corp',
        created_at: '2024-01-01T00:00:00Z',
        updated_at: '2024-01-02T00:00:00Z',
      })

      // Then it should have both timestamps set
      expect(organization.created_at).toBe('2024-01-01T00:00:00Z')
      expect(organization.updated_at).toBe('2024-01-02T00:00:00Z')
    })

    it('should create an instance without optional fields', () => {
      // Given fields without optional timestamps
      const organization = new Organization({
        id: 'org-123',
        name: 'Acme Corporation',
        slug: 'acme-corp',
      })

      // Then optional fields should be undefined
      expect(organization.created_at).toBeUndefined()
      expect(organization.updated_at).toBeUndefined()
    })

    it('should accept empty strings for required fields', () => {
      // Given empty string values
      const organization = new Organization({
        id: '',
        name: '',
        slug: '',
      })

      // Then it should accept empty strings
      expect(organization.id).toBe('')
      expect(organization.name).toBe('')
      expect(organization.slug).toBe('')
    })
  })

  describe('property access behavior', () => {
    it('should allow reading id property', () => {
      // Given an Organization instance
      const organization = new Organization({
        id: 'org-123',
        name: 'Acme Corporation',
        slug: 'acme-corp',
      })

      // When I access the id property
      // Then it should return the value
      expect(organization.id).toBe('org-123')
    })

    it('should allow reading name property', () => {
      // Given an Organization instance
      const organization = new Organization({
        id: 'org-123',
        name: 'Acme Corporation',
        slug: 'acme-corp',
      })

      // When I access the name property
      // Then it should return the value
      expect(organization.name).toBe('Acme Corporation')
    })

    it('should allow reading slug property', () => {
      // Given an Organization instance
      const organization = new Organization({
        id: 'org-123',
        name: 'Acme Corporation',
        slug: 'acme-corp',
      })

      // When I access the slug property
      // Then it should return the value
      expect(organization.slug).toBe('acme-corp')
    })

    it('should allow reading created_at property when set', () => {
      // Given an Organization instance with created_at
      const organization = new Organization({
        id: 'org-123',
        name: 'Acme Corporation',
        slug: 'acme-corp',
        created_at: '2024-01-01T00:00:00Z',
      })

      // When I access the created_at property
      // Then it should return the value
      expect(organization.created_at).toBe('2024-01-01T00:00:00Z')
    })

    it('should return undefined for created_at when not set', () => {
      // Given an Organization instance without created_at
      const organization = new Organization({
        id: 'org-123',
        name: 'Acme Corporation',
        slug: 'acme-corp',
      })

      // When I access the created_at property
      // Then it should return undefined
      expect(organization.created_at).toBeUndefined()
    })

    it('should allow reading updated_at property when set', () => {
      // Given an Organization instance with updated_at
      const organization = new Organization({
        id: 'org-123',
        name: 'Acme Corporation',
        slug: 'acme-corp',
        updated_at: '2024-01-02T00:00:00Z',
      })

      // When I access the updated_at property
      // Then it should return the value
      expect(organization.updated_at).toBe('2024-01-02T00:00:00Z')
    })

    it('should return undefined for updated_at when not set', () => {
      // Given an Organization instance without updated_at
      const organization = new Organization({
        id: 'org-123',
        name: 'Acme Corporation',
        slug: 'acme-corp',
      })

      // When I access the updated_at property
      // Then it should return undefined
      expect(organization.updated_at).toBeUndefined()
    })

    it('should have readonly properties', () => {
      // Given an Organization instance
      const organization = new Organization({
        id: 'org-123',
        name: 'Acme Corporation',
        slug: 'acme-corp',
      })

      // When I check the properties
      // Then they should be readonly (enforced by TypeScript)
      // Runtime check: properties exist and are accessible
      expect(organization.id).toBeDefined()
      expect(organization.name).toBeDefined()
      expect(organization.slug).toBeDefined()
    })
  })

  describe('tagged class behavior', () => {
    it('should have the Organization tag', () => {
      // Given an Organization instance
      const organization = new Organization({
        id: 'org-123',
        name: 'Acme Corporation',
        slug: 'acme-corp',
      })

      // When I check the tag
      // Then it should be tagged as 'Organization'
      // Data.TaggedClass provides a _tag property
      expect((organization as any)._tag).toBe('Organization')
    })

    it('should be identifiable as Organization type', () => {
      // Given an Organization instance
      const organization = new Organization({
        id: 'org-123',
        name: 'Acme Corporation',
        slug: 'acme-corp',
      })

      // When I check the instance
      // Then it should be an instance of Organization
      expect(organization).toBeInstanceOf(Organization)
    })
  })

  describe('structural equality behavior', () => {
    it('should be equal when all fields match', () => {
      // Given two Organization instances with same data
      const organization1 = new Organization({
        id: 'org-123',
        name: 'Acme Corporation',
        slug: 'acme-corp',
      })
      const organization2 = new Organization({
        id: 'org-123',
        name: 'Acme Corporation',
        slug: 'acme-corp',
      })

      // When I compare them
      // Then they should be structurally equal
      // Effect's Data.TaggedClass provides structural equality
      expect(organization1).toEqual(organization2)
    })

    it('should be equal when optional fields match', () => {
      // Given two Organization instances with same data including timestamps
      const organization1 = new Organization({
        id: 'org-123',
        name: 'Acme Corporation',
        slug: 'acme-corp',
        created_at: '2024-01-01T00:00:00Z',
        updated_at: '2024-01-02T00:00:00Z',
      })
      const organization2 = new Organization({
        id: 'org-123',
        name: 'Acme Corporation',
        slug: 'acme-corp',
        created_at: '2024-01-01T00:00:00Z',
        updated_at: '2024-01-02T00:00:00Z',
      })

      // When I compare them
      // Then they should be equal
      expect(organization1).toEqual(organization2)
    })

    it('should not be equal when id differs', () => {
      // Given two Organization instances with different id
      const organization1 = new Organization({
        id: 'org-123',
        name: 'Acme Corporation',
        slug: 'acme-corp',
      })
      const organization2 = new Organization({
        id: 'org-456',
        name: 'Acme Corporation',
        slug: 'acme-corp',
      })

      // When I compare them
      // Then they should not be equal
      expect(organization1).not.toEqual(organization2)
    })

    it('should not be equal when name differs', () => {
      // Given two Organization instances with different name
      const organization1 = new Organization({
        id: 'org-123',
        name: 'Acme Corporation',
        slug: 'acme-corp',
      })
      const organization2 = new Organization({
        id: 'org-123',
        name: 'Different Name',
        slug: 'acme-corp',
      })

      // When I compare them
      // Then they should not be equal
      expect(organization1).not.toEqual(organization2)
    })

    it('should not be equal when slug differs', () => {
      // Given two Organization instances with different slug
      const organization1 = new Organization({
        id: 'org-123',
        name: 'Acme Corporation',
        slug: 'acme-corp',
      })
      const organization2 = new Organization({
        id: 'org-123',
        name: 'Acme Corporation',
        slug: 'different-slug',
      })

      // When I compare them
      // Then they should not be equal
      expect(organization1).not.toEqual(organization2)
    })

    it('should not be equal when one has created_at and the other does not', () => {
      // Given two Organization instances where one has created_at
      const organization1 = new Organization({
        id: 'org-123',
        name: 'Acme Corporation',
        slug: 'acme-corp',
        created_at: '2024-01-01T00:00:00Z',
      })
      const organization2 = new Organization({
        id: 'org-123',
        name: 'Acme Corporation',
        slug: 'acme-corp',
      })

      // When I compare them
      // Then they should not be equal
      expect(organization1).not.toEqual(organization2)
    })

    it('should not be equal when created_at differs', () => {
      // Given two Organization instances with different created_at
      const organization1 = new Organization({
        id: 'org-123',
        name: 'Acme Corporation',
        slug: 'acme-corp',
        created_at: '2024-01-01T00:00:00Z',
      })
      const organization2 = new Organization({
        id: 'org-123',
        name: 'Acme Corporation',
        slug: 'acme-corp',
        created_at: '2024-01-02T00:00:00Z',
      })

      // When I compare them
      // Then they should not be equal
      expect(organization1).not.toEqual(organization2)
    })

    it('should not be equal when one has updated_at and the other does not', () => {
      // Given two Organization instances where one has updated_at
      const organization1 = new Organization({
        id: 'org-123',
        name: 'Acme Corporation',
        slug: 'acme-corp',
        updated_at: '2024-01-02T00:00:00Z',
      })
      const organization2 = new Organization({
        id: 'org-123',
        name: 'Acme Corporation',
        slug: 'acme-corp',
      })

      // When I compare them
      // Then they should not be equal
      expect(organization1).not.toEqual(organization2)
    })

    it('should not be equal when updated_at differs', () => {
      // Given two Organization instances with different updated_at
      const organization1 = new Organization({
        id: 'org-123',
        name: 'Acme Corporation',
        slug: 'acme-corp',
        updated_at: '2024-01-01T00:00:00Z',
      })
      const organization2 = new Organization({
        id: 'org-123',
        name: 'Acme Corporation',
        slug: 'acme-corp',
        updated_at: '2024-01-02T00:00:00Z',
      })

      // When I compare them
      // Then they should not be equal
      expect(organization1).not.toEqual(organization2)
    })
  })

  describe('usage in Dashboard service behavior', () => {
    it('should work with Dashboard service getOrganizations method', () => {
      // Given an Organization instance
      const organization = new Organization({
        id: 'org-123',
        name: 'Acme Corporation',
        slug: 'acme-corp',
      })

      // When I use it in Dashboard service context
      // Then it should have the correct structure
      // This verifies compatibility with Dashboard.getOrganizations return type
      expect(organization.id).toBe('org-123')
      expect(organization.name).toBe('Acme Corporation')
      expect(organization.slug).toBe('acme-corp')
    })
  })

  describe('usage in EntityApi behavior', () => {
    it('should work with EntityApi list method', () => {
      // Given an Organization instance
      const organization = new Organization({
        id: 'org-123',
        name: 'Acme Corporation',
        slug: 'acme-corp',
        created_at: '2024-01-01T00:00:00Z',
        updated_at: '2024-01-02T00:00:00Z',
      })

      // When I use it in EntityApi context
      // Then it should have the correct structure
      // This verifies compatibility with EntityApi list response
      expect(organization.id).toBe('org-123')
      expect(organization.name).toBe('Acme Corporation')
      expect(organization.slug).toBe('acme-corp')
    })
  })

  describe('slug format behavior', () => {
    it('should accept lowercase slugs', () => {
      // Given an Organization with lowercase slug
      const organization = new Organization({
        id: 'org-123',
        name: 'Acme Corporation',
        slug: 'acme-corp',
      })

      // Then slug should be accepted
      expect(organization.slug).toBe('acme-corp')
    })

    it('should accept slugs with hyphens', () => {
      // Given an Organization with hyphenated slug
      const organization = new Organization({
        id: 'org-123',
        name: 'Acme Corporation',
        slug: 'acme-corporation-inc',
      })

      // Then slug should be accepted
      expect(organization.slug).toBe('acme-corporation-inc')
    })

    it('should accept slugs with numbers', () => {
      // Given an Organization with numeric slug
      const organization = new Organization({
        id: 'org-123',
        name: 'Acme Corporation',
        slug: 'acme-corp-2024',
      })

      // Then slug should be accepted
      expect(organization.slug).toBe('acme-corp-2024')
    })

    it('should accept any string as slug', () => {
      // Given an Organization with custom slug format
      const organization = new Organization({
        id: 'org-123',
        name: 'Acme Corporation',
        slug: 'custom_slug_format',
      })

      // Then slug should accept any string
      expect(organization.slug).toBe('custom_slug_format')
    })
  })

  describe('timestamp format behavior', () => {
    it('should accept ISO 8601 timestamp strings', () => {
      // Given an Organization with ISO timestamp
      const organization = new Organization({
        id: 'org-123',
        name: 'Acme Corporation',
        slug: 'acme-corp',
        created_at: '2024-01-01T00:00:00Z',
      })

      // Then timestamp should be accepted
      expect(organization.created_at).toBe('2024-01-01T00:00:00Z')
    })

    it('should accept timestamps with timezone offset', () => {
      // Given an Organization with timestamp with offset
      const organization = new Organization({
        id: 'org-123',
        name: 'Acme Corporation',
        slug: 'acme-corp',
        created_at: '2024-01-01T00:00:00+00:00',
      })

      // Then timestamp should be accepted
      expect(organization.created_at).toBe('2024-01-01T00:00:00+00:00')
    })

    it('should accept any string as timestamp', () => {
      // Given an Organization with custom timestamp format
      const organization = new Organization({
        id: 'org-123',
        name: 'Acme Corporation',
        slug: 'acme-corp',
        created_at: '2024-01-01',
      })

      // Then timestamp should accept any string
      expect(organization.created_at).toBe('2024-01-01')
    })
  })

  describe('edge cases behavior', () => {
    it('should handle very long strings', () => {
      // Given very long string values
      const longString = 'a'.repeat(1000)
      const organization = new Organization({
        id: longString,
        name: longString,
        slug: longString,
      })

      // Then it should handle long strings
      expect(organization.id).toBe(longString)
      expect(organization.name).toBe(longString)
      expect(organization.slug).toBe(longString)
    })

    it('should handle special characters in strings', () => {
      // Given strings with special characters
      const organization = new Organization({
        id: 'org-123',
        name: 'Acme & Co., Inc.',
        slug: 'acme-co-inc',
      })

      // Then it should handle special characters
      expect(organization.name).toBe('Acme & Co., Inc.')
      expect(organization.slug).toBe('acme-co-inc')
    })

    it('should handle unicode characters', () => {
      // Given strings with unicode characters
      const organization = new Organization({
        id: 'org-123',
        name: 'Acme 测试公司',
        slug: 'acme-test',
      })

      // Then it should handle unicode characters
      expect(organization.name).toBe('Acme 测试公司')
      expect(organization.slug).toBe('acme-test')
    })
  })
})
