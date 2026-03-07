/**
 * BDD tests for User domain
 * Tests verify the behavior of the User and UserMembership Data.TaggedClass
 */

import { describe, test, expect } from 'bun:test'
import { User, UserMembership } from './User'

describe('User domain', () => {
  describe('UserMembership creation behavior', () => {
    test('should create membership with org_id, org_name, and roles', () => {
      // Given: organization and role data
      const org_id = 'org-123'
      const org_name = 'Acme Corporation'
      const roles = ['admin', 'member']

      // When: creating a UserMembership
      const membership = new UserMembership({ org_id, org_name, roles })

      // Then: membership should contain all provided values
      expect(membership.org_id).toBe(org_id)
      expect(membership.org_name).toBe(org_name)
      expect(membership.roles).toEqual(roles)
    })

    test('should create membership with empty roles array', () => {
      // Given: organization data with no roles
      const org_id = 'org-456'
      const org_name = 'Empty Org'
      const roles: readonly string[] = []

      // When: creating a UserMembership
      const membership = new UserMembership({ org_id, org_name, roles })

      // Then: roles should be empty array
      expect(membership.roles).toEqual([])
      expect(membership.roles.length).toBe(0)
    })

    test('should create membership with single role', () => {
      // Given: organization with one role
      const org_id = 'org-789'
      const org_name = 'Single Role Org'
      const roles = ['viewer']

      // When: creating a UserMembership
      const membership = new UserMembership({ org_id, org_name, roles })

      // Then: should have one role
      expect(membership.roles).toEqual(['viewer'])
      expect(membership.roles.length).toBe(1)
    })

    test('should have readonly properties', () => {
      // Given: a UserMembership instance
      const membership = new UserMembership({
        org_id: 'org-1',
        org_name: 'Test Org',
        roles: ['admin'],
      })

      // When: accessing properties
      // Then: properties should be accessible and readonly (TypeScript enforces this)
      expect(membership.org_id).toBe('org-1')
      expect(membership.org_name).toBe('Test Org')
      expect(membership.roles).toEqual(['admin'])
    })

    test('should support structural equality via TaggedClass', () => {
      // Given: two UserMembership instances with same data
      const membership1 = new UserMembership({
        org_id: 'org-1',
        org_name: 'Test Org',
        roles: ['admin'],
      })
      const membership2 = new UserMembership({
        org_id: 'org-1',
        org_name: 'Test Org',
        roles: ['admin'],
      })

      // When: comparing instances
      // Then: should be structurally equal (Data.TaggedClass provides this)
      expect(membership1).toEqual(membership2)
    })
  })

  describe('User creation behavior', () => {
    test('should create user with all required fields', () => {
      // Given: complete user data
      const id = 'user-123'
      const email = 'user@example.com'
      const is_active = true
      const is_admin = false
      const created_at = '2024-01-01T00:00:00Z'
      const memberships: readonly UserMembership[] = []

      // When: creating a User
      const user = new User({
        id,
        email,
        is_active,
        is_admin,
        created_at,
        memberships,
      })

      // Then: user should contain all provided values
      expect(user.id).toBe(id)
      expect(user.email).toBe(email)
      expect(user.is_active).toBe(is_active)
      expect(user.is_admin).toBe(is_admin)
      expect(user.created_at).toBe(created_at)
      expect(user.memberships).toEqual(memberships)
    })

    test('should create user with memberships', () => {
      // Given: user data with memberships
      const membership1 = new UserMembership({
        org_id: 'org-1',
        org_name: 'Org 1',
        roles: ['admin'],
      })
      const membership2 = new UserMembership({
        org_id: 'org-2',
        org_name: 'Org 2',
        roles: ['member', 'viewer'],
      })

      const user = new User({
        id: 'user-456',
        email: 'multi@example.com',
        is_active: true,
        is_admin: false,
        created_at: '2024-01-02T00:00:00Z',
        memberships: [membership1, membership2],
      })

      // When: accessing memberships
      // Then: should have all memberships
      expect(user.memberships.length).toBe(2)
      expect(user.memberships[0].org_id).toBe('org-1')
      expect(user.memberships[1].org_id).toBe('org-2')
    })

    test('should create inactive user', () => {
      // Given: user data with is_active false
      const user = new User({
        id: 'user-inactive',
        email: 'inactive@example.com',
        is_active: false,
        is_admin: false,
        created_at: '2024-01-01T00:00:00Z',
        memberships: [],
      })

      // When: checking active status
      // Then: should be inactive
      expect(user.is_active).toBe(false)
    })

    test('should create admin user', () => {
      // Given: user data with is_admin true
      const user = new User({
        id: 'user-admin',
        email: 'admin@example.com',
        is_active: true,
        is_admin: true,
        created_at: '2024-01-01T00:00:00Z',
        memberships: [],
      })

      // When: checking admin status
      // Then: should be admin
      expect(user.is_admin).toBe(true)
    })

    test('should create user with empty memberships', () => {
      // Given: user data with no memberships
      const user = new User({
        id: 'user-no-orgs',
        email: 'noorgs@example.com',
        is_active: true,
        is_admin: false,
        created_at: '2024-01-01T00:00:00Z',
        memberships: [],
      })

      // When: accessing memberships
      // Then: should have empty array
      expect(user.memberships).toEqual([])
      expect(user.memberships.length).toBe(0)
    })

    test('should have readonly properties', () => {
      // Given: a User instance
      const user = new User({
        id: 'user-1',
        email: 'test@example.com',
        is_active: true,
        is_admin: false,
        created_at: '2024-01-01T00:00:00Z',
        memberships: [],
      })

      // When: accessing properties
      // Then: properties should be accessible and readonly (TypeScript enforces this)
      expect(user.id).toBe('user-1')
      expect(user.email).toBe('test@example.com')
      expect(user.is_active).toBe(true)
      expect(user.is_admin).toBe(false)
      expect(user.created_at).toBe('2024-01-01T00:00:00Z')
      expect(user.memberships).toEqual([])
    })

    test('should support structural equality via TaggedClass', () => {
      // Given: two User instances with same data
      const membership = new UserMembership({
        org_id: 'org-1',
        org_name: 'Test Org',
        roles: ['admin'],
      })

      const user1 = new User({
        id: 'user-1',
        email: 'test@example.com',
        is_active: true,
        is_admin: false,
        created_at: '2024-01-01T00:00:00Z',
        memberships: [membership],
      })

      const user2 = new User({
        id: 'user-1',
        email: 'test@example.com',
        is_active: true,
        is_admin: false,
        created_at: '2024-01-01T00:00:00Z',
        memberships: [membership],
      })

      // When: comparing instances
      // Then: should be structurally equal (Data.TaggedClass provides this)
      expect(user1).toEqual(user2)
    })
  })

  describe('User dashboard usage behavior', () => {
    test('should be usable in Users service context', () => {
      // Given: a User instance
      const user = new User({
        id: 'user-dashboard',
        email: 'dashboard@example.com',
        is_active: true,
        is_admin: false,
        created_at: '2024-01-01T00:00:00Z',
        memberships: [],
      })

      // When: using in dashboard context
      // Then: should have all fields needed for dashboard display
      expect(user.id).toBeDefined()
      expect(user.email).toBeDefined()
      expect(user.is_active).toBeDefined()
      expect(user.is_admin).toBeDefined()
      expect(user.created_at).toBeDefined()
      expect(user.memberships).toBeDefined()
    })

    test('should support filtering by active status', () => {
      // Given: users with different active statuses
      const activeUser = new User({
        id: 'user-active',
        email: 'active@example.com',
        is_active: true,
        is_admin: false,
        created_at: '2024-01-01T00:00:00Z',
        memberships: [],
      })

      const inactiveUser = new User({
        id: 'user-inactive',
        email: 'inactive@example.com',
        is_active: false,
        is_admin: false,
        created_at: '2024-01-01T00:00:00Z',
        memberships: [],
      })

      // When: filtering users
      const users = [activeUser, inactiveUser]
      const activeUsers = users.filter((u) => u.is_active)

      // Then: should filter correctly
      expect(activeUsers.length).toBe(1)
      expect(activeUsers[0].id).toBe('user-active')
    })

    test('should support filtering by admin status', () => {
      // Given: users with different admin statuses
      const adminUser = new User({
        id: 'user-admin',
        email: 'admin@example.com',
        is_active: true,
        is_admin: true,
        created_at: '2024-01-01T00:00:00Z',
        memberships: [],
      })

      const regularUser = new User({
        id: 'user-regular',
        email: 'regular@example.com',
        is_active: true,
        is_admin: false,
        created_at: '2024-01-01T00:00:00Z',
        memberships: [],
      })

      // When: filtering admin users
      const users = [adminUser, regularUser]
      const adminUsers = users.filter((u) => u.is_admin)

      // Then: should filter correctly
      expect(adminUsers.length).toBe(1)
      expect(adminUsers[0].id).toBe('user-admin')
    })

    test('should support accessing organization memberships', () => {
      // Given: user with multiple organization memberships
      const membership1 = new UserMembership({
        org_id: 'org-1',
        org_name: 'Organization 1',
        roles: ['admin'],
      })
      const membership2 = new UserMembership({
        org_id: 'org-2',
        org_name: 'Organization 2',
        roles: ['member'],
      })

      const user = new User({
        id: 'user-multi',
        email: 'multi@example.com',
        is_active: true,
        is_admin: false,
        created_at: '2024-01-01T00:00:00Z',
        memberships: [membership1, membership2],
      })

      // When: accessing organization memberships
      // Then: should be able to iterate and access properties
      expect(user.memberships.length).toBe(2)
      expect(user.memberships[0].org_name).toBe('Organization 1')
      expect(user.memberships[1].org_name).toBe('Organization 2')
    })

    test('should support checking user roles in organization', () => {
      // Given: user with roles in an organization
      const membership = new UserMembership({
        org_id: 'org-1',
        org_name: 'Test Org',
        roles: ['admin', 'member', 'viewer'],
      })

      const user = new User({
        id: 'user-roles',
        email: 'roles@example.com',
        is_active: true,
        is_admin: false,
        created_at: '2024-01-01T00:00:00Z',
        memberships: [membership],
      })

      // When: checking if user has a role
      const orgMembership = user.memberships.find(
        (m) => m.org_id === 'org-1',
      )
      const hasAdminRole =
        orgMembership?.roles.includes('admin') ?? false

      // Then: should correctly identify role
      expect(hasAdminRole).toBe(true)
      expect(orgMembership?.roles).toContain('admin')
      expect(orgMembership?.roles).toContain('member')
      expect(orgMembership?.roles).toContain('viewer')
    })
  })

  describe('UserMembership role behavior', () => {
    test('should support multiple roles per membership', () => {
      // Given: membership with multiple roles
      const membership = new UserMembership({
        org_id: 'org-1',
        org_name: 'Multi Role Org',
        roles: ['admin', 'member', 'viewer', 'editor'],
      })

      // When: accessing roles
      // Then: should have all roles
      expect(membership.roles.length).toBe(4)
      expect(membership.roles).toContain('admin')
      expect(membership.roles).toContain('member')
      expect(membership.roles).toContain('viewer')
      expect(membership.roles).toContain('editor')
    })

    test('should support empty roles array', () => {
      // Given: membership with no roles
      const membership = new UserMembership({
        org_id: 'org-1',
        org_name: 'No Roles Org',
        roles: [],
      })

      // When: accessing roles
      // Then: should have empty array
      expect(membership.roles).toEqual([])
      expect(membership.roles.length).toBe(0)
    })
  })

  describe('TaggedClass behavior', () => {
    test('User should be tagged correctly', () => {
      // Given: a User instance
      const user = new User({
        id: 'user-1',
        email: 'test@example.com',
        is_active: true,
        is_admin: false,
        created_at: '2024-01-01T00:00:00Z',
        memberships: [],
      })

      // When: checking tag
      // Then: should have correct tag (Data.TaggedClass provides this)
      expect(user._tag).toBe('User')
    })

    test('UserMembership should be tagged correctly', () => {
      // Given: a UserMembership instance
      const membership = new UserMembership({
        org_id: 'org-1',
        org_name: 'Test Org',
        roles: ['admin'],
      })

      // When: checking tag
      // Then: should have correct tag (Data.TaggedClass provides this)
      expect(membership._tag).toBe('UserMembership')
    })
  })
})
