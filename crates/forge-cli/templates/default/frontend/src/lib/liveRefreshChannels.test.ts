/**
 * BDD unit tests for liveRefreshChannels (channel keys and entity ID mapping).
 * Tests use bun:test with Given/When/Then structure.
 */
import { describe, test, expect } from 'bun:test'
import {
  channelToEntityId,
  type LiveRefreshChannel,
  type ForgeWebsocketKey,
} from './liveRefreshChannels'

describe('liveRefreshChannels', () => {
  describe('channelToEntityId', () => {
    test('should map audit-log to audit_log', () => {
      // Given: audit-log channel
      const channel: LiveRefreshChannel = 'audit-log'

      // When: resolving entity id
      const entityId = channelToEntityId(channel)

      // Then: should be audit_log
      expect(entityId).toBe('audit_log')
    })

    test('should map users to user', () => {
      // Given: users channel
      const channel: LiveRefreshChannel = 'users'

      // When: resolving entity id
      const entityId = channelToEntityId(channel)

      // Then: should be user
      expect(entityId).toBe('user')
    })

    test('should map roles to role', () => {
      // Given: roles channel
      const channel: LiveRefreshChannel = 'roles'

      // When: resolving entity id
      const entityId = channelToEntityId(channel)

      // Then: should be role
      expect(entityId).toBe('role')
    })

    test('should map role_permissions to role_permission', () => {
      // Given: role_permissions channel
      const channel: LiveRefreshChannel = 'role_permissions'

      // When: resolving entity id
      const entityId = channelToEntityId(channel)

      // Then: should be role_permission
      expect(entityId).toBe('role_permission')
    })

    test('should map organizations to organization', () => {
      // Given: organizations channel
      const channel: LiveRefreshChannel = 'organizations'

      // When: resolving entity id
      const entityId = channelToEntityId(channel)

      // Then: should be organization
      expect(entityId).toBe('organization')
    })
  })

  describe('LiveRefreshChannel / ForgeWebsocketKey', () => {
    test('should accept all valid channel keys as ForgeWebsocketKey', () => {
      // Given: valid channel keys (ForgeWebsocketKey is alias of LiveRefreshChannel)
      const keys: ForgeWebsocketKey[] = [
        'audit-log',
        'users',
        'roles',
        'role_permissions',
        'organizations',
      ]

      // When: passing each to channelToEntityId
      // Then: each should resolve to expected entity id
      expect(channelToEntityId(keys[0])).toBe('audit_log')
      expect(channelToEntityId(keys[1])).toBe('user')
      expect(channelToEntityId(keys[2])).toBe('role')
      expect(channelToEntityId(keys[3])).toBe('role_permission')
      expect(channelToEntityId(keys[4])).toBe('organization')
    })

    test('should return string for every channel', () => {
      // Given: all LiveRefreshChannel values
      const channels: LiveRefreshChannel[] = [
        'audit-log',
        'users',
        'roles',
        'role_permissions',
        'organizations',
      ]

      // When: resolving each to entity id
      const results = channels.map((ch) => channelToEntityId(ch))

      // Then: every result is a non-empty string
      results.forEach((entityId) => {
        expect(typeof entityId).toBe('string')
        expect(entityId.length).toBeGreaterThan(0)
      })
    })
  })
})
