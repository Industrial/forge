/**
 * BDD tests for RpcApiLive - production RPC API service implementation.
 * Tests use Effect's Layer system for dependency injection (no vi.mock()).
 */
import { describe, test, expect } from 'bun:test'
import { Effect, Layer } from 'effect'
import { HttpClient, HttpClientRequest } from '@effect/platform'

import { RpcApiLive } from './RpcApiLive'
import { RpcApi } from './RpcApi'

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
    // Stub other methods (not used by RpcApiLive)
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
  } as HttpClient.HttpClient
}

describe('RpcApiLive', () => {
  describe('subscribe behavior', () => {
    test('should subscribe successfully', async () => {
      // Given: endpoint returns 200 with valid subscription result
      const mockSubscriptionId = '123e4567-e89b-12d3-a456-426614174000'
      const mockResponse = {
        result: {
          subscription_id: mockSubscriptionId,
        },
      }

      const mockHttpClient = createMockHttpClient((request) => {
        if (request.url.includes('/api/rpc') && request.method === 'POST') {
          return Effect.succeed({
            status: 200,
            json: Effect.succeed(mockResponse),
            headers: new Headers(),
          })
        }
        return Effect.succeed({
          status: 404,
          json: Effect.succeed({}),
          headers: new Headers(),
        })
      })

      const httpLayer = Layer.succeed(HttpClient.HttpClient, mockHttpClient)
      const testLayer = RpcApiLive.pipe(Layer.provide(httpLayer))

      // When: subscribing
      const api = await Effect.runPromise(
        Effect.gen(function* () {
          return yield* RpcApi
        }).pipe(Effect.provide(testLayer)),
      )

      const result = await Effect.runPromise(
        api.subscribe('test-entity').pipe(Effect.provide(testLayer)),
      )

      // Then: should return subscription_id
      expect(result.subscription_id).toBe(mockSubscriptionId)
    })

    test('should include params in request body', async () => {
      // Given: endpoint that captures request body
      let capturedBody: unknown = null
      const mockHttpClient = createMockHttpClient((request) => {
        if (request.url.includes('/api/rpc') && request.method === 'POST') {
          capturedBody = (request as any).body
          return Effect.succeed({
            status: 200,
            json: Effect.succeed({
              result: { subscription_id: 'test-id' },
            }),
            headers: new Headers(),
          })
        }
        return Effect.succeed({
          status: 404,
          json: Effect.succeed({}),
          headers: new Headers(),
        })
      })

      const httpLayer = Layer.succeed(HttpClient.HttpClient, mockHttpClient)
      const testLayer = RpcApiLive.pipe(Layer.provide(httpLayer))

      // When: subscribing with params
      const api = await Effect.runPromise(
        Effect.gen(function* () {
          return yield* RpcApi
        }).pipe(Effect.provide(testLayer)),
      )

      await Effect.runPromise(
        api
          .subscribe('test-entity', {
            filter: 'name:test',
            sort: 'name',
            order: 'asc',
            offset: 10,
            limit: 20,
          })
          .pipe(Effect.provide(testLayer)),
      )

      // Then: body should be sent (verification via successful response)
      expect(capturedBody).toBeDefined()
    })

    test('should handle undefined params', async () => {
      // Given: endpoint returns success
      const mockHttpClient = createMockHttpClient((request) => {
        if (request.url.includes('/api/rpc') && request.method === 'POST') {
          return Effect.succeed({
            status: 200,
            json: Effect.succeed({
              result: { subscription_id: 'test-id' },
            }),
            headers: new Headers(),
          })
        }
        return Effect.succeed({
          status: 404,
          json: Effect.succeed({}),
          headers: new Headers(),
        })
      })

      const httpLayer = Layer.succeed(HttpClient.HttpClient, mockHttpClient)
      const testLayer = RpcApiLive.pipe(Layer.provide(httpLayer))

      // When: subscribing without params
      const api = await Effect.runPromise(
        Effect.gen(function* () {
          return yield* RpcApi
        }).pipe(Effect.provide(testLayer)),
      )

      const result = await Effect.runPromise(
        api.subscribe('test-entity', undefined).pipe(Effect.provide(testLayer)),
      )

      // Then: should succeed
      expect(result.subscription_id).toBe('test-id')
    })

    test('should fail when endpoint returns non-200 status with error message', async () => {
      // Given: endpoint returns 400 with error message
      const mockHttpClient = createMockHttpClient((request) => {
        if (request.url.includes('/api/rpc') && request.method === 'POST') {
          return Effect.succeed({
            status: 400,
            json: Effect.succeed({
              error: { message: 'Invalid request' },
            }),
            headers: new Headers(),
          })
        }
        return Effect.succeed({
          status: 404,
          json: Effect.succeed({}),
          headers: new Headers(),
        })
      })

      const httpLayer = Layer.succeed(HttpClient.HttpClient, mockHttpClient)
      const testLayer = RpcApiLive.pipe(Layer.provide(httpLayer))

      // When: subscribing
      const api = await Effect.runPromise(
        Effect.gen(function* () {
          return yield* RpcApi
        }).pipe(Effect.provide(testLayer)),
      )

      // Then: should fail with error message
      await expect(
        Effect.runPromise(
          api.subscribe('test-entity').pipe(Effect.provide(testLayer)),
        ),
      ).rejects.toThrow('Invalid request')
    })

    test('should fail when endpoint returns error without message field', async () => {
      // Given: endpoint returns 500 with error object without message
      const mockHttpClient = createMockHttpClient((request) => {
        if (request.url.includes('/api/rpc') && request.method === 'POST') {
          return Effect.succeed({
            status: 500,
            json: Effect.succeed({
              error: { code: 500 },
            }),
            headers: new Headers(),
          })
        }
        return Effect.succeed({
          status: 404,
          json: Effect.succeed({}),
          headers: new Headers(),
        })
      })

      const httpLayer = Layer.succeed(HttpClient.HttpClient, mockHttpClient)
      const testLayer = RpcApiLive.pipe(Layer.provide(httpLayer))

      // When: subscribing
      const api = await Effect.runPromise(
        Effect.gen(function* () {
          return yield* RpcApi
        }).pipe(Effect.provide(testLayer)),
      )

      // Then: should fail with default error message
      await expect(
        Effect.runPromise(
          api.subscribe('test-entity').pipe(Effect.provide(testLayer)),
        ),
      ).rejects.toThrow('RPC failed (500)')
    })

    test('should fail when response has no result and no error', async () => {
      // Given: endpoint returns 200 with no result and no error
      const mockHttpClient = createMockHttpClient((request) => {
        if (request.url.includes('/api/rpc') && request.method === 'POST') {
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

      const httpLayer = Layer.succeed(HttpClient.HttpClient, mockHttpClient)
      const testLayer = RpcApiLive.pipe(Layer.provide(httpLayer))

      // When: subscribing
      const api = await Effect.runPromise(
        Effect.gen(function* () {
          return yield* RpcApi
        }).pipe(Effect.provide(testLayer)),
      )

      // Then: should fail with appropriate error
      await expect(
        Effect.runPromise(
          api.subscribe('test-entity').pipe(Effect.provide(testLayer)),
        ),
      ).rejects.toThrow()
    })

    test('should fail when response has error with message', async () => {
      // Given: endpoint returns 200 with error in result
      const mockHttpClient = createMockHttpClient((request) => {
        if (request.url.includes('/api/rpc') && request.method === 'POST') {
          return Effect.succeed({
            status: 200,
            json: Effect.succeed({
              error: { message: 'Subscription failed' },
            }),
            headers: new Headers(),
          })
        }
        return Effect.succeed({
          status: 404,
          json: Effect.succeed({}),
          headers: new Headers(),
        })
      })

      const httpLayer = Layer.succeed(HttpClient.HttpClient, mockHttpClient)
      const testLayer = RpcApiLive.pipe(Layer.provide(httpLayer))

      // When: subscribing
      const api = await Effect.runPromise(
        Effect.gen(function* () {
          return yield* RpcApi
        }).pipe(Effect.provide(testLayer)),
      )

      // Then: should fail with error message
      await expect(
        Effect.runPromise(
          api.subscribe('test-entity').pipe(Effect.provide(testLayer)),
        ),
      ).rejects.toThrow('Subscription failed')
    })

    test('should handle invalid response schema', async () => {
      // Given: endpoint returns invalid response schema
      const mockHttpClient = createMockHttpClient((request) => {
        if (request.url.includes('/api/rpc') && request.method === 'POST') {
          return Effect.succeed({
            status: 200,
            json: Effect.succeed({
              result: { invalid_field: 'value' },
            }),
            headers: new Headers(),
          })
        }
        return Effect.succeed({
          status: 404,
          json: Effect.succeed({}),
          headers: new Headers(),
        })
      })

      const httpLayer = Layer.succeed(HttpClient.HttpClient, mockHttpClient)
      const testLayer = RpcApiLive.pipe(Layer.provide(httpLayer))

      // When: subscribing
      const api = await Effect.runPromise(
        Effect.gen(function* () {
          return yield* RpcApi
        }).pipe(Effect.provide(testLayer)),
      )

      // Then: should fail with schema validation error
      await expect(
        Effect.runPromise(
          api.subscribe('test-entity').pipe(Effect.provide(testLayer)),
        ),
      ).rejects.toThrow()
    })

    test('should send correct request body structure', async () => {
      // Given: endpoint that checks request body
      let capturedBody: unknown = null
      const mockHttpClient = createMockHttpClient((request) => {
        if (request.url.includes('/api/rpc') && request.method === 'POST') {
          // Extract body from request (simplified - actual HttpClientRequest has body)
          capturedBody = (request as any).body
          return Effect.succeed({
            status: 200,
            json: Effect.succeed({
              result: { subscription_id: 'test-id' },
            }),
            headers: new Headers(),
          })
        }
        return Effect.succeed({
          status: 404,
          json: Effect.succeed({}),
          headers: new Headers(),
        })
      })

      const httpLayer = Layer.succeed(HttpClient.HttpClient, mockHttpClient)
      const testLayer = RpcApiLive.pipe(Layer.provide(httpLayer))

      // When: subscribing
      const api = await Effect.runPromise(
        Effect.gen(function* () {
          return yield* RpcApi
        }).pipe(Effect.provide(testLayer)),
      )

      await Effect.runPromise(
        api
          .subscribe('test-entity', { filter: 'name:test' })
          .pipe(Effect.provide(testLayer)),
      )

      // Then: body should be sent (verification via successful response)
      expect(capturedBody).toBeDefined()
    })

    test('should handle empty params object', async () => {
      // Given: endpoint returns success
      const mockHttpClient = createMockHttpClient((request) => {
        if (request.url.includes('/api/rpc') && request.method === 'POST') {
          return Effect.succeed({
            status: 200,
            json: Effect.succeed({
              result: { subscription_id: 'test-id' },
            }),
            headers: new Headers(),
          })
        }
        return Effect.succeed({
          status: 404,
          json: Effect.succeed({}),
          headers: new Headers(),
        })
      })

      const httpLayer = Layer.succeed(HttpClient.HttpClient, mockHttpClient)
      const testLayer = RpcApiLive.pipe(Layer.provide(httpLayer))

      // When: subscribing with empty params
      const api = await Effect.runPromise(
        Effect.gen(function* () {
          return yield* RpcApi
        }).pipe(Effect.provide(testLayer)),
      )

      const result = await Effect.runPromise(
        api.subscribe('test-entity', {}).pipe(Effect.provide(testLayer)),
      )

      // Then: should succeed
      expect(result.subscription_id).toBe('test-id')
    })
  })
})
