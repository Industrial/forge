/**
 * BDD tests for EntityApi service - tests the service interface via EntityApiMock.
 * Tests use Effect's Layer system for dependency injection (no vi.mock()).
 */
import { describe, test, expect } from 'bun:test'
import { Effect } from 'effect'

import { EntityApi } from './EntityApi'
import { EntityApiMock } from './EntityApiMock'
import type { EntityApiService, ListQueryParams, ListResponse } from './EntityApi'

describe('EntityApi service', () => {
  describe('list behavior', () => {
    test('should return empty list response by default', async () => {
      // Given: EntityApiMock layer
      // When: listing entities
      const entityApi = await Effect.runPromise(
        Effect.gen(function* () {
          return yield* EntityApi
        }).pipe(Effect.provide(EntityApiMock)),
      )

      const result = await Effect.runPromise(
        entityApi.list('organization').pipe(Effect.provide(EntityApiMock)),
      )

      // Then: should return empty data array
      expect(result).toHaveProperty('data')
      expect(result.data).toEqual([])
      expect(result.data).toHaveLength(0)
    })

    test('should accept entityId parameter', async () => {
      // Given: EntityApiMock layer
      // When: listing entities with different entityId
      const entityApi = await Effect.runPromise(
        Effect.gen(function* () {
          return yield* EntityApi
        }).pipe(Effect.provide(EntityApiMock)),
      )

      const result1 = await Effect.runPromise(
        entityApi.list('organization').pipe(Effect.provide(EntityApiMock)),
      )

      const result2 = await Effect.runPromise(
        entityApi.list('user').pipe(Effect.provide(EntityApiMock)),
      )

      // Then: should return empty data for both (mock ignores entityId)
      expect(result1.data).toEqual([])
      expect(result2.data).toEqual([])
    })

    test('should accept optional ListQueryParams', async () => {
      // Given: EntityApiMock layer with query params
      const params: ListQueryParams = {
        filter: 'name:eq:Acme',
        sort: 'name',
        order: 'asc',
        offset: 0,
        limit: 10,
      }

      // When: listing with params
      const entityApi = await Effect.runPromise(
        Effect.gen(function* () {
          return yield* EntityApi
        }).pipe(Effect.provide(EntityApiMock)),
      )

      const result = await Effect.runPromise(
        entityApi.list('organization', params).pipe(Effect.provide(EntityApiMock)),
      )

      // Then: should return empty data (mock ignores params)
      expect(result.data).toEqual([])
    })

    test('should accept empty params object', async () => {
      // Given: EntityApiMock layer with empty params
      const params: ListQueryParams = {}

      // When: listing with empty params
      const entityApi = await Effect.runPromise(
        Effect.gen(function* () {
          return yield* EntityApi
        }).pipe(Effect.provide(EntityApiMock)),
      )

      const result = await Effect.runPromise(
        entityApi.list('organization', params).pipe(Effect.provide(EntityApiMock)),
      )

      // Then: should return empty data
      expect(result.data).toEqual([])
    })

    test('should return ListResponse type', async () => {
      // Given: EntityApiMock layer
      // When: listing entities
      const entityApi = await Effect.runPromise(
        Effect.gen(function* () {
          return yield* EntityApi
        }).pipe(Effect.provide(EntityApiMock)),
      )

      const result = await Effect.runPromise(
        entityApi.list('organization').pipe(Effect.provide(EntityApiMock)),
      )

      // Then: should return ListResponse shape
      expect(result).toHaveProperty('data')
      expect(Array.isArray(result.data)).toBe(true)
    })
  })

  describe('get behavior', () => {
    test('should return empty object by default', async () => {
      // Given: EntityApiMock layer
      // When: getting an entity
      const entityApi = await Effect.runPromise(
        Effect.gen(function* () {
          return yield* EntityApi
        }).pipe(Effect.provide(EntityApiMock)),
      )

      const result = await Effect.runPromise(
        entityApi.get('organization', 'id-123').pipe(Effect.provide(EntityApiMock)),
      )

      // Then: should return empty object
      expect(result).toEqual({})
      expect(typeof result).toBe('object')
    })

    test('should accept entityId and id parameters', async () => {
      // Given: EntityApiMock layer
      // When: getting entities with different parameters
      const entityApi = await Effect.runPromise(
        Effect.gen(function* () {
          return yield* EntityApi
        }).pipe(Effect.provide(EntityApiMock)),
      )

      const result1 = await Effect.runPromise(
        entityApi.get('organization', 'id-1').pipe(Effect.provide(EntityApiMock)),
      )

      const result2 = await Effect.runPromise(
        entityApi.get('user', 'id-2').pipe(Effect.provide(EntityApiMock)),
      )

      // Then: should return empty objects (mock ignores parameters)
      expect(result1).toEqual({})
      expect(result2).toEqual({})
    })
  })

  describe('create behavior', () => {
    test('should return empty object by default', async () => {
      // Given: EntityApiMock layer
      const body = {
        name: 'New Organization',
        slug: 'new-org',
      }

      // When: creating an entity
      const entityApi = await Effect.runPromise(
        Effect.gen(function* () {
          return yield* EntityApi
        }).pipe(Effect.provide(EntityApiMock)),
      )

      const result = await Effect.runPromise(
        entityApi
          .create('organization', body)
          .pipe(Effect.provide(EntityApiMock)),
      )

      // Then: should return empty object
      expect(result).toEqual({})
    })

    test('should accept entityId and body parameters', async () => {
      // Given: EntityApiMock layer with different bodies
      const body1 = { name: 'Org 1', slug: 'org-1' }
      const body2 = { name: 'Org 2', slug: 'org-2' }

      // When: creating entities
      const entityApi = await Effect.runPromise(
        Effect.gen(function* () {
          return yield* EntityApi
        }).pipe(Effect.provide(EntityApiMock)),
      )

      const result1 = await Effect.runPromise(
        entityApi
          .create('organization', body1)
          .pipe(Effect.provide(EntityApiMock)),
      )

      const result2 = await Effect.runPromise(
        entityApi
          .create('user', body2)
          .pipe(Effect.provide(EntityApiMock)),
      )

      // Then: should return empty objects (mock ignores parameters)
      expect(result1).toEqual({})
      expect(result2).toEqual({})
    })

    test('should accept Record<string, unknown> body', async () => {
      // Given: EntityApiMock layer with complex body
      const body = {
        name: 'Test',
        nested: { value: 123 },
        array: [1, 2, 3],
      }

      // When: creating an entity
      const entityApi = await Effect.runPromise(
        Effect.gen(function* () {
          return yield* EntityApi
        }).pipe(Effect.provide(EntityApiMock)),
      )

      const result = await Effect.runPromise(
        entityApi
          .create('organization', body)
          .pipe(Effect.provide(EntityApiMock)),
      )

      // Then: should return empty object
      expect(result).toEqual({})
    })
  })

  describe('update behavior', () => {
    test('should return empty object by default', async () => {
      // Given: EntityApiMock layer
      const body = {
        name: 'Updated Organization',
      }

      // When: updating an entity
      const entityApi = await Effect.runPromise(
        Effect.gen(function* () {
          return yield* EntityApi
        }).pipe(Effect.provide(EntityApiMock)),
      )

      const result = await Effect.runPromise(
        entityApi
          .update('organization', 'id-123', body)
          .pipe(Effect.provide(EntityApiMock)),
      )

      // Then: should return empty object
      expect(result).toEqual({})
    })

    test('should accept entityId, id, and body parameters', async () => {
      // Given: EntityApiMock layer
      const body = { name: 'Updated' }

      // When: updating entities with different parameters
      const entityApi = await Effect.runPromise(
        Effect.gen(function* () {
          return yield* EntityApi
        }).pipe(Effect.provide(EntityApiMock)),
      )

      const result1 = await Effect.runPromise(
        entityApi
          .update('organization', 'id-1', body)
          .pipe(Effect.provide(EntityApiMock)),
      )

      const result2 = await Effect.runPromise(
        entityApi
          .update('user', 'id-2', { email: 'test@example.com' })
          .pipe(Effect.provide(EntityApiMock)),
      )

      // Then: should return empty objects (mock ignores parameters)
      expect(result1).toEqual({})
      expect(result2).toEqual({})
    })
  })

  describe('delete behavior', () => {
    test('should return void by default', async () => {
      // Given: EntityApiMock layer
      // When: deleting an entity
      const entityApi = await Effect.runPromise(
        Effect.gen(function* () {
          return yield* EntityApi
        }).pipe(Effect.provide(EntityApiMock)),
      )

      const result = await Effect.runPromise(
        entityApi.delete('organization', 'id-123').pipe(Effect.provide(EntityApiMock)),
      )

      // Then: should return undefined (void)
      expect(result).toBeUndefined()
    })

    test('should accept entityId and id parameters', async () => {
      // Given: EntityApiMock layer
      // When: deleting entities with different parameters
      const entityApi = await Effect.runPromise(
        Effect.gen(function* () {
          return yield* EntityApi
        }).pipe(Effect.provide(EntityApiMock)),
      )

      const result1 = await Effect.runPromise(
        entityApi.delete('organization', 'id-1').pipe(Effect.provide(EntityApiMock)),
      )

      const result2 = await Effect.runPromise(
        entityApi.delete('user', 'id-2').pipe(Effect.provide(EntityApiMock)),
      )

      // Then: should return undefined for both (mock ignores parameters)
      expect(result1).toBeUndefined()
      expect(result2).toBeUndefined()
    })
  })

  describe('service interface', () => {
    test('should export EntityApi tag', () => {
      // Given the EntityApi module
      // When I check the EntityApi export
      // Then it should be a Context tag
      expect(EntityApi).toBeDefined()
      expect(typeof EntityApi).toBe('object')
    })

    test('should export EntityApiService interface', () => {
      // Given the EntityApi module
      // When I use EntityApiService type
      // Then it should be usable
      const service: EntityApiService = {
        list: () => Effect.succeed({ data: [] }),
        get: () => Effect.succeed({}),
        create: () => Effect.succeed({}),
        update: () => Effect.succeed({}),
        delete: () => Effect.void,
      }
      expect(service).toBeDefined()
      expect(typeof service.list).toBe('function')
      expect(typeof service.get).toBe('function')
      expect(typeof service.create).toBe('function')
      expect(typeof service.update).toBe('function')
      expect(typeof service.delete).toBe('function')
    })

    test('should enforce EntityApiService interface contract', async () => {
      // Given an EntityApiService implementation
      // When I call methods
      // Then they must return correct Effect types
      const service: EntityApiService = {
        list: () => Effect.succeed({ data: [] }),
        get: () => Effect.succeed({}),
        create: () => Effect.succeed({}),
        update: () => Effect.succeed({}),
        delete: () => Effect.void,
      }

      const listResult = await Effect.runPromise(
        service.list('organization'),
      )
      expect(listResult).toHaveProperty('data')
      expect(Array.isArray(listResult.data)).toBe(true)

      const getResult = await Effect.runPromise(service.get('organization', 'id'))
      expect(typeof getResult).toBe('object')

      const createResult = await Effect.runPromise(
        service.create('organization', {}),
      )
      expect(typeof createResult).toBe('object')

      const updateResult = await Effect.runPromise(
        service.update('organization', 'id', {}),
      )
      expect(typeof updateResult).toBe('object')

      const deleteResult = await Effect.runPromise(
        service.delete('organization', 'id'),
      )
      expect(deleteResult).toBeUndefined()
    })
  })

  describe('EntityApiMock layer behavior', () => {
    test('should provide EntityApi service', async () => {
      // Given EntityApiMock layer
      // When I use it in an Effect program
      const program = Effect.gen(function* () {
        const service = yield* EntityApi
        expect(service).toBeDefined()
        expect(typeof service.list).toBe('function')
        expect(typeof service.get).toBe('function')
        expect(typeof service.create).toBe('function')
        expect(typeof service.update).toBe('function')
        expect(typeof service.delete).toBe('function')
        return service
      })

      // Then it should provide a valid EntityApi service
      const service = await Effect.runPromise(
        program.pipe(Effect.provide(EntityApiMock)),
      )
      expect(service).toBeDefined()
    })

    test('should work with multiple service accesses', async () => {
      // Given EntityApiMock layer
      // When I access the service multiple times
      const program = Effect.gen(function* () {
        const service1 = yield* EntityApi
        const service2 = yield* EntityApi
        return { service1, service2 }
      })

      const { service1, service2 } = await Effect.runPromise(
        program.pipe(Effect.provide(EntityApiMock)),
      )

      // Then it should return the same instance (Layer.succeed behavior)
      expect(service1).toBe(service2)
    })

    test('should be composable with other layers', async () => {
      // Given EntityApiMock layer
      const combinedLayer = EntityApiMock

      // When I use it in an Effect program
      const program = Effect.gen(function* () {
        const service = yield* EntityApi
        const result = yield* service.list('organization')
        return result
      })

      // Then it should work correctly
      const result = await Effect.runPromise(
        program.pipe(Effect.provide(combinedLayer)),
      )
      expect(result.data).toEqual([])
    })
  })

  describe('integration behavior', () => {
    test('should support full CRUD flow', async () => {
      // Given EntityApiMock layer
      // When I perform a full CRUD flow
      const program = Effect.gen(function* () {
        const api = yield* EntityApi

        // Create
        const created = yield* api.create('organization', {
          name: 'Test Org',
          slug: 'test-org',
        })
        expect(created).toEqual({})

        // Read (list)
        const list = yield* api.list('organization')
        expect(list.data).toEqual([])

        // Read (get)
        const got = yield* api.get('organization', 'id-123')
        expect(got).toEqual({})

        // Update
        const updated = yield* api.update('organization', 'id-123', {
          name: 'Updated Org',
        })
        expect(updated).toEqual({})

        // Delete
        yield* api.delete('organization', 'id-123')

        return { created, list, got, updated }
      })

      // Then it should complete successfully
      const result = await Effect.runPromise(
        program.pipe(Effect.provide(EntityApiMock)),
      )
      expect(result.created).toEqual({})
      expect(result.list.data).toEqual([])
      expect(result.got).toEqual({})
      expect(result.updated).toEqual({})
    })
  })
})
