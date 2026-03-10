/**
 * BDD tests for RpcApiMock - mock RPC API service implementation for testing.
 * Tests verify the behavior of the mock RpcApi implementation.
 */
import { describe, test, expect } from 'bun:test'
import { Effect } from 'effect'
import { RpcApiMock } from './RpcApiMock'
import { RpcApi } from './RpcApi'

describe('RpcApiMock', () => {
  describe('subscribe behavior', () => {
    test('should return subscription_id successfully', async () => {
      // Given: RpcApiMock layer
      // When: subscribing
      const program = Effect.gen(function* () {
        const api = yield* RpcApi
        return yield* api.subscribe('test-entity')
      })

      const result = await Effect.runPromise(
        program.pipe(Effect.provide(RpcApiMock)),
      )

      // Then: should return subscription_id with mock-sub prefix
      expect(result.subscription_id).toMatch(/^mock-sub-\d+$/)
    })

    test('should generate unique subscription IDs', async () => {
      // Given: RpcApiMock layer
      // When: subscribing multiple times
      const program = Effect.gen(function* () {
        const api = yield* RpcApi
        const sub1 = yield* api.subscribe('entity-1')
        const sub2 = yield* api.subscribe('entity-2')
        const sub3 = yield* api.subscribe('entity-3')
        return { sub1, sub2, sub3 }
      })

      const result = await Effect.runPromise(
        program.pipe(Effect.provide(RpcApiMock)),
      )

      // Then: each subscription should have unique ID
      expect(result.sub1.subscription_id).toMatch(/^mock-sub-\d+$/)
      expect(result.sub2.subscription_id).toMatch(/^mock-sub-\d+$/)
      expect(result.sub3.subscription_id).toMatch(/^mock-sub-\d+$/)
      expect(result.sub1.subscription_id).not.toBe(result.sub2.subscription_id)
      expect(result.sub2.subscription_id).not.toBe(result.sub3.subscription_id)

      // Extract counter values and verify they increment
      const counter1 = parseInt(result.sub1.subscription_id.split('-')[2], 10)
      const counter2 = parseInt(result.sub2.subscription_id.split('-')[2], 10)
      const counter3 = parseInt(result.sub3.subscription_id.split('-')[2], 10)
      expect(counter2).toBe(counter1 + 1)
      expect(counter3).toBe(counter2 + 1)
    })

    test('should increment subscription counter for each call', async () => {
      // Given: RpcApiMock layer
      // When: subscribing multiple times
      const program = Effect.gen(function* () {
        const api = yield* RpcApi
        const results = []
        for (let i = 0; i < 5; i++) {
          const result = yield* api.subscribe('test-entity')
          results.push(result.subscription_id)
        }
        return results
      })

      const result = await Effect.runPromise(
        program.pipe(Effect.provide(RpcApiMock)),
      )

      // Then: counter should increment sequentially
      expect(result).toHaveLength(5)
      const counters = result.map((id) => parseInt(id.split('-')[2], 10))
      expect(counters[1]).toBe(counters[0] + 1)
      expect(counters[2]).toBe(counters[1] + 1)
      expect(counters[3]).toBe(counters[2] + 1)
      expect(counters[4]).toBe(counters[3] + 1)
    })

    test('should return subscription_id regardless of entityId', async () => {
      // Given: RpcApiMock layer
      // When: subscribing with different entityIds
      const program = Effect.gen(function* () {
        const api = yield* RpcApi
        const sub1 = yield* api.subscribe('entity-1')
        const sub2 = yield* api.subscribe('entity-2')
        const sub3 = yield* api.subscribe('different/entity?id=123')
        return { sub1, sub2, sub3 }
      })

      const result = await Effect.runPromise(
        program.pipe(Effect.provide(RpcApiMock)),
      )

      // Then: all should return subscription_id (entityId ignored)
      expect(result.sub1.subscription_id).toMatch(/^mock-sub-\d+$/)
      expect(result.sub2.subscription_id).toMatch(/^mock-sub-\d+$/)
      expect(result.sub3.subscription_id).toMatch(/^mock-sub-\d+$/)
    })

    test('should return subscription_id regardless of params', async () => {
      // Given: RpcApiMock layer
      // When: subscribing with different params
      const program = Effect.gen(function* () {
        const api = yield* RpcApi
        const sub1 = yield* api.subscribe('test-entity', {
          filter: 'name:test',
          sort: 'name',
          order: 'asc',
          offset: 10,
          limit: 20,
        })
        const sub2 = yield* api.subscribe('test-entity', {
          filter: 'different',
        })
        const sub3 = yield* api.subscribe('test-entity', {})
        return { sub1, sub2, sub3 }
      })

      const result = await Effect.runPromise(
        program.pipe(Effect.provide(RpcApiMock)),
      )

      // Then: all should return subscription_id (params ignored)
      expect(result.sub1.subscription_id).toMatch(/^mock-sub-\d+$/)
      expect(result.sub2.subscription_id).toMatch(/^mock-sub-\d+$/)
      expect(result.sub3.subscription_id).toMatch(/^mock-sub-\d+$/)
    })

    test('should return subscription_id with undefined params', async () => {
      // Given: RpcApiMock layer
      // When: subscribing with undefined params
      const program = Effect.gen(function* () {
        const api = yield* RpcApi
        return yield* api.subscribe('test-entity', undefined)
      })

      const result = await Effect.runPromise(
        program.pipe(Effect.provide(RpcApiMock)),
      )

      // Then: should return subscription_id
      expect(result.subscription_id).toMatch(/^mock-sub-\d+$/)
    })

    test('should always succeed', async () => {
      // Given: RpcApiMock layer
      // When: subscribing
      const program = Effect.gen(function* () {
        const api = yield* RpcApi
        return yield* api.subscribe('test-entity')
      })

      // Then: should not throw
      const result = await Effect.runPromise(
        program.pipe(Effect.provide(RpcApiMock)),
      )

      expect(result.subscription_id).toBeDefined()
    })
  })

  describe('Layer behavior', () => {
    test('should provide RpcApi service', async () => {
      // Given: RpcApiMock layer
      // When: accessing RpcApi
      const program = Effect.gen(function* () {
        const api = yield* RpcApi
        return api
      })

      const result = await Effect.runPromise(
        program.pipe(Effect.provide(RpcApiMock)),
      )

      // Then: should return RpcApiService
      expect(result).toBeDefined()
      expect(result.subscribe).toBeDefined()
      expect(typeof result.subscribe).toBe('function')
    })

    test('should not require any dependencies', async () => {
      // Given: RpcApiMock layer (no dependencies)
      // When: using it directly
      const program = Effect.gen(function* () {
        const api = yield* RpcApi
        const result = yield* api.subscribe('test-entity', {
          filter: 'test',
        })
        return result
      })

      // Then: should work without providing additional layers
      const result = await Effect.runPromise(
        program.pipe(Effect.provide(RpcApiMock)),
      )

      expect(result.subscription_id).toMatch(/^mock-sub-\d+$/)
    })

    test('should share counter across layer uses', async () => {
      // Given: RpcApiMock uses singleton instance
      // When: using layer multiple times
      const program1 = Effect.gen(function* () {
        const api = yield* RpcApi
        return yield* api.subscribe('entity-1')
      })
      const program2 = Effect.gen(function* () {
        const api = yield* RpcApi
        return yield* api.subscribe('entity-2')
      })

      const result1 = await Effect.runPromise(
        program1.pipe(Effect.provide(RpcApiMock)),
      )
      const result2 = await Effect.runPromise(
        program2.pipe(Effect.provide(RpcApiMock)),
      )

      // Then: counter should continue incrementing (shared instance)
      expect(result1.subscription_id).toMatch(/^mock-sub-\d+$/)
      expect(result2.subscription_id).toMatch(/^mock-sub-\d+$/)
      const counter1 = parseInt(result1.subscription_id.split('-')[2], 10)
      const counter2 = parseInt(result2.subscription_id.split('-')[2], 10)
      expect(counter2).toBeGreaterThan(counter1)
    })
  })

  describe('subscription counter behavior', () => {
    test('should return subscription_id with counter format', async () => {
      // Given: RpcApiMock layer
      // When: subscribing
      const program = Effect.gen(function* () {
        const api = yield* RpcApi
        return yield* api.subscribe('test-entity')
      })

      const result = await Effect.runPromise(
        program.pipe(Effect.provide(RpcApiMock)),
      )

      // Then: should match mock-sub-{number} format
      expect(result.subscription_id).toMatch(/^mock-sub-\d+$/)
      const counter = parseInt(result.subscription_id.split('-')[2], 10)
      expect(counter).toBeGreaterThan(0)
    })

    test('should increment counter across multiple subscriptions', async () => {
      // Given: RpcApiMock layer
      // When: subscribing multiple times in sequence
      const program = Effect.gen(function* () {
        const api = yield* RpcApi
        const results = []
        for (let i = 0; i < 10; i++) {
          const result = yield* api.subscribe(`entity-${i}`)
          results.push(result.subscription_id)
        }
        return results
      })

      const result = await Effect.runPromise(
        program.pipe(Effect.provide(RpcApiMock)),
      )

      // Then: counter should increment sequentially
      expect(result).toHaveLength(10)
      const counters = result.map((id) => parseInt(id.split('-')[2], 10))
      for (let i = 1; i < counters.length; i++) {
        expect(counters[i]).toBe(counters[i - 1] + 1)
      }
    })
  })

  describe('edge cases', () => {
    test('should handle empty string entityId', async () => {
      // Given: RpcApiMock layer
      // When: subscribing with empty entityId
      const program = Effect.gen(function* () {
        const api = yield* RpcApi
        return yield* api.subscribe('')
      })

      const result = await Effect.runPromise(
        program.pipe(Effect.provide(RpcApiMock)),
      )

      // Then: should still return subscription_id
      expect(result.subscription_id).toMatch(/^mock-sub-\d+$/)
    })

    test('should handle special characters in entityId', async () => {
      // Given: RpcApiMock layer
      // When: subscribing with special characters in entityId
      const program = Effect.gen(function* () {
        const api = yield* RpcApi
        return yield* api.subscribe('test/entity?id=123&name=test')
      })

      const result = await Effect.runPromise(
        program.pipe(Effect.provide(RpcApiMock)),
      )

      // Then: should still return subscription_id (entityId ignored)
      expect(result.subscription_id).toMatch(/^mock-sub-\d+$/)
    })

    test('should handle very long entityId', async () => {
      // Given: RpcApiMock layer
      // When: subscribing with very long entityId
      const longEntityId = 'a'.repeat(1000)
      const program = Effect.gen(function* () {
        const api = yield* RpcApi
        return yield* api.subscribe(longEntityId)
      })

      const result = await Effect.runPromise(
        program.pipe(Effect.provide(RpcApiMock)),
      )

      // Then: should still return subscription_id
      expect(result.subscription_id).toMatch(/^mock-sub-\d+$/)
    })
  })
})
