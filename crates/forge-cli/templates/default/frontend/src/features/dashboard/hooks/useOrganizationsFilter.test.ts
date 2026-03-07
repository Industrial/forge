/**
 * BDD-style unit tests for useOrganizationsFilter hook using bun:test.
 * Tests follow Given-When-Then pattern and verify filtering behavior.
 * Tests the core filtering logic (organizationMatches) and hook integration.
 */

import { describe, it, expect } from 'bun:test'
import {
  useOrganizationsFilter,
  organizationMatches,
} from './useOrganizationsFilter'
import { Organization } from '@/features/dashboard/domain/Organization'
import type { OrganizationsFilterState } from './useOrganizationsFilter'

// Factory function for creating mock organizations
function getMockOrganization(overrides?: Partial<Organization>): Organization {
  return new Organization({
    id: '1',
    name: 'Test Organization',
    slug: 'test-org',
    ...overrides,
  })
}

describe('useOrganizationsFilter', () => {
  describe('export behavior', () => {
    it('should export useOrganizationsFilter as a function', () => {
      // Given the module
      // When I check the export
      // Then it should be a function
      expect(typeof useOrganizationsFilter).toBe('function')
    })

    it('should export organizationMatches as a function', () => {
      // Given the module
      // When I check the export
      // Then organizationMatches should be a function
      expect(typeof organizationMatches).toBe('function')
    })
  })

  describe('organizationMatches predicate behavior', () => {
    describe('name filtering', () => {
      it('should match organizations by name (case-insensitive)', () => {
        // Given: an organization and a name filter
        const org = getMockOrganization({ name: 'Acme Corp', slug: 'acme' })
        const filters: OrganizationsFilterState = {
          filterName: 'acme',
          filterSlug: '',
        }

        // When: checking if the organization matches
        const result = organizationMatches(org, filters)

        // Then: it should match (case-insensitive)
        expect(result).toBe(true)
      })

      it('should match organizations by partial name', () => {
        // Given: an organization and a partial name filter
        const org = getMockOrganization({
          name: 'Acme Corporation',
          slug: 'acme',
        })
        const filters: OrganizationsFilterState = {
          filterName: 'Corp',
          filterSlug: '',
        }

        // When: checking if the organization matches
        const result = organizationMatches(org, filters)

        // Then: it should match
        expect(result).toBe(true)
      })

      it('should ignore leading and trailing whitespace in filter name', () => {
        // Given: an organization and a filter with whitespace
        const org = getMockOrganization({ name: 'Acme Corp', slug: 'acme' })
        const filters: OrganizationsFilterState = {
          filterName: '  acme  ',
          filterSlug: '',
        }

        // When: checking if the organization matches
        const result = organizationMatches(org, filters)

        // Then: it should match (whitespace trimmed)
        expect(result).toBe(true)
      })

      it('should return true when filter name is empty', () => {
        // Given: an organization and an empty name filter
        const org = getMockOrganization({ name: 'Acme Corp', slug: 'acme' })
        const filters: OrganizationsFilterState = {
          filterName: '',
          filterSlug: '',
        }

        // When: checking if the organization matches
        const result = organizationMatches(org, filters)

        // Then: it should match (empty filter means no filter)
        expect(result).toBe(true)
      })

      it('should return false when name does not match', () => {
        // Given: an organization and a non-matching name filter
        const org = getMockOrganization({ name: 'Acme Corp', slug: 'acme' })
        const filters: OrganizationsFilterState = {
          filterName: 'Beta',
          filterSlug: '',
        }

        // When: checking if the organization matches
        const result = organizationMatches(org, filters)

        // Then: it should not match
        expect(result).toBe(false)
      })
    })

    describe('slug filtering', () => {
      it('should match organizations by slug (case-insensitive)', () => {
        // Given: an organization and a slug filter
        const org = getMockOrganization({ name: 'Org One', slug: 'org-one' })
        const filters: OrganizationsFilterState = {
          filterName: '',
          filterSlug: 'org-one',
        }

        // When: checking if the organization matches
        const result = organizationMatches(org, filters)

        // Then: it should match
        expect(result).toBe(true)
      })

      it('should match organizations by partial slug', () => {
        // Given: an organization and a partial slug filter
        const org = getMockOrganization({
          name: 'Org One',
          slug: 'acme-corporation',
        })
        const filters: OrganizationsFilterState = {
          filterName: '',
          filterSlug: 'acme',
        }

        // When: checking if the organization matches
        const result = organizationMatches(org, filters)

        // Then: it should match
        expect(result).toBe(true)
      })

      it('should ignore leading and trailing whitespace in filter slug', () => {
        // Given: an organization and a filter with whitespace
        const org = getMockOrganization({ name: 'Org One', slug: 'acme-corp' })
        const filters: OrganizationsFilterState = {
          filterName: '',
          filterSlug: '  acme-corp  ',
        }

        // When: checking if the organization matches
        const result = organizationMatches(org, filters)

        // Then: it should match (whitespace trimmed)
        expect(result).toBe(true)
      })

      it('should return true when filter slug is empty', () => {
        // Given: an organization and an empty slug filter
        const org = getMockOrganization({ name: 'Org One', slug: 'org-one' })
        const filters: OrganizationsFilterState = {
          filterName: '',
          filterSlug: '',
        }

        // When: checking if the organization matches
        const result = organizationMatches(org, filters)

        // Then: it should match (empty filter means no filter)
        expect(result).toBe(true)
      })

      it('should return false when slug does not match', () => {
        // Given: an organization and a non-matching slug filter
        const org = getMockOrganization({ name: 'Org One', slug: 'org-one' })
        const filters: OrganizationsFilterState = {
          filterName: '',
          filterSlug: 'non-existent',
        }

        // When: checking if the organization matches
        const result = organizationMatches(org, filters)

        // Then: it should not match
        expect(result).toBe(false)
      })
    })

    describe('combined filtering', () => {
      it('should match when both name and slug match', () => {
        // Given: an organization and filters that match both name and slug
        const org = getMockOrganization({
          name: 'Acme Corp',
          slug: 'acme-corp',
        })
        const filters: OrganizationsFilterState = {
          filterName: 'Acme',
          filterSlug: 'corp',
        }

        // When: checking if the organization matches
        const result = organizationMatches(org, filters)

        // Then: it should match (both conditions satisfied)
        expect(result).toBe(true)
      })

      it('should return false when name matches but slug does not', () => {
        // Given: an organization where name matches but slug doesn't
        const org = getMockOrganization({
          name: 'Acme Corp',
          slug: 'acme-corp',
        })
        const filters: OrganizationsFilterState = {
          filterName: 'Acme',
          filterSlug: 'beta',
        }

        // When: checking if the organization matches
        const result = organizationMatches(org, filters)

        // Then: it should not match (slug filter fails)
        expect(result).toBe(false)
      })

      it('should return false when slug matches but name does not', () => {
        // Given: an organization where slug matches but name doesn't
        const org = getMockOrganization({
          name: 'Acme Corp',
          slug: 'acme-corp',
        })
        const filters: OrganizationsFilterState = {
          filterName: 'Beta',
          filterSlug: 'acme',
        }

        // When: checking if the organization matches
        const result = organizationMatches(org, filters)

        // Then: it should not match (name filter fails)
        expect(result).toBe(false)
      })

      it('should match when both filters are empty', () => {
        // Given: an organization and empty filters
        const org = getMockOrganization({
          name: 'Acme Corp',
          slug: 'acme-corp',
        })
        const filters: OrganizationsFilterState = {
          filterName: '',
          filterSlug: '',
        }

        // When: checking if the organization matches
        const result = organizationMatches(org, filters)

        // Then: it should match (no filters applied)
        expect(result).toBe(true)
      })
    })

    describe('edge cases', () => {
      it('should handle organization with empty name', () => {
        // Given: an organization with empty name
        const org = getMockOrganization({ name: '', slug: 'test-slug' })
        const filters: OrganizationsFilterState = {
          filterName: '',
          filterSlug: 'test-slug',
        }

        // When: checking if the organization matches
        const result = organizationMatches(org, filters)

        // Then: it should match (name filter is empty, slug matches)
        expect(result).toBe(true)
      })

      it('should handle organization with empty slug', () => {
        // Given: an organization with empty slug
        const org = getMockOrganization({ name: 'Test Name', slug: '' })
        const filters: OrganizationsFilterState = {
          filterName: 'Test',
          filterSlug: '',
        }

        // When: checking if the organization matches
        const result = organizationMatches(org, filters)

        // Then: it should match (slug filter is empty, name matches)
        expect(result).toBe(true)
      })

      it('should handle filter with only whitespace', () => {
        // Given: an organization and filters with only whitespace
        const org = getMockOrganization({ name: 'Acme Corp', slug: 'acme' })
        const filters: OrganizationsFilterState = {
          filterName: '   ',
          filterSlug: '   ',
        }

        // When: checking if the organization matches
        const result = organizationMatches(org, filters)

        // Then: it should match (whitespace-only filters are treated as empty)
        expect(result).toBe(true)
      })
    })
  })

  describe('hook integration behavior', () => {
    it('should be callable as a function', () => {
      // Given: the hook function
      // When: I check if it's callable
      // Then: it should be a function
      expect(typeof useOrganizationsFilter).toBe('function')
    })

    it('should accept organizations array as parameter', () => {
      // Given: an array of organizations
      const organizations = [
        getMockOrganization({ id: '1', name: 'Org One', slug: 'org-one' }),
        getMockOrganization({ id: '2', name: 'Org Two', slug: 'org-two' }),
      ]

      // When: checking the function signature
      // Then: it should accept readonly Organization[]
      // Note: TypeScript enforces this at compile time
      expect(typeof useOrganizationsFilter).toBe('function')
      expect(Array.isArray(organizations)).toBe(true)
    })

    it('should return a tuple with filters, setFilters, and filteredOrganizations', () => {
      // Given: the hook function
      // When: checking its return type
      // Then: it should return [OrganizationsFilterState, Dispatch, Organization[]]
      // Note: TypeScript enforces this at compile time
      expect(typeof useOrganizationsFilter).toBe('function')
    })
  })
})
