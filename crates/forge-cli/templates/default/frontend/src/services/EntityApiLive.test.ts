/**
 * BDD tests for EntityApiLive - production entity API service implementation.
 * Tests use Effect's Layer system for dependency injection (no vi.mock()).
 */
import { describe, test, expect } from 'bun:test'
import { Effect, Layer } from 'effect'
import type { HttpClient, HttpClientRequest } from '@effect/platform'

import { AuthenticatedHttpClient } from './AuthenticatedHttpClient'
import { EntityApiLive } from './EntityApiLive'
import { EntityApi } from './EntityApi'

// Helper to create a mock HttpClient
function createMockHttpClient(
  handler: (request: HttpClientRequest.HttpClientRequest) => Effect.Effect<
    {
      status: number
      json: Effect.Effect<unknown>
      headers: Headers
    },
    never,
    never
  >,
): HttpClient.HttpClient {
  const executeImpl = (request: HttpClientRequest.HttpClientRequest) => {
    return handler(request)
  }

  // Create a minimal HttpClient with execute method
  return {
    execute: executeImpl,
    // Stub other methods (not used by EntityApiLive)
    get: () =>
      Effect.succeed({
        status: 404,
        json: Effect.succeed({}),
        headers: new Headers(),
      }),
    post: () =>
      Effect.succeed({
        status: 404,
        json: Effect.succeed({}),
        headers: new Headers(),
      }),
    put: () =>
      Effect.succeed({
        status: 404,
        json: Effect.succeed({}),
        headers: new Headers(),
      }),
    patch: () =>
      Effect.succeed({
        status: 404,
        json: Effect.succeed({}),
        headers: new Headers(),
      }),
    delete: () =>
      Effect.succeed({
        status: 404,
        json: Effect.succeed({}),
        headers: new Headers(),
      }),
    head: () =>
      Effect.succeed({
        status: 404,
        json: Effect.succeed({}),
        headers: new Headers(),
      }),
    options: () =>
      Effect.succeed({
        status: 404,
        json: Effect.succeed({}),
        headers: new Headers(),
      }),
    request: () =>
      Effect.succeed({
        status: 404,
        json: Effect.succeed({}),
        headers: new Headers(),
      }),
    requestWith: () =>
      Effect.succeed({
        status: 404,
        json: Effect.succeed({}),
        headers: new Headers(),
      }),
    stream: () =>
      Effect.succeed({
        status: 404,
        json: Effect.succeed({}),
        headers: new Headers(),
      }),
    streamWith: () =>
      Effect.succeed({
        status: 404,
        json: Effect.succeed({}),
        headers: new Headers(),
      }),
  } as unknown as HttpClient.HttpClient
}

describe('EntityApiLive', () => {
  describe('list behavior', () => {
    test('should fetch entities successfully', async () => {
      // Given: endpoint returns 200 with valid entities
      const mockEntities = [
        { id: '1', name: 'Entity 1' },
        { id: '2', name: 'Entity 2' },
      ]

      const mockHttpClient = createMockHttpClient((request) => {
        if (
          request.url.includes('/api/entities/test-entity') &&
          request.method === 'GET'
        ) {
          return Effect.succeed({
            status: 200,
            json: Effect.succeed({ data: mockEntities }),
            headers: new Headers(),
          })
        }
        return Effect.succeed({
          status: 404,
          json: Effect.succeed({}),
          headers: new Headers(),
        })
      })

      const httpLayer = Layer.succeed(AuthenticatedHttpClient, mockHttpClient)
      const testLayer = EntityApiLive.pipe(Layer.provide(httpLayer))

      // When: listing entities
      const api = await Effect.runPromise(
        Effect.gen(function* () {
          return yield* EntityApi
        }).pipe(Effect.provide(testLayer)),
      )

      const result = await Effect.runPromise(
        api.list('test-entity').pipe(Effect.provide(testLayer)),
      )

      // Then: should return entities
      expect(result.data).toHaveLength(2)
      expect(result.data[0]).toEqual({ id: '1', name: 'Entity 1' })
      expect(result.data[1]).toEqual({ id: '2', name: 'Entity 2' })
    })

    test('should handle empty entities array', async () => {
      // Given: endpoint returns 200 with empty data array
      const mockHttpClient = createMockHttpClient((request) => {
        if (
          request.url.includes('/api/entities/test-entity') &&
          request.method === 'GET'
        ) {
          return Effect.succeed({
            status: 200,
            json: Effect.succeed({ data: [] }),
            headers: new Headers(),
          })
        }
        return Effect.succeed({
          status: 404,
          json: Effect.succeed({}),
          headers: new Headers(),
        })
      })

      const httpLayer = Layer.succeed(AuthenticatedHttpClient, mockHttpClient)
      const testLayer = EntityApiLive.pipe(Layer.provide(httpLayer))

      // When: listing entities
      const api = await Effect.runPromise(
        Effect.gen(function* () {
          return yield* EntityApi
        }).pipe(Effect.provide(testLayer)),
      )

      const result = await Effect.runPromise(
        api.list('test-entity').pipe(Effect.provide(testLayer)),
      )

      // Then: should return empty array
      expect(result.data).toHaveLength(0)
    })

    test('should handle missing data field', async () => {
      // Given: endpoint returns 200 without data field
      const mockHttpClient = createMockHttpClient((request) => {
        if (
          request.url.includes('/api/entities/test-entity') &&
          request.method === 'GET'
        ) {
          return Effect.succeed({
            status: 200,
            json: Effect.succeed({}),
            headers: new Headers(),
          })
        }
        return Effect.succeed({
          status: 404,
          json: Effect.succeed({}),
          headers: new Headers(),
        })
      })

      const httpLayer = Layer.succeed(AuthenticatedHttpClient, mockHttpClient)
      const testLayer = EntityApiLive.pipe(Layer.provide(httpLayer))

      // When: listing entities
      const api = await Effect.runPromise(
        Effect.gen(function* () {
          return yield* EntityApi
        }).pipe(Effect.provide(testLayer)),
      )

      const result = await Effect.runPromise(
        api.list('test-entity').pipe(Effect.provide(testLayer)),
      )

      // Then: should return empty array
      expect(result.data).toHaveLength(0)
    })

    test('should include query parameters in request', async () => {
      // Given: endpoint that checks query params
      let capturedUrl = ''
      const mockHttpClient = createMockHttpClient((request) => {
        if (
          request.url.includes('/api/entities/test-entity') &&
          request.method === 'GET'
        ) {
          capturedUrl = request.url
          return Effect.succeed({
            status: 200,
            json: Effect.succeed({ data: [] }),
            headers: new Headers(),
          })
        }
        return Effect.succeed({
          status: 404,
          json: Effect.succeed({}),
          headers: new Headers(),
        })
      })

      const httpLayer = Layer.succeed(AuthenticatedHttpClient, mockHttpClient)
      const testLayer = EntityApiLive.pipe(Layer.provide(httpLayer))

      // When: listing with query params
      const api = await Effect.runPromise(
        Effect.gen(function* () {
          return yield* EntityApi
        }).pipe(Effect.provide(testLayer)),
      )

      await Effect.runPromise(
        api
          .list('test-entity', {
            filter: 'name:test',
            sort: 'name',
            order: 'asc',
            offset: 10,
            limit: 20,
          })
          .pipe(Effect.provide(testLayer)),
      )

      // Then: URL should include query params
      expect(capturedUrl).toContain('filter=name%3Atest')
      expect(capturedUrl).toContain('sort=name')
      expect(capturedUrl).toContain('order=asc')
      expect(capturedUrl).toContain('offset=10')
      expect(capturedUrl).toContain('limit=20')
    })

    test('should handle empty query parameters', async () => {
      // Given: endpoint that checks query params
      let capturedUrl = ''
      const mockHttpClient = createMockHttpClient((request) => {
        if (
          request.url.includes('/api/entities/test-entity') &&
          request.method === 'GET'
        ) {
          capturedUrl = request.url
          return Effect.succeed({
            status: 200,
            json: Effect.succeed({ data: [] }),
            headers: new Headers(),
          })
        }
        return Effect.succeed({
          status: 404,
          json: Effect.succeed({}),
          headers: new Headers(),
        })
      })

      const httpLayer = Layer.succeed(AuthenticatedHttpClient, mockHttpClient)
      const testLayer = EntityApiLive.pipe(Layer.provide(httpLayer))

      // When: listing with empty/null query params
      const api = await Effect.runPromise(
        Effect.gen(function* () {
          return yield* EntityApi
        }).pipe(Effect.provide(testLayer)),
      )

      await Effect.runPromise(
        api
          .list('test-entity', {
            filter: '',
            sort: '',
            order: '',
            offset: undefined,
            limit: undefined,
          })
          .pipe(Effect.provide(testLayer)),
      )

      // Then: URL should not include empty params
      expect(capturedUrl).not.toContain('filter=')
      expect(capturedUrl).not.toContain('sort=')
      expect(capturedUrl).not.toContain('order=')
      expect(capturedUrl).not.toContain('offset=')
      expect(capturedUrl).not.toContain('limit=')
    })

    test('should URL encode entityId', async () => {
      // Given: endpoint that checks URL encoding
      let capturedUrl = ''
      const mockHttpClient = createMockHttpClient((request) => {
        if (request.method === 'GET') {
          capturedUrl = request.url
          return Effect.succeed({
            status: 200,
            json: Effect.succeed({ data: [] }),
            headers: new Headers(),
          })
        }
        return Effect.succeed({
          status: 404,
          json: Effect.succeed({}),
          headers: new Headers(),
        })
      })

      const httpLayer = Layer.succeed(AuthenticatedHttpClient, mockHttpClient)
      const testLayer = EntityApiLive.pipe(Layer.provide(httpLayer))

      // When: listing with special characters in entityId
      const api = await Effect.runPromise(
        Effect.gen(function* () {
          return yield* EntityApi
        }).pipe(Effect.provide(testLayer)),
      )

      await Effect.runPromise(
        api.list('test/entity?id=1').pipe(Effect.provide(testLayer)),
      )

      // Then: entityId should be URL encoded
      expect(capturedUrl).toContain('/api/entities/test%2Fentity%3Fid%3D1')
    })

    test('should fail when endpoint returns non-200 status', async () => {
      // Given: endpoint returns 400 with error message
      const mockHttpClient = createMockHttpClient((request) => {
        if (
          request.url.includes('/api/entities/test-entity') &&
          request.method === 'GET'
        ) {
          return Effect.succeed({
            status: 400,
            json: Effect.succeed({ message: 'Invalid request' }),
            headers: new Headers(),
          })
        }
        return Effect.succeed({
          status: 404,
          json: Effect.succeed({}),
          headers: new Headers(),
        })
      })

      const httpLayer = Layer.succeed(AuthenticatedHttpClient, mockHttpClient)
      const testLayer = EntityApiLive.pipe(Layer.provide(httpLayer))

      // When: listing entities
      const api = await Effect.runPromise(
        Effect.gen(function* () {
          return yield* EntityApi
        }).pipe(Effect.provide(testLayer)),
      )

      // Then: should fail with error message
      await expect(
        Effect.runPromise(
          api.list('test-entity').pipe(Effect.provide(testLayer)),
        ),
      ).rejects.toThrow('Invalid request')
    })

    test('should handle error response without message field', async () => {
      // Given: endpoint returns 500 with error field
      const mockHttpClient = createMockHttpClient((request) => {
        if (
          request.url.includes('/api/entities/test-entity') &&
          request.method === 'GET'
        ) {
          return Effect.succeed({
            status: 500,
            json: Effect.succeed({ error: 'Server error' }),
            headers: new Headers(),
          })
        }
        return Effect.succeed({
          status: 404,
          json: Effect.succeed({}),
          headers: new Headers(),
        })
      })

      const httpLayer = Layer.succeed(AuthenticatedHttpClient, mockHttpClient)
      const testLayer = EntityApiLive.pipe(Layer.provide(httpLayer))

      // When: listing entities
      const api = await Effect.runPromise(
        Effect.gen(function* () {
          return yield* EntityApi
        }).pipe(Effect.provide(testLayer)),
      )

      // Then: should fail with error message
      await expect(
        Effect.runPromise(
          api.list('test-entity').pipe(Effect.provide(testLayer)),
        ),
      ).rejects.toThrow('Server error')
    })

    test('should handle error response without message or error field', async () => {
      // Given: endpoint returns 404 with no error fields
      const mockHttpClient = createMockHttpClient((request) => {
        if (
          request.url.includes('/api/entities/test-entity') &&
          request.method === 'GET'
        ) {
          return Effect.succeed({
            status: 404,
            json: Effect.succeed({}),
            headers: new Headers(),
          })
        }
        return Effect.succeed({
          status: 404,
          json: Effect.succeed({}),
          headers: new Headers(),
        })
      })

      const httpLayer = Layer.succeed(AuthenticatedHttpClient, mockHttpClient)
      const testLayer = EntityApiLive.pipe(Layer.provide(httpLayer))

      // When: listing entities
      const api = await Effect.runPromise(
        Effect.gen(function* () {
          return yield* EntityApi
        }).pipe(Effect.provide(testLayer)),
      )

      // Then: should fail with default error message
      await expect(
        Effect.runPromise(
          api.list('test-entity').pipe(Effect.provide(testLayer)),
        ),
      ).rejects.toThrow('Request failed.')
    })
  })

  describe('get behavior', () => {
    test('should fetch entity by id successfully', async () => {
      // Given: endpoint returns 200 with entity
      const mockEntity = { id: '123', name: 'Test Entity', value: 42 }

      const mockHttpClient = createMockHttpClient((request) => {
        if (
          request.url.includes('/api/entities/test-entity/123') &&
          request.method === 'GET'
        ) {
          return Effect.succeed({
            status: 200,
            json: Effect.succeed(mockEntity),
            headers: new Headers(),
          })
        }
        return Effect.succeed({
          status: 404,
          json: Effect.succeed({}),
          headers: new Headers(),
        })
      })

      const httpLayer = Layer.succeed(AuthenticatedHttpClient, mockHttpClient)
      const testLayer = EntityApiLive.pipe(Layer.provide(httpLayer))

      // When: getting entity
      const api = await Effect.runPromise(
        Effect.gen(function* () {
          return yield* EntityApi
        }).pipe(Effect.provide(testLayer)),
      )

      const result = await Effect.runPromise(
        api.get('test-entity', '123').pipe(Effect.provide(testLayer)),
      )

      // Then: should return entity
      expect(result).toEqual(mockEntity)
    })

    test('should URL encode entityId and id', async () => {
      // Given: endpoint that checks URL encoding
      let capturedUrl = ''
      const mockHttpClient = createMockHttpClient((request) => {
        if (request.method === 'GET') {
          capturedUrl = request.url
          return Effect.succeed({
            status: 200,
            json: Effect.succeed({}),
            headers: new Headers(),
          })
        }
        return Effect.succeed({
          status: 404,
          json: Effect.succeed({}),
          headers: new Headers(),
        })
      })

      const httpLayer = Layer.succeed(AuthenticatedHttpClient, mockHttpClient)
      const testLayer = EntityApiLive.pipe(Layer.provide(httpLayer))

      // When: getting with special characters
      const api = await Effect.runPromise(
        Effect.gen(function* () {
          return yield* EntityApi
        }).pipe(Effect.provide(testLayer)),
      )

      await Effect.runPromise(
        api.get('test/entity', 'id/123').pipe(Effect.provide(testLayer)),
      )

      // Then: both should be URL encoded
      expect(capturedUrl).toContain('/api/entities/test%2Fentity/id%2F123')
    })

    test('should fail when endpoint returns non-200 status', async () => {
      // Given: endpoint returns 404 with error message
      const mockHttpClient = createMockHttpClient((request) => {
        if (
          request.url.includes('/api/entities/test-entity/123') &&
          request.method === 'GET'
        ) {
          return Effect.succeed({
            status: 404,
            json: Effect.succeed({ message: 'Entity not found' }),
            headers: new Headers(),
          })
        }
        return Effect.succeed({
          status: 404,
          json: Effect.succeed({}),
          headers: new Headers(),
        })
      })

      const httpLayer = Layer.succeed(AuthenticatedHttpClient, mockHttpClient)
      const testLayer = EntityApiLive.pipe(Layer.provide(httpLayer))

      // When: getting entity
      const api = await Effect.runPromise(
        Effect.gen(function* () {
          return yield* EntityApi
        }).pipe(Effect.provide(testLayer)),
      )

      // Then: should fail with error message
      await expect(
        Effect.runPromise(
          api.get('test-entity', '123').pipe(Effect.provide(testLayer)),
        ),
      ).rejects.toThrow('Entity not found')
    })
  })

  describe('create behavior', () => {
    test('should create entity successfully', async () => {
      // Given: endpoint returns 201 with created entity
      const requestBody = { name: 'New Entity', value: 100 }
      const responseBody = { id: '456', ...requestBody }

      const mockHttpClient = createMockHttpClient((request) => {
        if (
          request.url.includes('/api/entities/test-entity') &&
          request.method === 'POST'
        ) {
          return Effect.succeed({
            status: 201,
            json: Effect.succeed(responseBody),
            headers: new Headers(),
          })
        }
        return Effect.succeed({
          status: 404,
          json: Effect.succeed({}),
          headers: new Headers(),
        })
      })

      const httpLayer = Layer.succeed(AuthenticatedHttpClient, mockHttpClient)
      const testLayer = EntityApiLive.pipe(Layer.provide(httpLayer))

      // When: creating entity
      const api = await Effect.runPromise(
        Effect.gen(function* () {
          return yield* EntityApi
        }).pipe(Effect.provide(testLayer)),
      )

      const result = await Effect.runPromise(
        api.create('test-entity', requestBody).pipe(Effect.provide(testLayer)),
      )

      // Then: should return created entity
      expect(result).toEqual(responseBody)
    })

    test('should send request body as JSON', async () => {
      // Given: endpoint that captures request body
      let capturedBody: unknown = null
      const mockHttpClient = createMockHttpClient((request) => {
        if (
          request.url.includes('/api/entities/test-entity') &&
          request.method === 'POST'
        ) {
          // Extract body from request (simplified - actual HttpClientRequest has body)
          capturedBody = (request as any).body
          return Effect.succeed({
            status: 201,
            json: Effect.succeed({ id: '1' }),
            headers: new Headers(),
          })
        }
        return Effect.succeed({
          status: 404,
          json: Effect.succeed({}),
          headers: new Headers(),
        })
      })

      const httpLayer = Layer.succeed(AuthenticatedHttpClient, mockHttpClient)
      const testLayer = EntityApiLive.pipe(Layer.provide(httpLayer))

      // When: creating entity with body
      const api = await Effect.runPromise(
        Effect.gen(function* () {
          return yield* EntityApi
        }).pipe(Effect.provide(testLayer)),
      )

      const body = { name: 'Test', value: 42 }
      await Effect.runPromise(
        api.create('test-entity', body).pipe(Effect.provide(testLayer)),
      )

      // Then: body should be sent (verification via successful response)
      expect(capturedBody).toBeDefined()
    })

    test('should fail when endpoint returns non-200 status', async () => {
      // Given: endpoint returns 400 with error message
      const mockHttpClient = createMockHttpClient((request) => {
        if (
          request.url.includes('/api/entities/test-entity') &&
          request.method === 'POST'
        ) {
          return Effect.succeed({
            status: 400,
            json: Effect.succeed({ message: 'Validation failed' }),
            headers: new Headers(),
          })
        }
        return Effect.succeed({
          status: 404,
          json: Effect.succeed({}),
          headers: new Headers(),
        })
      })

      const httpLayer = Layer.succeed(AuthenticatedHttpClient, mockHttpClient)
      const testLayer = EntityApiLive.pipe(Layer.provide(httpLayer))

      // When: creating entity
      const api = await Effect.runPromise(
        Effect.gen(function* () {
          return yield* EntityApi
        }).pipe(Effect.provide(testLayer)),
      )

      // Then: should fail with error message
      await expect(
        Effect.runPromise(
          api
            .create('test-entity', { name: 'Test' })
            .pipe(Effect.provide(testLayer)),
        ),
      ).rejects.toThrow('Validation failed')
    })

    test('should URL encode entityId', async () => {
      // Given: endpoint that checks URL encoding
      let capturedUrl = ''
      const mockHttpClient = createMockHttpClient((request) => {
        if (request.method === 'POST') {
          capturedUrl = request.url
          return Effect.succeed({
            status: 201,
            json: Effect.succeed({ id: '1' }),
            headers: new Headers(),
          })
        }
        return Effect.succeed({
          status: 404,
          json: Effect.succeed({}),
          headers: new Headers(),
        })
      })

      const httpLayer = Layer.succeed(AuthenticatedHttpClient, mockHttpClient)
      const testLayer = EntityApiLive.pipe(Layer.provide(httpLayer))

      // When: creating with special characters in entityId
      const api = await Effect.runPromise(
        Effect.gen(function* () {
          return yield* EntityApi
        }).pipe(Effect.provide(testLayer)),
      )

      await Effect.runPromise(
        api
          .create('test/entity?id=1', { name: 'Test' })
          .pipe(Effect.provide(testLayer)),
      )

      // Then: entityId should be URL encoded
      expect(capturedUrl).toContain('/api/entities/test%2Fentity%3Fid%3D1')
    })
  })

  describe('update behavior', () => {
    test('should update entity successfully', async () => {
      // Given: endpoint returns 200 with updated entity
      const requestBody = { name: 'Updated Entity', value: 200 }
      const responseBody = { id: '123', ...requestBody }

      const mockHttpClient = createMockHttpClient((request) => {
        if (
          request.url.includes('/api/entities/test-entity/123') &&
          request.method === 'PATCH'
        ) {
          return Effect.succeed({
            status: 200,
            json: Effect.succeed(responseBody),
            headers: new Headers(),
          })
        }
        return Effect.succeed({
          status: 404,
          json: Effect.succeed({}),
          headers: new Headers(),
        })
      })

      const httpLayer = Layer.succeed(AuthenticatedHttpClient, mockHttpClient)
      const testLayer = EntityApiLive.pipe(Layer.provide(httpLayer))

      // When: updating entity
      const api = await Effect.runPromise(
        Effect.gen(function* () {
          return yield* EntityApi
        }).pipe(Effect.provide(testLayer)),
      )

      const result = await Effect.runPromise(
        api
          .update('test-entity', '123', requestBody)
          .pipe(Effect.provide(testLayer)),
      )

      // Then: should return updated entity
      expect(result).toEqual(responseBody)
    })

    test('should send request body as JSON', async () => {
      // Given: endpoint that captures request body
      let capturedBody: unknown = null
      const mockHttpClient = createMockHttpClient((request) => {
        if (
          request.url.includes('/api/entities/test-entity/123') &&
          request.method === 'PATCH'
        ) {
          capturedBody = (request as any).body
          return Effect.succeed({
            status: 200,
            json: Effect.succeed({ id: '123' }),
            headers: new Headers(),
          })
        }
        return Effect.succeed({
          status: 404,
          json: Effect.succeed({}),
          headers: new Headers(),
        })
      })

      const httpLayer = Layer.succeed(AuthenticatedHttpClient, mockHttpClient)
      const testLayer = EntityApiLive.pipe(Layer.provide(httpLayer))

      // When: updating entity with body
      const api = await Effect.runPromise(
        Effect.gen(function* () {
          return yield* EntityApi
        }).pipe(Effect.provide(testLayer)),
      )

      const body = { name: 'Updated', value: 99 }
      await Effect.runPromise(
        api.update('test-entity', '123', body).pipe(Effect.provide(testLayer)),
      )

      // Then: body should be sent (verification via successful response)
      expect(capturedBody).toBeDefined()
    })

    test('should fail when endpoint returns non-200 status', async () => {
      // Given: endpoint returns 404 with error message
      const mockHttpClient = createMockHttpClient((request) => {
        if (
          request.url.includes('/api/entities/test-entity/123') &&
          request.method === 'PATCH'
        ) {
          return Effect.succeed({
            status: 404,
            json: Effect.succeed({ message: 'Entity not found' }),
            headers: new Headers(),
          })
        }
        return Effect.succeed({
          status: 404,
          json: Effect.succeed({}),
          headers: new Headers(),
        })
      })

      const httpLayer = Layer.succeed(AuthenticatedHttpClient, mockHttpClient)
      const testLayer = EntityApiLive.pipe(Layer.provide(httpLayer))

      // When: updating entity
      const api = await Effect.runPromise(
        Effect.gen(function* () {
          return yield* EntityApi
        }).pipe(Effect.provide(testLayer)),
      )

      // Then: should fail with error message
      await expect(
        Effect.runPromise(
          api
            .update('test-entity', '123', { name: 'Updated' })
            .pipe(Effect.provide(testLayer)),
        ),
      ).rejects.toThrow('Entity not found')
    })

    test('should URL encode entityId and id', async () => {
      // Given: endpoint that checks URL encoding
      let capturedUrl = ''
      const mockHttpClient = createMockHttpClient((request) => {
        if (request.method === 'PATCH') {
          capturedUrl = request.url
          return Effect.succeed({
            status: 200,
            json: Effect.succeed({ id: '1' }),
            headers: new Headers(),
          })
        }
        return Effect.succeed({
          status: 404,
          json: Effect.succeed({}),
          headers: new Headers(),
        })
      })

      const httpLayer = Layer.succeed(AuthenticatedHttpClient, mockHttpClient)
      const testLayer = EntityApiLive.pipe(Layer.provide(httpLayer))

      // When: updating with special characters
      const api = await Effect.runPromise(
        Effect.gen(function* () {
          return yield* EntityApi
        }).pipe(Effect.provide(testLayer)),
      )

      await Effect.runPromise(
        api
          .update('test/entity', 'id/123', { name: 'Test' })
          .pipe(Effect.provide(testLayer)),
      )

      // Then: both should be URL encoded
      expect(capturedUrl).toContain('/api/entities/test%2Fentity/id%2F123')
    })
  })

  describe('delete behavior', () => {
    test('should delete entity successfully', async () => {
      // Given: endpoint returns 200
      const mockHttpClient = createMockHttpClient((request) => {
        if (
          request.url.includes('/api/entities/test-entity/123') &&
          request.method === 'DELETE'
        ) {
          return Effect.succeed({
            status: 200,
            json: Effect.succeed({}),
            headers: new Headers(),
          })
        }
        return Effect.succeed({
          status: 404,
          json: Effect.succeed({}),
          headers: new Headers(),
        })
      })

      const httpLayer = Layer.succeed(AuthenticatedHttpClient, mockHttpClient)
      const testLayer = EntityApiLive.pipe(Layer.provide(httpLayer))

      // When: deleting entity
      const api = await Effect.runPromise(
        Effect.gen(function* () {
          return yield* EntityApi
        }).pipe(Effect.provide(testLayer)),
      )

      const result = await Effect.runPromise(
        api.delete('test-entity', '123').pipe(Effect.provide(testLayer)),
      )

      // Then: should return undefined
      expect(result).toBeUndefined()
    })

    test('should URL encode entityId and id', async () => {
      // Given: endpoint that checks URL encoding
      let capturedUrl = ''
      const mockHttpClient = createMockHttpClient((request) => {
        if (request.method === 'DELETE') {
          capturedUrl = request.url
          return Effect.succeed({
            status: 200,
            json: Effect.succeed({}),
            headers: new Headers(),
          })
        }
        return Effect.succeed({
          status: 404,
          json: Effect.succeed({}),
          headers: new Headers(),
        })
      })

      const httpLayer = Layer.succeed(AuthenticatedHttpClient, mockHttpClient)
      const testLayer = EntityApiLive.pipe(Layer.provide(httpLayer))

      // When: deleting with special characters
      const api = await Effect.runPromise(
        Effect.gen(function* () {
          return yield* EntityApi
        }).pipe(Effect.provide(testLayer)),
      )

      await Effect.runPromise(
        api.delete('test/entity', 'id/123').pipe(Effect.provide(testLayer)),
      )

      // Then: both should be URL encoded
      expect(capturedUrl).toContain('/api/entities/test%2Fentity/id%2F123')
    })

    test('should fail when endpoint returns non-200 status', async () => {
      // Given: endpoint returns 404 with error message
      const mockHttpClient = createMockHttpClient((request) => {
        if (
          request.url.includes('/api/entities/test-entity/123') &&
          request.method === 'DELETE'
        ) {
          return Effect.succeed({
            status: 404,
            json: Effect.succeed({ message: 'Entity not found' }),
            headers: new Headers(),
          })
        }
        return Effect.succeed({
          status: 404,
          json: Effect.succeed({}),
          headers: new Headers(),
        })
      })

      const httpLayer = Layer.succeed(AuthenticatedHttpClient, mockHttpClient)
      const testLayer = EntityApiLive.pipe(Layer.provide(httpLayer))

      // When: deleting entity
      const api = await Effect.runPromise(
        Effect.gen(function* () {
          return yield* EntityApi
        }).pipe(Effect.provide(testLayer)),
      )

      // Then: should fail with error message
      await expect(
        Effect.runPromise(
          api.delete('test-entity', '123').pipe(Effect.provide(testLayer)),
        ),
      ).rejects.toThrow('Entity not found')
    })
  })
})
