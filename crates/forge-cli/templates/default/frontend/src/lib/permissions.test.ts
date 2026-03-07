/**
 * BDD tests for Permissions service - permission checking utilities.
 * Tests use bun:test with BDD-style Given/When/Then structure.
 */
import { describe, test, expect } from 'bun:test'
import {
  hasPermission,
  shouldShowWithPermissions,
  shouldHideWithPermissions,
  shouldntShowWithPermissions,
  shouldntHideWithPermissions,
  shouldShowWithoutPermissions,
  shouldHideWithoutPermissions,
  shouldntShowWithoutPermissions,
  shouldntHideWithoutPermissions,
} from './permissions'

describe('Permissions service', () => {
  describe('hasPermission behavior', () => {
    test('should return true when user has required permission', () => {
      // Given: user has the required permission
      const permissions = ['read:users', 'write:users']
      const required = 'read:users'

      // When: checking permission
      const result = hasPermission(permissions, required)

      // Then: should return true
      expect(result).toBe(true)
    })

    test('should return true when user has any of multiple required permissions', () => {
      // Given: user has one of the required permissions
      const permissions = ['read:users']
      const required = ['read:users', 'write:users']

      // When: checking permissions
      const result = hasPermission(permissions, required)

      // Then: should return true
      expect(result).toBe(true)
    })

    test('should return false when user lacks required permission', () => {
      // Given: user does not have the required permission
      const permissions = ['read:users']
      const required = 'write:users'

      // When: checking permission
      const result = hasPermission(permissions, required)

      // Then: should return false
      expect(result).toBe(false)
    })

    test('should return false when user has none of multiple required permissions', () => {
      // Given: user has none of the required permissions
      const permissions = ['read:posts']
      const required = ['read:users', 'write:users']

      // When: checking permissions
      const result = hasPermission(permissions, required)

      // Then: should return false
      expect(result).toBe(false)
    })

    test('should return true when user has all.read permission', () => {
      // Given: user has all.read permission
      const permissions = ['all.read']
      const required = 'read:users'

      // When: checking permission
      const result = hasPermission(permissions, required)

      // Then: should return true (all.read grants all read permissions)
      expect(result).toBe(true)
    })

    test('should return true when user has all.write permission', () => {
      // Given: user has all.write permission
      const permissions = ['all.write']
      const required = 'write:users'

      // When: checking permission
      const result = hasPermission(permissions, required)

      // Then: should return true (all.write grants all write permissions)
      expect(result).toBe(true)
    })

    test('should return true when user has all.read for write permission check', () => {
      // Given: user has all.read permission
      const permissions = ['all.read']
      const required = 'write:users'

      // When: checking write permission
      const result = hasPermission(permissions, required)

      // Then: should return true (all.read grants all permissions)
      expect(result).toBe(true)
    })

    test('should handle empty permissions array', () => {
      // Given: user has no permissions
      const permissions: string[] = []
      const required = 'read:users'

      // When: checking permission
      const result = hasPermission(permissions, required)

      // Then: should return false
      expect(result).toBe(false)
    })

    test('should handle empty required permissions', () => {
      // Given: user has permissions but required is empty
      const permissions = ['read:users']
      const required: string[] = []

      // When: checking permissions
      const result = hasPermission(permissions, required)

      // Then: should return false (no permissions to match)
      expect(result).toBe(false)
    })

    test('should handle single string required permission', () => {
      // Given: required is a single string
      const permissions = ['read:users']
      const required = 'read:users'

      // When: checking permission
      const result = hasPermission(permissions, required)

      // Then: should return true
      expect(result).toBe(true)
    })

    test('should handle readonly array permissions', () => {
      // Given: permissions is a readonly array
      const permissions = ['read:users'] as readonly string[]
      const required = 'read:users'

      // When: checking permission
      const result = hasPermission(permissions, required)

      // Then: should return true
      expect(result).toBe(true)
    })
  })

  describe('shouldShowWithPermissions behavior', () => {
    test('should return true when user has required permission', () => {
      // Given: user has the required permission
      const userPermissions = ['read:users']
      const required = 'read:users'

      // When: checking if should show
      const result = shouldShowWithPermissions(userPermissions, required)

      // Then: should return true
      expect(result).toBe(true)
    })

    test('should return true when user has any of multiple required permissions', () => {
      // Given: user has one of the required permissions
      const userPermissions = ['read:users']
      const required1 = 'read:users'
      const required2 = 'write:users'

      // When: checking if should show
      const result = shouldShowWithPermissions(
        userPermissions,
        required1,
        required2,
      )

      // Then: should return true
      expect(result).toBe(true)
    })

    test('should return false when user lacks required permission', () => {
      // Given: user does not have the required permission
      const userPermissions = ['read:posts']
      const required = 'read:users'

      // When: checking if should show
      const result = shouldShowWithPermissions(userPermissions, required)

      // Then: should return false
      expect(result).toBe(false)
    })

    test('should return true when user has all.read', () => {
      // Given: user has all.read permission
      const userPermissions = ['all.read']
      const required = 'read:users'

      // When: checking if should show
      const result = shouldShowWithPermissions(userPermissions, required)

      // Then: should return true
      expect(result).toBe(true)
    })
  })

  describe('shouldHideWithPermissions behavior', () => {
    test('should return true when user has blacklist permission', () => {
      // Given: user has the blacklist permission
      const userPermissions = ['suspended']
      const blacklist = 'suspended'

      // When: checking if should hide
      const result = shouldHideWithPermissions(userPermissions, blacklist)

      // Then: should return true
      expect(result).toBe(true)
    })

    test('should return false when user lacks blacklist permission', () => {
      // Given: user does not have the blacklist permission
      const userPermissions = ['read:users']
      const blacklist = 'suspended'

      // When: checking if should hide
      const result = shouldHideWithPermissions(userPermissions, blacklist)

      // Then: should return false
      expect(result).toBe(false)
    })

    test('should return true when user has any of multiple blacklist permissions', () => {
      // Given: user has one of the blacklist permissions
      const userPermissions = ['suspended']
      const blacklist1 = 'suspended'
      const blacklist2 = 'banned'

      // When: checking if should hide
      const result = shouldHideWithPermissions(
        userPermissions,
        blacklist1,
        blacklist2,
      )

      // Then: should return true
      expect(result).toBe(true)
    })
  })

  describe('shouldntShowWithPermissions behavior', () => {
    test('should return false when user has required permission', () => {
      // Given: user has the required permission
      const userPermissions = ['read:users']
      const required = 'read:users'

      // When: checking if shouldnt show
      const result = shouldntShowWithPermissions(userPermissions, required)

      // Then: should return false (should show)
      expect(result).toBe(false)
    })

    test('should return true when user lacks required permission', () => {
      // Given: user does not have the required permission
      const userPermissions = ['read:posts']
      const required = 'read:users'

      // When: checking if shouldnt show
      const result = shouldntShowWithPermissions(userPermissions, required)

      // Then: should return true (should not show)
      expect(result).toBe(true)
    })
  })

  describe('shouldntHideWithPermissions behavior', () => {
    test('should return false when user has blacklist permission', () => {
      // Given: user has the blacklist permission
      const userPermissions = ['suspended']
      const blacklist = 'suspended'

      // When: checking if shouldnt hide
      const result = shouldntHideWithPermissions(userPermissions, blacklist)

      // Then: should return false (should hide)
      expect(result).toBe(false)
    })

    test('should return true when user lacks blacklist permission', () => {
      // Given: user does not have the blacklist permission
      const userPermissions = ['read:users']
      const blacklist = 'suspended'

      // When: checking if shouldnt hide
      const result = shouldntHideWithPermissions(userPermissions, blacklist)

      // Then: should return true (should not hide)
      expect(result).toBe(true)
    })
  })

  describe('shouldShowWithoutPermissions behavior', () => {
    test('should return true when user has none of the given permissions', () => {
      // Given: user has none of the given permissions
      const userPermissions = ['read:users']
      const permissions = 'suspended'

      // When: checking if should show without permissions
      const result = shouldShowWithoutPermissions(userPermissions, permissions)

      // Then: should return true
      expect(result).toBe(true)
    })

    test('should return false when user has any of the given permissions', () => {
      // Given: user has one of the given permissions
      const userPermissions = ['suspended']
      const permissions = 'suspended'

      // When: checking if should show without permissions
      const result = shouldShowWithoutPermissions(userPermissions, permissions)

      // Then: should return false
      expect(result).toBe(false)
    })

    test('should return true when user has none of multiple given permissions', () => {
      // Given: user has none of the given permissions
      const userPermissions = ['read:users']
      const permissions1 = 'suspended'
      const permissions2 = 'banned'

      // When: checking if should show without permissions
      const result = shouldShowWithoutPermissions(
        userPermissions,
        permissions1,
        permissions2,
      )

      // Then: should return true
      expect(result).toBe(true)
    })
  })

  describe('shouldHideWithoutPermissions behavior', () => {
    test('should return true when user has none of the given permissions', () => {
      // Given: user has none of the given permissions
      const userPermissions = ['read:users']
      const permissions = 'suspended'

      // When: checking if should hide without permissions
      const result = shouldHideWithoutPermissions(userPermissions, permissions)

      // Then: should return true
      expect(result).toBe(true)
    })

    test('should return false when user has any of the given permissions', () => {
      // Given: user has one of the given permissions
      const userPermissions = ['suspended']
      const permissions = 'suspended'

      // When: checking if should hide without permissions
      const result = shouldHideWithoutPermissions(userPermissions, permissions)

      // Then: should return false
      expect(result).toBe(false)
    })
  })

  describe('shouldntShowWithoutPermissions behavior', () => {
    test('should return false when user has none of the given permissions', () => {
      // Given: user has none of the given permissions
      const userPermissions = ['read:users']
      const permissions = 'suspended'

      // When: checking if shouldnt show without permissions
      const result = shouldntShowWithoutPermissions(
        userPermissions,
        permissions,
      )

      // Then: should return false (should show)
      expect(result).toBe(false)
    })

    test('should return true when user has any of the given permissions', () => {
      // Given: user has one of the given permissions
      const userPermissions = ['suspended']
      const permissions = 'suspended'

      // When: checking if shouldnt show without permissions
      const result = shouldntShowWithoutPermissions(
        userPermissions,
        permissions,
      )

      // Then: should return true (should not show)
      expect(result).toBe(true)
    })
  })

  describe('shouldntHideWithoutPermissions behavior', () => {
    test('should return false when user has none of the given permissions', () => {
      // Given: user has none of the given permissions
      const userPermissions = ['read:users']
      const permissions = 'suspended'

      // When: checking if shouldnt hide without permissions
      const result = shouldntHideWithoutPermissions(
        userPermissions,
        permissions,
      )

      // Then: should return false (should hide)
      expect(result).toBe(false)
    })

    test('should return true when user has any of the given permissions', () => {
      // Given: user has one of the given permissions
      const userPermissions = ['suspended']
      const permissions = 'suspended'

      // When: checking if shouldnt hide without permissions
      const result = shouldntHideWithoutPermissions(
        userPermissions,
        permissions,
      )

      // Then: should return true (should not hide)
      expect(result).toBe(true)
    })
  })
})
