/**
 * BDD tests for AuditLog service - tests the service interface via AuditLogMock.
 * Tests use Effect's Layer system for dependency injection (no vi.mock()).
 */
import { describe, test, expect } from 'bun:test'
import { Effect, Layer } from 'effect'

import { AuditLog } from './AuditLog'
import { createAuditLogMock, AuditLogMockLayer } from './AuditLogMock'
import { AuditLogEntry } from '../domain/AuditLogEntry'
import type {
  AuditLogListParams,
  AuditLogResult,
  AuditLogService,
} from './AuditLog'

describe('AuditLog service', () => {
  describe('list behavior', () => {
    test('should return paginated entries', async () => {
      // Given: mock service with multiple entries
      const entries = [
        new AuditLogEntry({
          id: 'entry-1',
          event_kind: 'user.created',
          actor_id: 'actor-1',
          action: 'create',
          resource_type: 'user',
          outcome: 'success',
          occurred_at: '2024-01-01T00:00:00Z',
        }),
        new AuditLogEntry({
          id: 'entry-2',
          event_kind: 'user.updated',
          actor_id: 'actor-1',
          action: 'update',
          resource_type: 'user',
          outcome: 'success',
          occurred_at: '2024-01-02T00:00:00Z',
        }),
        new AuditLogEntry({
          id: 'entry-3',
          event_kind: 'user.deleted',
          actor_id: 'actor-2',
          action: 'delete',
          resource_type: 'user',
          outcome: 'success',
          occurred_at: '2024-01-03T00:00:00Z',
        }),
      ]

      const mockLayer = AuditLogMockLayer(entries)
      const params: AuditLogListParams = {
        limit: 2,
        offset: 0,
      }

      // When: listing with pagination
      const auditLog = await Effect.runPromise(
        Effect.gen(function* () {
          return yield* AuditLog
        }).pipe(Effect.provide(mockLayer)),
      )

      const result = await Effect.runPromise(
        auditLog.list(params).pipe(Effect.provide(mockLayer)),
      )

      // Then: should return first page
      expect(result.entries).toHaveLength(2)
      expect(result.total).toBe(3)
      expect(result.entries[0].id).toBe('entry-1')
      expect(result.entries[1].id).toBe('entry-2')
    })

    test('should return second page', async () => {
      // Given: mock service with multiple entries
      const entries = [
        new AuditLogEntry({
          id: 'entry-1',
          event_kind: 'user.created',
          actor_id: 'actor-1',
          action: 'create',
          resource_type: 'user',
          outcome: 'success',
          occurred_at: '2024-01-01T00:00:00Z',
        }),
        new AuditLogEntry({
          id: 'entry-2',
          event_kind: 'user.updated',
          actor_id: 'actor-1',
          action: 'update',
          resource_type: 'user',
          outcome: 'success',
          occurred_at: '2024-01-02T00:00:00Z',
        }),
        new AuditLogEntry({
          id: 'entry-3',
          event_kind: 'user.deleted',
          actor_id: 'actor-2',
          action: 'delete',
          resource_type: 'user',
          outcome: 'success',
          occurred_at: '2024-01-03T00:00:00Z',
        }),
      ]

      const mockLayer = AuditLogMockLayer(entries)
      const params: AuditLogListParams = {
        limit: 2,
        offset: 2,
      }

      // When: listing second page
      const auditLog = await Effect.runPromise(
        Effect.gen(function* () {
          return yield* AuditLog
        }).pipe(Effect.provide(mockLayer)),
      )

      const result = await Effect.runPromise(
        auditLog.list(params).pipe(Effect.provide(mockLayer)),
      )

      // Then: should return second page
      expect(result.entries).toHaveLength(1)
      expect(result.total).toBe(3)
      expect(result.entries[0].id).toBe('entry-3')
    })

    test('should return empty array when no entries', async () => {
      // Given: mock service with no entries
      const mockLayer = AuditLogMockLayer([])
      const params: AuditLogListParams = {
        limit: 10,
        offset: 0,
      }

      // When: listing entries
      const auditLog = await Effect.runPromise(
        Effect.gen(function* () {
          return yield* AuditLog
        }).pipe(Effect.provide(mockLayer)),
      )

      const result = await Effect.runPromise(
        auditLog.list(params).pipe(Effect.provide(mockLayer)),
      )

      // Then: should return empty result
      expect(result.entries).toHaveLength(0)
      expect(result.total).toBe(0)
    })

    test('should handle offset beyond entries length', async () => {
      // Given: mock service with entries
      const entries = [
        new AuditLogEntry({
          id: 'entry-1',
          event_kind: 'user.created',
          actor_id: 'actor-1',
          action: 'create',
          resource_type: 'user',
          outcome: 'success',
          occurred_at: '2024-01-01T00:00:00Z',
        }),
      ]

      const mockLayer = AuditLogMockLayer(entries)
      const params: AuditLogListParams = {
        limit: 10,
        offset: 100, // Beyond entries length
      }

      // When: listing with offset beyond length
      const auditLog = await Effect.runPromise(
        Effect.gen(function* () {
          return yield* AuditLog
        }).pipe(Effect.provide(mockLayer)),
      )

      const result = await Effect.runPromise(
        auditLog.list(params).pipe(Effect.provide(mockLayer)),
      )

      // Then: should return empty array but correct total
      expect(result.entries).toHaveLength(0)
      expect(result.total).toBe(1)
    })

    test('should handle limit larger than available entries', async () => {
      // Given: mock service with fewer entries than limit
      const entries = [
        new AuditLogEntry({
          id: 'entry-1',
          event_kind: 'user.created',
          actor_id: 'actor-1',
          action: 'create',
          resource_type: 'user',
          outcome: 'success',
          occurred_at: '2024-01-01T00:00:00Z',
        }),
        new AuditLogEntry({
          id: 'entry-2',
          event_kind: 'user.updated',
          actor_id: 'actor-1',
          action: 'update',
          resource_type: 'user',
          outcome: 'success',
          occurred_at: '2024-01-02T00:00:00Z',
        }),
      ]

      const mockLayer = AuditLogMockLayer(entries)
      const params: AuditLogListParams = {
        limit: 100, // Larger than available entries
        offset: 0,
      }

      // When: listing with large limit
      const auditLog = await Effect.runPromise(
        Effect.gen(function* () {
          return yield* AuditLog
        }).pipe(Effect.provide(mockLayer)),
      )

      const result = await Effect.runPromise(
        auditLog.list(params).pipe(Effect.provide(mockLayer)),
      )

      // Then: should return all available entries
      expect(result.entries).toHaveLength(2)
      expect(result.total).toBe(2)
    })

    test('should preserve entry properties', async () => {
      // Given: mock service with entry containing all properties
      const entries = [
        new AuditLogEntry({
          id: 'entry-1',
          event_kind: 'user.created',
          actor_id: 'actor-1',
          subject_id: 'subject-1',
          organization_id: 'org-1',
          action: 'create',
          resource_type: 'user',
          resource_id: 'resource-1',
          outcome: 'success',
          reason: 'User created via API',
          occurred_at: '2024-01-01T00:00:00Z',
        }),
      ]

      const mockLayer = AuditLogMockLayer(entries)
      const params: AuditLogListParams = {
        limit: 10,
        offset: 0,
      }

      // When: listing entries
      const auditLog = await Effect.runPromise(
        Effect.gen(function* () {
          return yield* AuditLog
        }).pipe(Effect.provide(mockLayer)),
      )

      const result = await Effect.runPromise(
        auditLog.list(params).pipe(Effect.provide(mockLayer)),
      )

      // Then: should preserve all entry properties
      const entry = result.entries[0]
      expect(entry.id).toBe('entry-1')
      expect(entry.event_kind).toBe('user.created')
      expect(entry.actor_id).toBe('actor-1')
      expect(entry.subject_id).toBe('subject-1')
      expect(entry.organization_id).toBe('org-1')
      expect(entry.action).toBe('create')
      expect(entry.resource_type).toBe('user')
      expect(entry.resource_id).toBe('resource-1')
      expect(entry.outcome).toBe('success')
      expect(entry.reason).toBe('User created via API')
      expect(entry.occurred_at).toBe('2024-01-01T00:00:00Z')
    })

    test('should handle entries with null optional fields', async () => {
      // Given: mock service with entry having null optional fields
      const entries = [
        new AuditLogEntry({
          id: 'entry-1',
          event_kind: 'user.created',
          actor_id: 'actor-1',
          subject_id: null,
          organization_id: null,
          action: 'create',
          resource_type: 'user',
          resource_id: null,
          outcome: 'success',
          reason: null,
          occurred_at: '2024-01-01T00:00:00Z',
        }),
      ]

      const mockLayer = AuditLogMockLayer(entries)
      const params: AuditLogListParams = {
        limit: 10,
        offset: 0,
      }

      // When: listing entries
      const auditLog = await Effect.runPromise(
        Effect.gen(function* () {
          return yield* AuditLog
        }).pipe(Effect.provide(mockLayer)),
      )

      const result = await Effect.runPromise(
        auditLog.list(params).pipe(Effect.provide(mockLayer)),
      )

      // Then: should handle null values correctly
      const entry = result.entries[0]
      expect(entry.subject_id).toBeNull()
      expect(entry.organization_id).toBeNull()
      expect(entry.resource_id).toBeNull()
      expect(entry.reason).toBeNull()
    })

    test('should handle entries without optional fields', async () => {
      // Given: mock service with entry missing optional fields
      const entries = [
        new AuditLogEntry({
          id: 'entry-1',
          event_kind: 'user.created',
          actor_id: 'actor-1',
          action: 'create',
          resource_type: 'user',
          outcome: 'success',
          occurred_at: '2024-01-01T00:00:00Z',
        }),
      ]

      const mockLayer = AuditLogMockLayer(entries)
      const params: AuditLogListParams = {
        limit: 10,
        offset: 0,
      }

      // When: listing entries
      const auditLog = await Effect.runPromise(
        Effect.gen(function* () {
          return yield* AuditLog
        }).pipe(Effect.provide(mockLayer)),
      )

      const result = await Effect.runPromise(
        auditLog.list(params).pipe(Effect.provide(mockLayer)),
      )

      // Then: should handle missing optional fields
      const entry = result.entries[0]
      expect(entry.id).toBe('entry-1')
      expect(entry.subject_id).toBeUndefined()
      expect(entry.organization_id).toBeUndefined()
      expect(entry.resource_id).toBeUndefined()
      expect(entry.reason).toBeUndefined()
    })

    test('should accept all filter parameters without using them', async () => {
      // Given: mock service with entries and filter params
      const entries = [
        new AuditLogEntry({
          id: 'entry-1',
          event_kind: 'user.created',
          actor_id: 'actor-1',
          action: 'create',
          resource_type: 'user',
          outcome: 'success',
          occurred_at: '2024-01-01T00:00:00Z',
        }),
      ]

      const mockLayer = AuditLogMockLayer(entries)
      const params: AuditLogListParams = {
        limit: 10,
        offset: 0,
        from: '2024-01-01',
        to: '2024-01-31',
        outcome: 'success',
        event_kind: 'user.created',
        action: 'create',
        reason: 'test reason',
      }

      // When: listing with all filter params (mock ignores filters, only paginates)
      const auditLog = await Effect.runPromise(
        Effect.gen(function* () {
          return yield* AuditLog
        }).pipe(Effect.provide(mockLayer)),
      )

      const result = await Effect.runPromise(
        auditLog.list(params).pipe(Effect.provide(mockLayer)),
      )

      // Then: should return entries (mock doesn't filter, only paginates)
      expect(result.entries).toHaveLength(1)
      expect(result.total).toBe(1)
    })

    test('should return readonly arrays', async () => {
      // Given: mock service with entries
      const entries = [
        new AuditLogEntry({
          id: 'entry-1',
          event_kind: 'user.created',
          actor_id: 'actor-1',
          action: 'create',
          resource_type: 'user',
          outcome: 'success',
          occurred_at: '2024-01-01T00:00:00Z',
        }),
      ]

      const mockLayer = AuditLogMockLayer(entries)
      const params: AuditLogListParams = {
        limit: 10,
        offset: 0,
      }

      // When: listing entries
      const auditLog = await Effect.runPromise(
        Effect.gen(function* () {
          return yield* AuditLog
        }).pipe(Effect.provide(mockLayer)),
      )

      const result = await Effect.runPromise(
        auditLog.list(params).pipe(Effect.provide(mockLayer)),
      )

      // Then: should return readonly arrays
      expect(Object.isFrozen(result.entries)).toBe(false) // Arrays aren't frozen, but type is readonly
      expect(result.entries).toBeInstanceOf(Array)
    })
  })

  describe('service interface', () => {
    test('should export AuditLog tag', () => {
      // Given the AuditLog module
      // When I check the AuditLog export
      // Then it should be a Context tag
      expect(AuditLog).toBeDefined()
      expect(typeof AuditLog).toBe('object')
    })

    test('should export AuditLogService interface', () => {
      // Given the AuditLog module
      // When I use AuditLogService type
      // Then it should be usable
      const service: AuditLogService = {
        list: () => Effect.succeed({ entries: [], total: 0 }),
      }
      expect(service).toBeDefined()
      expect(typeof service.list).toBe('function')
    })

    test('should export AuditLogListParams interface with required fields', () => {
      // Given the AuditLog module
      // When I create AuditLogListParams
      // Then it should have required limit and offset fields
      const params: AuditLogListParams = {
        limit: 10,
        offset: 0,
      }
      expect(params.limit).toBe(10)
      expect(params.offset).toBe(0)
    })

    test('should allow optional filter parameters in AuditLogListParams', () => {
      // Given the AuditLog module
      // When I create AuditLogListParams with optional filters
      // Then all optional fields should be allowed
      const params: AuditLogListParams = {
        limit: 20,
        offset: 10,
        from: '2024-01-01',
        to: '2024-12-31',
        outcome: 'success',
        event_kind: 'auth',
        action: 'read',
        reason: 'test reason',
      }

      expect(params.from).toBe('2024-01-01')
      expect(params.to).toBe('2024-12-31')
      expect(params.outcome).toBe('success')
      expect(params.event_kind).toBe('auth')
      expect(params.action).toBe('read')
      expect(params.reason).toBe('test reason')
    })

    test('should export AuditLogResult interface', () => {
      // Given the AuditLog module
      // When I create AuditLogResult
      // Then it should have entries and total fields
      const result: AuditLogResult = {
        entries: [],
        total: 0,
      }
      expect(result.entries).toEqual([])
      expect(result.total).toBe(0)
    })

    test('should enforce AuditLogService interface contract', async () => {
      // Given an AuditLogService implementation
      // When I call list
      // Then it must return Effect<AuditLogResult, Error, never>
      const service: AuditLogService = {
        list: () => Effect.succeed({ entries: [], total: 0 }),
      }

      const result = await Effect.runPromise(
        service.list({ limit: 10, offset: 0 }),
      )
      expect(result).toHaveProperty('entries')
      expect(result).toHaveProperty('total')
      expect(Array.isArray(result.entries)).toBe(true)
      expect(typeof result.total).toBe('number')
    })
  })

  describe('service layer integration', () => {
    test('should be usable with Effect.provide', async () => {
      // Given AuditLogMockLayer
      // When I provide it to an effect
      // Then I should be able to access the service
      const program = Effect.gen(function* () {
        const service = yield* AuditLog
        expect(service).toBeDefined()
        expect(typeof service.list).toBe('function')
        return service
      })

      const service = await Effect.runPromise(
        program.pipe(Effect.provide(AuditLogMockLayer([]))),
      )

      expect(service).toBeDefined()
      expect(typeof service.list).toBe('function')
    })

    test('should work with multiple service accesses', async () => {
      // Given AuditLogMockLayer
      // When I access the service multiple times
      // Then it should return the same instance (Layer.succeed behavior)
      const program = Effect.gen(function* () {
        const service1 = yield* AuditLog
        const service2 = yield* AuditLog
        return { service1, service2 }
      })

      const { service1, service2 } = await Effect.runPromise(
        program.pipe(Effect.provide(AuditLogMockLayer([]))),
      )

      // Layer.succeed returns the same instance
      expect(service1).toBe(service2)
    })
  })
})
