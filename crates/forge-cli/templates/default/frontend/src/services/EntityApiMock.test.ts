/**
 * BDD tests for EntityApiMock
 * Tests verify the behavior of the mock EntityApi implementation for testing
 */
import { describe, test, expect } from 'bun:test'
import { Effect, Layer } from 'effect'
import { EntityApiMock } from './EntityApiMock'
import { EntityApi } from './EntityApi'

describe('EntityApiMock', () => {
  describe('list behavior', () => {
    test('should return empty data array', async () => {
      // Given: EntityApiMock layer
      // When: listing entities
      const program = Effect.gen(function* () {
        const api = yield* EntityApi
        return yield* api.list('test-entity')
      })

      const result = await Effect.runPromise(
        program.pipe(Effect.provide(EntityApiMock)),
      )

      // Then: should return empty data array
      expect(result.data).toEqual([])
      expect(result.data).toHaveLength(0)
    })

    test('should return empty data array regardless of entityId', async () => {
      // Given: EntityApiMock layer
      // When: listing with different entityIds
      const program1 = Effect.gen(function* () {
        const api = yield* EntityApi
        return yield* api.list('entity-1')
      })
      const program2 = Effect.gen(function* () {
        const api = yield* EntityApi
        return yield* api.list('entity-2')
      })
      const program3 = Effect.gen(function* () {
        const api = yield* EntityApi
        return yield* api.list('different/entity?id=123')
      })

      const result1 = await Effect.runPromise(
        program1.pipe(Effect.provide(EntityApiMock)),
      )
      const result2 = await Effect.runPromise(
        program2.pipe(Effect.provide(EntityApiMock)),
      )
      const result3 = await Effect.runPromise(
        program3.pipe(Effect.provide(EntityApiMock)),
      )

      // Then: all should return empty data array
      expect(result1.data).toEqual([])
      expect(result2.data).toEqual([])
      expect(result3.data).toEqual([])
    })

    test('should return empty data array regardless of query params', async () => {
      // Given: EntityApiMock layer
      // When: listing with query params
      const program = Effect.gen(function* () {
        const api = yield* EntityApi
        return yield* api.list('test-entity', {
          filter: 'name:test',
          sort: 'name',
          order: 'asc',
          offset: 10,
          limit: 20,
        })
      })

      const result = await Effect.runPromise(
        program.pipe(Effect.provide(EntityApiMock)),
      )

      // Then: should return empty data array (params ignored)
      expect(result.data).toEqual([])
    })

    test('should return empty data array with undefined params', async () => {
      // Given: EntityApiMock layer
      // When: listing with undefined params
      const program = Effect.gen(function* () {
        const api = yield* EntityApi
        return yield* api.list('test-entity', undefined)
      })

      const result = await Effect.runPromise(
        program.pipe(Effect.provide(EntityApiMock)),
      )

      // Then: should return empty data array
      expect(result.data).toEqual([])
    })
  })

  describe('get behavior', () => {
    test('should return empty object', async () => {
      // Given: EntityApiMock layer
      // When: getting entity
      const program = Effect.gen(function* () {
        const api = yield* EntityApi
        return yield* api.get('test-entity', '123')
      })

      const result = await Effect.runPromise(
        program.pipe(Effect.provide(EntityApiMock)),
      )

      // Then: should return empty object
      expect(result).toEqual({})
    })

    test('should return empty object regardless of entityId', async () => {
      // Given: EntityApiMock layer
      // When: getting with different entityIds
      const program1 = Effect.gen(function* () {
        const api = yield* EntityApi
        return yield* api.get('entity-1', 'id-1')
      })
      const program2 = Effect.gen(function* () {
        const api = yield* EntityApi
        return yield* api.get('entity-2', 'id-2')
      })

      const result1 = await Effect.runPromise(
        program1.pipe(Effect.provide(EntityApiMock)),
      )
      const result2 = await Effect.runPromise(
        program2.pipe(Effect.provide(EntityApiMock)),
      )

      // Then: all should return empty object
      expect(result1).toEqual({})
      expect(result2).toEqual({})
    })

    test('should return empty object regardless of id', async () => {
      // Given: EntityApiMock layer
      // When: getting with different ids
      const program = Effect.gen(function* () {
        const api = yield* EntityApi
        return yield* api.get('test-entity', 'different-id')
      })

      const result = await Effect.runPromise(
        program.pipe(Effect.provide(EntityApiMock)),
      )

      // Then: should return empty object
      expect(result).toEqual({})
    })
  })

  describe('create behavior', () => {
    test('should return empty object', async () => {
      // Given: EntityApiMock layer
      // When: creating entity
      const program = Effect.gen(function* () {
        const api = yield* EntityApi
        return yield* api.create('test-entity', { name: 'Test', value: 42 })
      })

      const result = await Effect.runPromise(
        program.pipe(Effect.provide(EntityApiMock)),
      )

      // Then: should return empty object
      expect(result).toEqual({})
    })

    test('should return empty object regardless of entityId', async () => {
      // Given: EntityApiMock layer
      // When: creating with different entityIds
      const program1 = Effect.gen(function* () {
        const api = yield* EntityApi
        return yield* api.create('entity-1', { name: 'Test' })
      })
      const program2 = Effect.gen(function* () {
        const api = yield* EntityApi
        return yield* api.create('entity-2', { name: 'Test' })
      })

      const result1 = await Effect.runPromise(
        program1.pipe(Effect.provide(EntityApiMock)),
      )
      const result2 = await Effect.runPromise(
        program2.pipe(Effect.provide(EntityApiMock)),
      )

      // Then: all should return empty object
      expect(result1).toEqual({})
      expect(result2).toEqual({})
    })

    test('should return empty object regardless of body', async () => {
      // Given: EntityApiMock layer
      // When: creating with different bodies
      const program1 = Effect.gen(function* () {
        const api = yield* EntityApi
        return yield* api.create('test-entity', { name: 'Test' })
      })
      const program2 = Effect.gen(function* () {
        const api = yield* EntityApi
        return yield* api.create('test-entity', { value: 100 })
      })
      const program3 = Effect.gen(function* () {
        const api = yield* EntityApi
        return yield* api.create('test-entity', {})
      })

      const result1 = await Effect.runPromise(
        program1.pipe(Effect.provide(EntityApiMock)),
      )
      const result2 = await Effect.runPromise(
        program2.pipe(Effect.provide(EntityApiMock)),
      )
      const result3 = await Effect.runPromise(
        program3.pipe(Effect.provide(EntityApiMock)),
      )

      // Then: all should return empty object (body ignored)
      expect(result1).toEqual({})
      expect(result2).toEqual({})
      expect(result3).toEqual({})
    })
  })

  describe('update behavior', () => {
    test('should return empty object', async () => {
      // Given: EntityApiMock layer
      // When: updating entity
      const program = Effect.gen(function* () {
        const api = yield* EntityApi
        return yield* api.update('test-entity', '123', {
          name: 'Updated',
          value: 99,
        })
      })

      const result = await Effect.runPromise(
        program.pipe(Effect.provide(EntityApiMock)),
      )

      // Then: should return empty object
      expect(result).toEqual({})
    })

    test('should return empty object regardless of entityId', async () => {
      // Given: EntityApiMock layer
      // When: updating with different entityIds
      const program1 = Effect.gen(function* () {
        const api = yield* EntityApi
        return yield* api.update('entity-1', 'id-1', { name: 'Updated' })
      })
      const program2 = Effect.gen(function* () {
        const api = yield* EntityApi
        return yield* api.update('entity-2', 'id-2', { name: 'Updated' })
      })

      const result1 = await Effect.runPromise(
        program1.pipe(Effect.provide(EntityApiMock)),
      )
      const result2 = await Effect.runPromise(
        program2.pipe(Effect.provide(EntityApiMock)),
      )

      // Then: all should return empty object
      expect(result1).toEqual({})
      expect(result2).toEqual({})
    })

    test('should return empty object regardless of id', async () => {
      // Given: EntityApiMock layer
      // When: updating with different ids
      const program = Effect.gen(function* () {
        const api = yield* EntityApi
        return yield* api.update('test-entity', 'different-id', {
          name: 'Updated',
        })
      })

      const result = await Effect.runPromise(
        program.pipe(Effect.provide(EntityApiMock)),
      )

      // Then: should return empty object
      expect(result).toEqual({})
    })

    test('should return empty object regardless of body', async () => {
      // Given: EntityApiMock layer
      // When: updating with different bodies
      const program1 = Effect.gen(function* () {
        const api = yield* EntityApi
        return yield* api.update('test-entity', '123', { name: 'Updated' })
      })
      const program2 = Effect.gen(function* () {
        const api = yield* EntityApi
        return yield* api.update('test-entity', '123', { value: 200 })
      })
      const program3 = Effect.gen(function* () {
        const api = yield* EntityApi
        return yield* api.update('test-entity', '123', {})
      })

      const result1 = await Effect.runPromise(
        program1.pipe(Effect.provide(EntityApiMock)),
      )
      const result2 = await Effect.runPromise(
        program2.pipe(Effect.provide(EntityApiMock)),
      )
      const result3 = await Effect.runPromise(
        program3.pipe(Effect.provide(EntityApiMock)),
      )

      // Then: all should return empty object (body ignored)
      expect(result1).toEqual({})
      expect(result2).toEqual({})
      expect(result3).toEqual({})
    })
  })

  describe('delete behavior', () => {
    test('should succeed and return undefined', async () => {
      // Given: EntityApiMock layer
      // When: deleting entity
      const program = Effect.gen(function* () {
        const api = yield* EntityApi
        return yield* api.delete('test-entity', '123')
      })

      const result = await Effect.runPromise(
        program.pipe(Effect.provide(EntityApiMock)),
      )

      // Then: should return undefined
      expect(result).toBeUndefined()
    })

    test('should succeed regardless of entityId', async () => {
      // Given: EntityApiMock layer
      // When: deleting with different entityIds
      const program1 = Effect.gen(function* () {
        const api = yield* EntityApi
        return yield* api.delete('entity-1', 'id-1')
      })
      const program2 = Effect.gen(function* () {
        const api = yield* EntityApi
        return yield* api.delete('entity-2', 'id-2')
      })

      const result1 = await Effect.runPromise(
        program1.pipe(Effect.provide(EntityApiMock)),
      )
      const result2 = await Effect.runPromise(
        program2.pipe(Effect.provide(EntityApiMock)),
      )

      // Then: all should succeed
      expect(result1).toBeUndefined()
      expect(result2).toBeUndefined()
    })

    test('should succeed regardless of id', async () => {
      // Given: EntityApiMock layer
      // When: deleting with different ids
      const program = Effect.gen(function* () {
        const api = yield* EntityApi
        return yield* api.delete('test-entity', 'different-id')
      })

      const result = await Effect.runPromise(
        program.pipe(Effect.provide(EntityApiMock)),
      )

      // Then: should succeed
      expect(result).toBeUndefined()
    })
  })

  describe('Layer behavior', () => {
    test('should provide EntityApi service', async () => {
      // Given: EntityApiMock layer
      // When: accessing EntityApi
      const program = Effect.gen(function* () {
        const api = yield* EntityApi
        return api
      })

      const result = await Effect.runPromise(
        program.pipe(Effect.provide(EntityApiMock)),
      )

      // Then: should return EntityApiService
      expect(result).toBeDefined()
      expect(result.list).toBeDefined()
      expect(result.get).toBeDefined()
      expect(result.create).toBeDefined()
      expect(result.update).toBeDefined()
      expect(result.delete).toBeDefined()
      expect(typeof result.list).toBe('function')
      expect(typeof result.get).toBe('function')
      expect(typeof result.create).toBe('function')
      expect(typeof result.update).toBe('function')
      expect(typeof result.delete).toBe('function')
    })

    test('should not require any dependencies', async () => {
      // Given: EntityApiMock layer (no dependencies)
      // When: using it directly
      const program = Effect.gen(function* () {
        const api = yield* EntityApi
        const listResult = yield* api.list('test')
        const getResult = yield* api.get('test', 'id')
        const createResult = yield* api.create('test', {})
        const updateResult = yield* api.update('test', 'id', {})
        const deleteResult = yield* api.delete('test', 'id')
        return {
          listResult,
          getResult,
          createResult,
          updateResult,
          deleteResult,
        }
      })

      // Then: should work without providing additional layers
      const result = await Effect.runPromise(
        program.pipe(Effect.provide(EntityApiMock)),
      )

      expect(result.listResult.data).toEqual([])
      expect(result.getResult).toEqual({})
      expect(result.createResult).toEqual({})
      expect(result.updateResult).toEqual({})
      expect(result.deleteResult).toBeUndefined()
    })
  })
})
