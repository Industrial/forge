/**
 * BDD tests for membershipsSummary utility function
 */
import { describe, test, expect } from 'bun:test'
import { membershipsSummary, type MembershipLike } from './membershipsSummary'

describe('membershipsSummary', () => {
  describe('empty array handling', () => {
    test('should return "—" for empty array', () => {
      // Given: empty memberships array
      const memberships: readonly MembershipLike[] = []

      // When: generating summary
      const result = membershipsSummary(memberships)

      // Then: should return "—"
      expect(result).toBe('—')
    })
  })

  describe('single membership formatting', () => {
    test('should format membership with roles', () => {
      // Given: single membership with roles
      const memberships: readonly MembershipLike[] = [
        {
          org_name: 'Acme Corp',
          roles: ['admin', 'user'],
        },
      ]

      // When: generating summary
      const result = membershipsSummary(memberships)

      // Then: should format as "org_name: role1, role2"
      expect(result).toBe('Acme Corp: admin, user')
    })

    test('should format membership with single role', () => {
      // Given: single membership with one role
      const memberships: readonly MembershipLike[] = [
        {
          org_name: 'Tech Inc',
          roles: ['admin'],
        },
      ]

      // When: generating summary
      const result = membershipsSummary(memberships)

      // Then: should format as "org_name: role"
      expect(result).toBe('Tech Inc: admin')
    })

    test('should format membership with null roles', () => {
      // Given: single membership with null roles
      const memberships: readonly MembershipLike[] = [
        {
          org_name: 'Org Name',
          roles: null,
        },
      ]

      // When: generating summary
      const result = membershipsSummary(memberships)

      // Then: should format as "org_name: —"
      expect(result).toBe('Org Name: —')
    })

    test('should format membership with undefined roles', () => {
      // Given: single membership with undefined roles
      const memberships: readonly MembershipLike[] = [
        {
          org_name: 'Org Name',
          roles: undefined,
        },
      ]

      // When: generating summary
      const result = membershipsSummary(memberships)

      // Then: should format as "org_name: —"
      expect(result).toBe('Org Name: —')
    })

    test('should format membership with empty roles array', () => {
      // Given: single membership with empty roles array
      const memberships: readonly MembershipLike[] = [
        {
          org_name: 'Org Name',
          roles: [],
        },
      ]

      // When: generating summary
      const result = membershipsSummary(memberships)

      // Then: should format as "org_name: —"
      expect(result).toBe('Org Name: —')
    })

    test('should format membership without roles property', () => {
      // Given: single membership without roles property
      const memberships: readonly MembershipLike[] = [
        {
          org_name: 'Org Name',
        },
      ]

      // When: generating summary
      const result = membershipsSummary(memberships)

      // Then: should format as "org_name: —"
      expect(result).toBe('Org Name: —')
    })
  })

  describe('multiple memberships formatting', () => {
    test('should format multiple memberships with roles', () => {
      // Given: multiple memberships with roles
      const memberships: readonly MembershipLike[] = [
        {
          org_name: 'Acme Corp',
          roles: ['admin', 'user'],
        },
        {
          org_name: 'Tech Inc',
          roles: ['viewer'],
        },
      ]

      // When: generating summary
      const result = membershipsSummary(memberships)

      // Then: should join with "; "
      expect(result).toBe('Acme Corp: admin, user; Tech Inc: viewer')
    })

    test('should format multiple memberships with mixed roles', () => {
      // Given: multiple memberships with mixed roles (some null, some with roles)
      const memberships: readonly MembershipLike[] = [
        {
          org_name: 'Org A',
          roles: ['admin'],
        },
        {
          org_name: 'Org B',
          roles: null,
        },
        {
          org_name: 'Org C',
          roles: ['user', 'viewer'],
        },
      ]

      // When: generating summary
      const result = membershipsSummary(memberships)

      // Then: should format all correctly
      expect(result).toBe('Org A: admin; Org B: —; Org C: user, viewer')
    })

    test('should format multiple memberships with empty roles', () => {
      // Given: multiple memberships with empty roles arrays
      const memberships: readonly MembershipLike[] = [
        {
          org_name: 'Org A',
          roles: [],
        },
        {
          org_name: 'Org B',
          roles: [],
        },
      ]

      // When: generating summary
      const result = membershipsSummary(memberships)

      // Then: should format as "—" for roles
      expect(result).toBe('Org A: —; Org B: —')
    })

    test('should handle many memberships', () => {
      // Given: many memberships
      const memberships: readonly MembershipLike[] = [
        { org_name: 'Org 1', roles: ['role1'] },
        { org_name: 'Org 2', roles: ['role2', 'role3'] },
        { org_name: 'Org 3', roles: null },
        { org_name: 'Org 4', roles: ['role4'] },
        { org_name: 'Org 5', roles: [] },
      ]

      // When: generating summary
      const result = membershipsSummary(memberships)

      // Then: should format all correctly
      expect(result).toBe(
        'Org 1: role1; Org 2: role2, role3; Org 3: —; Org 4: role4; Org 5: —',
      )
    })
  })

  describe('edge cases', () => {
    test('should handle roles with special characters', () => {
      // Given: membership with roles containing special characters
      const memberships: readonly MembershipLike[] = [
        {
          org_name: 'Org Name',
          roles: ['role-with-dash', 'role_with_underscore', 'role.with.dot'],
        },
      ]

      // When: generating summary
      const result = membershipsSummary(memberships)

      // Then: should include special characters
      expect(result).toBe(
        'Org Name: role-with-dash, role_with_underscore, role.with.dot',
      )
    })

    test('should handle org_name with special characters', () => {
      // Given: membership with org_name containing special characters
      const memberships: readonly MembershipLike[] = [
        {
          org_name: "O'Reilly & Associates",
          roles: ['admin'],
        },
      ]

      // When: generating summary
      const result = membershipsSummary(memberships)

      // Then: should include special characters
      expect(result).toBe("O'Reilly & Associates: admin")
    })

    test('should handle very long role names', () => {
      // Given: membership with very long role names
      const memberships: readonly MembershipLike[] = [
        {
          org_name: 'Org',
          roles: [
            'very-long-role-name-that-might-be-very-descriptive',
            'another-very-long-role-name',
          ],
        },
      ]

      // When: generating summary
      const result = membershipsSummary(memberships)

      // Then: should include full role names
      expect(result).toBe(
        'Org: very-long-role-name-that-might-be-very-descriptive, another-very-long-role-name',
      )
    })

    test('should handle many roles in single membership', () => {
      // Given: membership with many roles
      const memberships: readonly MembershipLike[] = [
        {
          org_name: 'Org',
          roles: ['role1', 'role2', 'role3', 'role4', 'role5'],
        },
      ]

      // When: generating summary
      const result = membershipsSummary(memberships)

      // Then: should include all roles
      expect(result).toBe('Org: role1, role2, role3, role4, role5')
    })

    test('should handle empty string in roles array', () => {
      // Given: membership with empty string in roles
      const memberships: readonly MembershipLike[] = [
        {
          org_name: 'Org',
          roles: ['', 'role2'],
        },
      ]

      // When: generating summary
      const result = membershipsSummary(memberships)

      // Then: should include empty string
      expect(result).toBe('Org: , role2')
    })

    test('should handle whitespace in roles', () => {
      // Given: membership with whitespace in roles
      const memberships: readonly MembershipLike[] = [
        {
          org_name: 'Org',
          roles: ['  role1  ', 'role2'],
        },
      ]

      // When: generating summary
      const result = membershipsSummary(memberships)

      // Then: should include whitespace
      expect(result).toBe('Org:   role1  , role2')
    })
  })

  describe('format consistency', () => {
    test('should produce consistent output for same input', () => {
      // Given: same memberships array
      const memberships: readonly MembershipLike[] = [
        {
          org_name: 'Org A',
          roles: ['admin'],
        },
        {
          org_name: 'Org B',
          roles: ['user'],
        },
      ]

      // When: generating summary multiple times
      const result1 = membershipsSummary(memberships)
      const result2 = membershipsSummary(memberships)
      const result3 = membershipsSummary(memberships)

      // Then: should produce same output
      expect(result1).toBe(result2)
      expect(result2).toBe(result3)
      expect(result1).toBe('Org A: admin; Org B: user')
    })

    test('should handle readonly arrays', () => {
      // Given: readonly array (as const)
      const memberships = [
        {
          org_name: 'Org',
          roles: ['admin'] as const,
        },
      ] as const

      // When: generating summary
      const result = membershipsSummary(memberships)

      // Then: should work correctly
      expect(result).toBe('Org: admin')
    })
  })
})
