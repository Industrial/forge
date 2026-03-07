/**
 * BDD tests for AuditLogMock - tests the mock implementation itself.
 * Tests use Effect's Layer system for dependency injection (no vi.mock()).
 */
import { describe, test, expect } from 'bun:test'
import { Effect, Layer } from 'effect'

import { AuditLog } from './AuditLog'
import { createAuditLogMock, AuditLogMockLayer } from './AuditLogMock'
import { AuditLogEntry } from '../domain/AuditLogEntry'
import type { AuditLogListParams } from './AuditLog'

describe('AuditLogMock', () => {
  describe('createAuditLogMock behavior', () => {
    test('should create mock service with empty entries by default', async () => {
      // Given: createAuditLogMock called without arguments
      const mock = createAuditLogMock()
      const mockLayer = Layer.succeed(AuditLog, mock)

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

    test('should create mock service with initial entries', async () => {
      // Given: createAuditLogMock called with initial entries
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

      const mock = createAuditLogMock(entries)
      const mockLayer = Layer.succeed(AuditLog, mock)

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

      // Then: should return initial entries
      expect(result.entries).toHaveLength(2)
      expect(result.total).toBe(2)
      expect(result.entries[0].id).toBe('entry-1')
      expect(result.entries[1].id).toBe('entry-2')
    })

    test('should normalize entries that are not AuditLogEntry instances', async () => {
      // Given: createAuditLogMock called with plain objects
      const plainEntries = [
        {
          id: 'entry-1',
          event_kind: 'user.created',
          actor_id: 'actor-1',
          action: 'create',
          resource_type: 'user',
          outcome: 'success',
          occurred_at: '2024-01-01T00:00:00Z',
        },
      ]

      const mock = createAuditLogMock(plainEntries as any)
      const mockLayer = Layer.succeed(AuditLog, mock)

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

      // Then: should normalize to AuditLogEntry instances
      expect(result.entries).toHaveLength(1)
      expect(result.entries[0]).toBeInstanceOf(AuditLogEntry)
      expect(result.entries[0].id).toBe('entry-1')
    })

    test('should preserve AuditLogEntry instances', async () => {
      // Given: createAuditLogMock called with AuditLogEntry instances
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

      const mock = createAuditLogMock(entries)
      const mockLayer = Layer.succeed(AuditLog, mock)

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

      // Then: should preserve instances (not create new ones)
      expect(result.entries[0]).toBeInstanceOf(AuditLogEntry)
      expect(result.entries[0].id).toBe('entry-1')
    })

    test('should handle mixed entry types', async () => {
      // Given: createAuditLogMock called with mix of instances and plain objects
      const mixedEntries = [
        new AuditLogEntry({
          id: 'entry-1',
          event_kind: 'user.created',
          actor_id: 'actor-1',
          action: 'create',
          resource_type: 'user',
          outcome: 'success',
          occurred_at: '2024-01-01T00:00:00Z',
        }),
        {
          id: 'entry-2',
          event_kind: 'user.updated',
          actor_id: 'actor-1',
          action: 'update',
          resource_type: 'user',
          outcome: 'success',
          occurred_at: '2024-01-02T00:00:00Z',
        } as any,
      ]

      const mock = createAuditLogMock(mixedEntries)
      const mockLayer = Layer.succeed(AuditLog, mock)

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

      // Then: should normalize all entries
      expect(result.entries).toHaveLength(2)
      expect(result.entries[0]).toBeInstanceOf(AuditLogEntry)
      expect(result.entries[1]).toBeInstanceOf(AuditLogEntry)
      expect(result.entries[0].id).toBe('entry-1')
      expect(result.entries[1].id).toBe('entry-2')
    })

    test('should correctly calculate pagination boundaries', async () => {
      // Given: mock with multiple entries
      const entries = Array.from({ length: 10 }, (_, i) =>
        new AuditLogEntry({
          id: `entry-${i + 1}`,
          event_kind: 'user.created',
          actor_id: 'actor-1',
          action: 'create',
          resource_type: 'user',
          outcome: 'success',
          occurred_at: `2024-01-${String(i + 1).padStart(2, '0')}T00:00:00Z`,
        }),
      )

      const mock = createAuditLogMock(entries)
      const mockLayer = Layer.succeed(AuditLog, mock)

      // When: requesting page with limit and offset
      const auditLog = await Effect.runPromise(
        Effect.gen(function* () {
          return yield* AuditLog
        }).pipe(Effect.provide(mockLayer)),
      )

      const result = await Effect.runPromise(
        auditLog
          .list({ limit: 3, offset: 2 })
          .pipe(Effect.provide(mockLayer)),
      )

      // Then: should return correct page slice
      expect(result.entries).toHaveLength(3)
      expect(result.total).toBe(10)
      expect(result.entries[0].id).toBe('entry-3')
      expect(result.entries[1].id).toBe('entry-4')
      expect(result.entries[2].id).toBe('entry-5')
    })

    test('should handle offset at end of entries', async () => {
      // Given: mock with entries, offset at end
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

      const mock = createAuditLogMock(entries)
      const mockLayer = Layer.succeed(AuditLog, mock)

      // When: requesting with offset at end
      const auditLog = await Effect.runPromise(
        Effect.gen(function* () {
          return yield* AuditLog
        }).pipe(Effect.provide(mockLayer)),
      )

      const result = await Effect.runPromise(
        auditLog
          .list({ limit: 10, offset: 2 })
          .pipe(Effect.provide(mockLayer)),
      )

      // Then: should return empty array but correct total
      expect(result.entries).toHaveLength(0)
      expect(result.total).toBe(2)
    })

    test('should use Math.min for end boundary calculation', async () => {
      // Given: mock with entries, limit larger than remaining entries
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

      const mock = createAuditLogMock(entries)
      const mockLayer = Layer.succeed(AuditLog, mock)

      // When: requesting with offset=1, limit=100 (larger than remaining)
      const auditLog = await Effect.runPromise(
        Effect.gen(function* () {
          return yield* AuditLog
        }).pipe(Effect.provide(mockLayer)),
      )

      const result = await Effect.runPromise(
        auditLog
          .list({ limit: 100, offset: 1 })
          .pipe(Effect.provide(mockLayer)),
      )

      // Then: should return only remaining entries (Math.min prevents overflow)
      expect(result.entries).toHaveLength(1)
      expect(result.total).toBe(2)
      expect(result.entries[0].id).toBe('entry-2')
    })
  })

  describe('AuditLogMockLayer behavior', () => {
    test('should create layer with empty entries by default', async () => {
      // Given: AuditLogMockLayer called without arguments
      const mockLayer = AuditLogMockLayer()

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

    test('should create layer with initial entries', async () => {
      // Given: AuditLogMockLayer called with initial entries
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

      // Then: should return initial entries
      expect(result.entries).toHaveLength(1)
      expect(result.total).toBe(1)
      expect(result.entries[0].id).toBe('entry-1')
    })

    test('should provide AuditLog service via Layer.succeed', async () => {
      // Given: AuditLogMockLayer
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

      // When: accessing service multiple times
      const program = Effect.gen(function* () {
        const service1 = yield* AuditLog
        const service2 = yield* AuditLog
        return { service1, service2 }
      })

      const { service1, service2 } = await Effect.runPromise(
        program.pipe(Effect.provide(mockLayer)),
      )

      // Then: should return same instance (Layer.succeed behavior)
      expect(service1).toBe(service2)
      expect(typeof service1.list).toBe('function')
      expect(typeof service2.list).toBe('function')
    })

    test('should be composable with other layers', async () => {
      // Given: AuditLogMockLayer and another layer
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
      const combinedLayer = Layer.mergeAll(mockLayer)

      const params: AuditLogListParams = {
        limit: 10,
        offset: 0,
      }

      // When: using combined layer
      const auditLog = await Effect.runPromise(
        Effect.gen(function* () {
          return yield* AuditLog
        }).pipe(Effect.provide(combinedLayer)),
      )

      const result = await Effect.runPromise(
        auditLog.list(params).pipe(Effect.provide(combinedLayer)),
      )

      // Then: should work correctly
      expect(result.entries).toHaveLength(1)
      expect(result.total).toBe(1)
    })
  })

  describe('mock isolation', () => {
    test('should maintain separate state per mock instance', async () => {
      // Given: two separate mock instances
      const entries1 = [
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

      const entries2 = [
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

      const mock1 = createAuditLogMock(entries1)
      const mock2 = createAuditLogMock(entries2)
      const layer1 = Layer.succeed(AuditLog, mock1)
      const layer2 = Layer.succeed(AuditLog, mock2)

      const params: AuditLogListParams = {
        limit: 10,
        offset: 0,
      }

      // When: listing from each mock
      const auditLog1 = await Effect.runPromise(
        Effect.gen(function* () {
          return yield* AuditLog
        }).pipe(Effect.provide(layer1)),
      )

      const auditLog2 = await Effect.runPromise(
        Effect.gen(function* () {
          return yield* AuditLog
        }).pipe(Effect.provide(layer2)),
      )

      const result1 = await Effect.runPromise(
        auditLog1.list(params).pipe(Effect.provide(layer1)),
      )

      const result2 = await Effect.runPromise(
        auditLog2.list(params).pipe(Effect.provide(layer2)),
      )

      // Then: each mock should have its own state
      expect(result1.entries).toHaveLength(1)
      expect(result1.entries[0].id).toBe('entry-1')
      expect(result2.entries).toHaveLength(1)
      expect(result2.entries[0].id).toBe('entry-2')
    })
  })
})
