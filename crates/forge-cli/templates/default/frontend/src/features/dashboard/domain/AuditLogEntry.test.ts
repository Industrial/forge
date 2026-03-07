/**
 * BDD tests for AuditLogEntry domain
 * Tests verify the behavior of the AuditLogEntry Data.TaggedClass
 * AuditLogEntry represents an audit log entry used by the AuditLog service
 */

import { describe, test, expect } from 'bun:test'
import { AuditLogEntry } from './AuditLogEntry'

describe('AuditLogEntry domain', () => {
  describe('instance creation behavior', () => {
    test('should create an instance with all required properties', () => {
      // Given: audit log entry data with required properties
      const id = 'entry-123'
      const eventKind = 'authz'
      const actorId = 'actor-456'
      const action = 'read'
      const resourceType = 'user'
      const outcome = 'allowed'
      const occurredAt = '2024-01-01T00:00:00Z'

      // When: I create an AuditLogEntry instance
      const entry = new AuditLogEntry({
        id,
        event_kind: eventKind,
        actor_id: actorId,
        action,
        resource_type: resourceType,
        outcome,
        occurred_at: occurredAt,
      })

      // Then: it should have all required properties
      expect(entry.id).toBe(id)
      expect(entry.event_kind).toBe(eventKind)
      expect(entry.actor_id).toBe(actorId)
      expect(entry.action).toBe(action)
      expect(entry.resource_type).toBe(resourceType)
      expect(entry.outcome).toBe(outcome)
      expect(entry.occurred_at).toBe(occurredAt)
    })

    test('should create an instance with optional subject_id', () => {
      // Given: audit log entry data with subject_id
      const subjectId = 'subject-789'
      const entry = new AuditLogEntry({
        id: 'entry-123',
        event_kind: 'authz',
        actor_id: 'actor-456',
        subject_id: subjectId,
        action: 'read',
        resource_type: 'user',
        outcome: 'allowed',
        occurred_at: '2024-01-01T00:00:00Z',
      })

      // Then: it should include the subject_id
      expect(entry.subject_id).toBe(subjectId)
    })

    test('should create an instance with null subject_id', () => {
      // Given: audit log entry data with null subject_id
      const entry = new AuditLogEntry({
        id: 'entry-123',
        event_kind: 'authz',
        actor_id: 'actor-456',
        subject_id: null,
        action: 'read',
        resource_type: 'user',
        outcome: 'allowed',
        occurred_at: '2024-01-01T00:00:00Z',
      })

      // Then: subject_id should be null
      expect(entry.subject_id).toBeNull()
    })

    test('should create an instance with optional organization_id', () => {
      // Given: audit log entry data with organization_id
      const orgId = 'org-789'
      const entry = new AuditLogEntry({
        id: 'entry-123',
        event_kind: 'authz',
        actor_id: 'actor-456',
        organization_id: orgId,
        action: 'read',
        resource_type: 'user',
        outcome: 'allowed',
        occurred_at: '2024-01-01T00:00:00Z',
      })

      // Then: it should include the organization_id
      expect(entry.organization_id).toBe(orgId)
    })

    test('should create an instance with null organization_id', () => {
      // Given: audit log entry data with null organization_id
      const entry = new AuditLogEntry({
        id: 'entry-123',
        event_kind: 'authz',
        actor_id: 'actor-456',
        organization_id: null,
        action: 'read',
        resource_type: 'user',
        outcome: 'allowed',
        occurred_at: '2024-01-01T00:00:00Z',
      })

      // Then: organization_id should be null
      expect(entry.organization_id).toBeNull()
    })

    test('should create an instance with optional resource_id', () => {
      // Given: audit log entry data with resource_id
      const resourceId = 'resource-789'
      const entry = new AuditLogEntry({
        id: 'entry-123',
        event_kind: 'mutation',
        actor_id: 'actor-456',
        action: 'update',
        resource_type: 'user',
        resource_id: resourceId,
        outcome: 'success',
        occurred_at: '2024-01-01T00:00:00Z',
      })

      // Then: it should include the resource_id
      expect(entry.resource_id).toBe(resourceId)
    })

    test('should create an instance with null resource_id', () => {
      // Given: audit log entry data with null resource_id
      const entry = new AuditLogEntry({
        id: 'entry-123',
        event_kind: 'mutation',
        actor_id: 'actor-456',
        action: 'create',
        resource_type: 'user',
        resource_id: null,
        outcome: 'success',
        occurred_at: '2024-01-01T00:00:00Z',
      })

      // Then: resource_id should be null
      expect(entry.resource_id).toBeNull()
    })

    test('should create an instance with optional reason', () => {
      // Given: audit log entry data with reason
      const reason = 'User has required permissions'
      const entry = new AuditLogEntry({
        id: 'entry-123',
        event_kind: 'authz',
        actor_id: 'actor-456',
        action: 'read',
        resource_type: 'user',
        outcome: 'allowed',
        reason,
        occurred_at: '2024-01-01T00:00:00Z',
      })

      // Then: it should include the reason
      expect(entry.reason).toBe(reason)
    })

    test('should create an instance with null reason', () => {
      // Given: audit log entry data with null reason
      const entry = new AuditLogEntry({
        id: 'entry-123',
        event_kind: 'authz',
        actor_id: 'actor-456',
        action: 'read',
        resource_type: 'user',
        outcome: 'allowed',
        reason: null,
        occurred_at: '2024-01-01T00:00:00Z',
      })

      // Then: reason should be null
      expect(entry.reason).toBeNull()
    })

    test('should create an instance with all optional fields', () => {
      // Given: audit log entry data with all optional fields
      const entry = new AuditLogEntry({
        id: 'entry-123',
        event_kind: 'mutation',
        actor_id: 'actor-456',
        subject_id: 'subject-789',
        organization_id: 'org-789',
        action: 'update',
        resource_type: 'user',
        resource_id: 'resource-789',
        outcome: 'success',
        reason: 'User updated successfully',
        occurred_at: '2024-01-01T00:00:00Z',
      })

      // Then: it should have all fields
      expect(entry.subject_id).toBe('subject-789')
      expect(entry.organization_id).toBe('org-789')
      expect(entry.resource_id).toBe('resource-789')
      expect(entry.reason).toBe('User updated successfully')
    })
  })

  describe('property access behavior', () => {
    test('should access id property', () => {
      // Given: an AuditLogEntry instance
      const entry = new AuditLogEntry({
        id: 'entry-123',
        event_kind: 'authz',
        actor_id: 'actor-456',
        action: 'read',
        resource_type: 'user',
        outcome: 'allowed',
        occurred_at: '2024-01-01T00:00:00Z',
      })

      // When: accessing id property
      // Then: should return the id
      expect(entry.id).toBe('entry-123')
    })

    test('should access event_kind property', () => {
      // Given: an AuditLogEntry instance
      const entry = new AuditLogEntry({
        id: 'entry-123',
        event_kind: 'auth',
        actor_id: 'actor-456',
        action: 'read',
        resource_type: 'user',
        outcome: 'success',
        occurred_at: '2024-01-01T00:00:00Z',
      })

      // When: accessing event_kind property
      // Then: should return the event_kind
      expect(entry.event_kind).toBe('auth')
    })

    test('should access actor_id property', () => {
      // Given: an AuditLogEntry instance
      const entry = new AuditLogEntry({
        id: 'entry-123',
        event_kind: 'authz',
        actor_id: 'actor-789',
        action: 'read',
        resource_type: 'user',
        outcome: 'allowed',
        occurred_at: '2024-01-01T00:00:00Z',
      })

      // When: accessing actor_id property
      // Then: should return the actor_id
      expect(entry.actor_id).toBe('actor-789')
    })

    test('should access action property', () => {
      // Given: an AuditLogEntry instance
      const entry = new AuditLogEntry({
        id: 'entry-123',
        event_kind: 'mutation',
        actor_id: 'actor-456',
        action: 'delete',
        resource_type: 'user',
        outcome: 'success',
        occurred_at: '2024-01-01T00:00:00Z',
      })

      // When: accessing action property
      // Then: should return the action
      expect(entry.action).toBe('delete')
    })

    test('should access resource_type property', () => {
      // Given: an AuditLogEntry instance
      const entry = new AuditLogEntry({
        id: 'entry-123',
        event_kind: 'mutation',
        actor_id: 'actor-456',
        action: 'create',
        resource_type: 'organization',
        outcome: 'success',
        occurred_at: '2024-01-01T00:00:00Z',
      })

      // When: accessing resource_type property
      // Then: should return the resource_type
      expect(entry.resource_type).toBe('organization')
    })

    test('should access outcome property', () => {
      // Given: an AuditLogEntry instance
      const entry = new AuditLogEntry({
        id: 'entry-123',
        event_kind: 'authz',
        actor_id: 'actor-456',
        action: 'read',
        resource_type: 'user',
        outcome: 'denied',
        occurred_at: '2024-01-01T00:00:00Z',
      })

      // When: accessing outcome property
      // Then: should return the outcome
      expect(entry.outcome).toBe('denied')
    })

    test('should access occurred_at property', () => {
      // Given: an AuditLogEntry instance
      const occurredAt = '2024-12-31T23:59:59Z'
      const entry = new AuditLogEntry({
        id: 'entry-123',
        event_kind: 'authz',
        actor_id: 'actor-456',
        action: 'read',
        resource_type: 'user',
        outcome: 'allowed',
        occurred_at: occurredAt,
      })

      // When: accessing occurred_at property
      // Then: should return the occurred_at
      expect(entry.occurred_at).toBe(occurredAt)
    })
  })

  describe('tagged class behavior', () => {
    test('should have the AuditLogEntry tag', () => {
      // Given: an AuditLogEntry instance
      const entry = new AuditLogEntry({
        id: 'entry-123',
        event_kind: 'authz',
        actor_id: 'actor-456',
        action: 'read',
        resource_type: 'user',
        outcome: 'allowed',
        occurred_at: '2024-01-01T00:00:00Z',
      })

      // When: checking the tag
      // Then: it should be tagged as 'AuditLogEntry'
      // Data.TaggedClass provides a _tag property
      expect((entry as any)._tag).toBe('AuditLogEntry')
    })

    test('should be identifiable as AuditLogEntry type', () => {
      // Given: an AuditLogEntry instance
      const entry = new AuditLogEntry({
        id: 'entry-123',
        event_kind: 'authz',
        actor_id: 'actor-456',
        action: 'read',
        resource_type: 'user',
        outcome: 'allowed',
        occurred_at: '2024-01-01T00:00:00Z',
      })

      // When: checking the instance
      // Then: it should be an instance of AuditLogEntry
      expect(entry).toBeInstanceOf(AuditLogEntry)
    })
  })

  describe('readonly fields behavior', () => {
    test('should have readonly id field', () => {
      // Given: an AuditLogEntry instance
      const entry = new AuditLogEntry({
        id: 'entry-123',
        event_kind: 'authz',
        actor_id: 'actor-456',
        action: 'read',
        resource_type: 'user',
        outcome: 'allowed',
        occurred_at: '2024-01-01T00:00:00Z',
      })

      // When: accessing id property
      // Then: id should be readable
      expect(entry.id).toBe('entry-123')
      // TypeScript readonly modifier prevents mutation at compile time
    })

    test('should have readonly occurred_at field', () => {
      // Given: an AuditLogEntry instance
      const occurredAt = '2024-01-01T00:00:00Z'
      const entry = new AuditLogEntry({
        id: 'entry-123',
        event_kind: 'authz',
        actor_id: 'actor-456',
        action: 'read',
        resource_type: 'user',
        outcome: 'allowed',
        occurred_at: occurredAt,
      })

      // When: accessing occurred_at property
      // Then: occurred_at should be readable
      expect(entry.occurred_at).toBe(occurredAt)
      // TypeScript readonly modifier prevents mutation at compile time
    })
  })

  describe('event_kind values behavior', () => {
    test('should support authz event_kind', () => {
      // Given: audit log entry with authz event_kind
      const entry = new AuditLogEntry({
        id: 'entry-123',
        event_kind: 'authz',
        actor_id: 'actor-456',
        action: 'read',
        resource_type: 'user',
        outcome: 'allowed',
        occurred_at: '2024-01-01T00:00:00Z',
      })

      // Then: event_kind should be authz
      expect(entry.event_kind).toBe('authz')
    })

    test('should support auth event_kind', () => {
      // Given: audit log entry with auth event_kind
      const entry = new AuditLogEntry({
        id: 'entry-123',
        event_kind: 'auth',
        actor_id: 'actor-456',
        action: 'read',
        resource_type: 'user',
        outcome: 'success',
        occurred_at: '2024-01-01T00:00:00Z',
      })

      // Then: event_kind should be auth
      expect(entry.event_kind).toBe('auth')
    })

    test('should support mutation event_kind', () => {
      // Given: audit log entry with mutation event_kind
      const entry = new AuditLogEntry({
        id: 'entry-123',
        event_kind: 'mutation',
        actor_id: 'actor-456',
        action: 'update',
        resource_type: 'user',
        outcome: 'success',
        occurred_at: '2024-01-01T00:00:00Z',
      })

      // Then: event_kind should be mutation
      expect(entry.event_kind).toBe('mutation')
    })

    test('should support custom event_kind', () => {
      // Given: audit log entry with custom event_kind
      const entry = new AuditLogEntry({
        id: 'entry-123',
        event_kind: 'custom',
        actor_id: 'actor-456',
        action: 'read',
        resource_type: 'user',
        outcome: 'success',
        occurred_at: '2024-01-01T00:00:00Z',
      })

      // Then: event_kind should be custom
      expect(entry.event_kind).toBe('custom')
    })
  })

  describe('outcome values behavior', () => {
    test('should support allowed outcome', () => {
      // Given: audit log entry with allowed outcome
      const entry = new AuditLogEntry({
        id: 'entry-123',
        event_kind: 'authz',
        actor_id: 'actor-456',
        action: 'read',
        resource_type: 'user',
        outcome: 'allowed',
        occurred_at: '2024-01-01T00:00:00Z',
      })

      // Then: outcome should be allowed
      expect(entry.outcome).toBe('allowed')
    })

    test('should support denied outcome', () => {
      // Given: audit log entry with denied outcome
      const entry = new AuditLogEntry({
        id: 'entry-123',
        event_kind: 'authz',
        actor_id: 'actor-456',
        action: 'read',
        resource_type: 'user',
        outcome: 'denied',
        occurred_at: '2024-01-01T00:00:00Z',
      })

      // Then: outcome should be denied
      expect(entry.outcome).toBe('denied')
    })

    test('should support success outcome', () => {
      // Given: audit log entry with success outcome
      const entry = new AuditLogEntry({
        id: 'entry-123',
        event_kind: 'mutation',
        actor_id: 'actor-456',
        action: 'create',
        resource_type: 'user',
        outcome: 'success',
        occurred_at: '2024-01-01T00:00:00Z',
      })

      // Then: outcome should be success
      expect(entry.outcome).toBe('success')
    })

    test('should support failure outcome', () => {
      // Given: audit log entry with failure outcome
      const entry = new AuditLogEntry({
        id: 'entry-123',
        event_kind: 'mutation',
        actor_id: 'actor-456',
        action: 'create',
        resource_type: 'user',
        outcome: 'failure',
        occurred_at: '2024-01-01T00:00:00Z',
      })

      // Then: outcome should be failure
      expect(entry.outcome).toBe('failure')
    })
  })

  describe('edge cases', () => {
    test('should handle empty string values', () => {
      // Given: audit log entry with empty string values
      const entry = new AuditLogEntry({
        id: '',
        event_kind: '',
        actor_id: '',
        action: '',
        resource_type: '',
        outcome: '',
        occurred_at: '',
      })

      // Then: should accept empty strings
      expect(entry.id).toBe('')
      expect(entry.event_kind).toBe('')
      expect(entry.actor_id).toBe('')
      expect(entry.action).toBe('')
      expect(entry.resource_type).toBe('')
      expect(entry.outcome).toBe('')
      expect(entry.occurred_at).toBe('')
    })

    test('should handle long reason strings', () => {
      // Given: audit log entry with long reason
      const longReason = 'A'.repeat(500)
      const entry = new AuditLogEntry({
        id: 'entry-123',
        event_kind: 'authz',
        actor_id: 'actor-456',
        action: 'read',
        resource_type: 'user',
        outcome: 'allowed',
        reason: longReason,
        occurred_at: '2024-01-01T00:00:00Z',
      })

      // Then: should accept long reason
      expect(entry.reason).toBe(longReason)
      expect(entry.reason?.length).toBe(500)
    })

    test('should handle ISO 8601 date strings', () => {
      // Given: audit log entry with ISO 8601 date
      const isoDate = '2024-01-01T12:34:56.789Z'
      const entry = new AuditLogEntry({
        id: 'entry-123',
        event_kind: 'authz',
        actor_id: 'actor-456',
        action: 'read',
        resource_type: 'user',
        outcome: 'allowed',
        occurred_at: isoDate,
      })

      // Then: should accept ISO 8601 date format
      expect(entry.occurred_at).toBe(isoDate)
    })
  })
})
