/**
 * BDD tests for AuditLogLive - production audit log service implementation.
 * Tests use Effect's Layer system for dependency injection (no vi.mock()).
 */
import { describe, test, expect } from 'bun:test'
import { Effect, Layer } from 'effect'
import { HttpClient, HttpClientRequest } from '@effect/platform'

import { AuditLogLive } from './AuditLogLive'
import { AuditLog } from './AuditLog'
import { AuditLogEntry } from '../domain/AuditLogEntry'
import type { AuditLogListParams } from './AuditLog'

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
    // Stub other methods (not used by AuditLogLive)
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

describe('AuditLogLive', () => {
  describe('list behavior', () => {
    test('should fetch audit log entries successfully', async () => {
      // Given: endpoint returns 200 with valid entries
      const mockEntries = [
        {
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
        },
        {
          id: 'entry-2',
          event_kind: 'user.updated',
          actor_id: 'actor-2',
          action: 'update',
          resource_type: 'user',
          outcome: 'success',
          occurred_at: '2024-01-02T00:00:00Z',
        },
      ]

      const mockHttpClient = createMockHttpClient((request) => {
        if (request.url.includes('/api/dashboard/audit-log')) {
          return Effect.succeed({
            status: 200,
            json: Effect.succeed({
              entries: mockEntries,
              total: 2,
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
      const testLayer = AuditLogLive.pipe(Layer.provide(httpLayer))

      const params: AuditLogListParams = {
        limit: 10,
        offset: 0,
      }

      // When: listing audit log entries
      const auditLog = await Effect.runPromise(
        Effect.gen(function* () {
          return yield* AuditLog
        }).pipe(Effect.provide(testLayer)),
      )

      const result = await Effect.runPromise(
        auditLog.list(params).pipe(Effect.provide(testLayer)),
      )

      // Then: should return entries and total
      expect(result.entries).toHaveLength(2)
      expect(result.total).toBe(2)
      expect(result.entries[0]).toBeInstanceOf(AuditLogEntry)
      expect(result.entries[0].id).toBe('entry-1')
      expect(result.entries[0].event_kind).toBe('user.created')
      expect(result.entries[0].actor_id).toBe('actor-1')
      expect(result.entries[0].subject_id).toBe('subject-1')
      expect(result.entries[0].organization_id).toBe('org-1')
      expect(result.entries[0].action).toBe('create')
      expect(result.entries[0].resource_type).toBe('user')
      expect(result.entries[0].resource_id).toBe('resource-1')
      expect(result.entries[0].outcome).toBe('success')
      expect(result.entries[0].reason).toBe('User created via API')
      expect(result.entries[0].occurred_at).toBe('2024-01-01T00:00:00Z')
    })

    test('should include all query parameters in URL', async () => {
      // Given: params with all filter options
      const params: AuditLogListParams = {
        limit: 20,
        offset: 10,
        from: '2024-01-01',
        to: '2024-01-31',
        outcome: 'success',
        event_kind: 'user.created',
        action: 'create',
        reason: 'test reason',
      }

      let capturedUrl = ''

      const mockHttpClient = createMockHttpClient((request) => {
        if (request.url.includes('/api/dashboard/audit-log')) {
          capturedUrl = request.url
          return Effect.succeed({
            status: 200,
            json: Effect.succeed({ entries: [], total: 0 }),
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
      const testLayer = AuditLogLive.pipe(Layer.provide(httpLayer))

      // When: listing with all parameters
      const auditLog = await Effect.runPromise(
        Effect.gen(function* () {
          return yield* AuditLog
        }).pipe(Effect.provide(testLayer)),
      )

      await Effect.runPromise(
        auditLog.list(params).pipe(Effect.provide(testLayer)),
      )

      // Then: URL should contain all parameters
      expect(capturedUrl).toContain('limit=20')
      expect(capturedUrl).toContain('offset=10')
      expect(capturedUrl).toContain('from=2024-01-01')
      expect(capturedUrl).toContain('to=2024-01-31')
      expect(capturedUrl).toContain('outcome=success')
      expect(capturedUrl).toContain('event_kind=user.created')
      expect(capturedUrl).toContain('action=create')
      expect(capturedUrl).toContain('reason=test+reason')
    })

    test('should trim reason parameter before adding to URL', async () => {
      // Given: params with reason containing whitespace
      const params: AuditLogListParams = {
        limit: 10,
        offset: 0,
        reason: '  test reason  ',
      }

      let capturedUrl = ''

      const mockHttpClient = createMockHttpClient((request) => {
        if (request.url.includes('/api/dashboard/audit-log')) {
          capturedUrl = request.url
          return Effect.succeed({
            status: 200,
            json: Effect.succeed({ entries: [], total: 0 }),
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
      const testLayer = AuditLogLive.pipe(Layer.provide(httpLayer))

      // When: listing with trimmed reason
      const auditLog = await Effect.runPromise(
        Effect.gen(function* () {
          return yield* AuditLog
        }).pipe(Effect.provide(testLayer)),
      )

      await Effect.runPromise(
        auditLog.list(params).pipe(Effect.provide(testLayer)),
      )

      // Then: URL should contain trimmed reason (no leading/trailing spaces)
      expect(capturedUrl).toContain('reason=test+reason')
      expect(capturedUrl).not.toContain('reason=++test+reason++')
    })

    test('should not include reason parameter if empty after trim', async () => {
      // Given: params with reason that's only whitespace
      const params: AuditLogListParams = {
        limit: 10,
        offset: 0,
        reason: '   ',
      }

      let capturedUrl = ''

      const mockHttpClient = createMockHttpClient((request) => {
        if (request.url.includes('/api/dashboard/audit-log')) {
          capturedUrl = request.url
          return Effect.succeed({
            status: 200,
            json: Effect.succeed({ entries: [], total: 0 }),
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
      const testLayer = AuditLogLive.pipe(Layer.provide(httpLayer))

      // When: listing with whitespace-only reason
      const auditLog = await Effect.runPromise(
        Effect.gen(function* () {
          return yield* AuditLog
        }).pipe(Effect.provide(testLayer)),
      )

      await Effect.runPromise(
        auditLog.list(params).pipe(Effect.provide(testLayer)),
      )

      // Then: URL should not contain reason parameter
      expect(capturedUrl).not.toContain('reason=')
    })

    test('should handle empty entries array', async () => {
      // Given: endpoint returns 200 with empty entries
      const mockHttpClient = createMockHttpClient((request) => {
        if (request.url.includes('/api/dashboard/audit-log')) {
          return Effect.succeed({
            status: 200,
            json: Effect.succeed({
              entries: [],
              total: 0,
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
      const testLayer = AuditLogLive.pipe(Layer.provide(httpLayer))

      const params: AuditLogListParams = {
        limit: 10,
        offset: 0,
      }

      // When: listing entries
      const auditLog = await Effect.runPromise(
        Effect.gen(function* () {
          return yield* AuditLog
        }).pipe(Effect.provide(testLayer)),
      )

      const result = await Effect.runPromise(
        auditLog.list(params).pipe(Effect.provide(testLayer)),
      )

      // Then: should return empty entries
      expect(result.entries).toHaveLength(0)
      expect(result.total).toBe(0)
    })

    test('should handle missing entries field', async () => {
      // Given: endpoint returns 200 but missing entries field
      const mockHttpClient = createMockHttpClient((request) => {
        if (request.url.includes('/api/dashboard/audit-log')) {
          return Effect.succeed({
            status: 200,
            json: Effect.succeed({
              total: 0,
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
      const testLayer = AuditLogLive.pipe(Layer.provide(httpLayer))

      const params: AuditLogListParams = {
        limit: 10,
        offset: 0,
      }

      // When: listing entries
      const auditLog = await Effect.runPromise(
        Effect.gen(function* () {
          return yield* AuditLog
        }).pipe(Effect.provide(testLayer)),
      )

      const result = await Effect.runPromise(
        auditLog.list(params).pipe(Effect.provide(testLayer)),
      )

      // Then: should return empty entries array as default
      expect(result.entries).toHaveLength(0)
      expect(result.total).toBe(0)
    })

    test('should handle missing total field', async () => {
      // Given: endpoint returns 200 but missing total field
      const mockHttpClient = createMockHttpClient((request) => {
        if (request.url.includes('/api/dashboard/audit-log')) {
          return Effect.succeed({
            status: 200,
            json: Effect.succeed({
              entries: [],
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
      const testLayer = AuditLogLive.pipe(Layer.provide(httpLayer))

      const params: AuditLogListParams = {
        limit: 10,
        offset: 0,
      }

      // When: listing entries
      const auditLog = await Effect.runPromise(
        Effect.gen(function* () {
          return yield* AuditLog
        }).pipe(Effect.provide(testLayer)),
      )

      const result = await Effect.runPromise(
        auditLog.list(params).pipe(Effect.provide(testLayer)),
      )

      // Then: should return 0 as default total
      expect(result.total).toBe(0)
    })

    test('should handle entries with null optional fields', async () => {
      // Given: endpoint returns entries with null optional fields
      const mockEntries = [
        {
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
        },
      ]

      const mockHttpClient = createMockHttpClient((request) => {
        if (request.url.includes('/api/dashboard/audit-log')) {
          return Effect.succeed({
            status: 200,
            json: Effect.succeed({
              entries: mockEntries,
              total: 1,
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
      const testLayer = AuditLogLive.pipe(Layer.provide(httpLayer))

      const params: AuditLogListParams = {
        limit: 10,
        offset: 0,
      }

      // When: listing entries
      const auditLog = await Effect.runPromise(
        Effect.gen(function* () {
          return yield* AuditLog
        }).pipe(Effect.provide(testLayer)),
      )

      const result = await Effect.runPromise(
        auditLog.list(params).pipe(Effect.provide(testLayer)),
      )

      // Then: should handle null values correctly
      const entry = result.entries[0]
      expect(entry.subject_id).toBeNull()
      expect(entry.organization_id).toBeNull()
      expect(entry.resource_id).toBeNull()
      expect(entry.reason).toBeNull()
    })

    test('should fail when endpoint returns 403', async () => {
      // Given: endpoint returns 403
      const mockHttpClient = createMockHttpClient((request) => {
        if (request.url.includes('/api/dashboard/audit-log')) {
          return Effect.succeed({
            status: 403,
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
      const testLayer = AuditLogLive.pipe(Layer.provide(httpLayer))

      const params: AuditLogListParams = {
        limit: 10,
        offset: 0,
      }

      // When: listing entries
      const auditLog = await Effect.runPromise(
        Effect.gen(function* () {
          return yield* AuditLog
        }).pipe(Effect.provide(testLayer)),
      )

      // Then: should fail with permission error
      await expect(
        Effect.runPromise(
          auditLog.list(params).pipe(Effect.provide(testLayer)),
        ),
      ).rejects.toThrow('You do not have permission to view the audit log.')
    })

    test('should fail when endpoint returns non-200 status', async () => {
      // Given: endpoint returns 500
      const mockHttpClient = createMockHttpClient((request) => {
        if (request.url.includes('/api/dashboard/audit-log')) {
          return Effect.succeed({
            status: 500,
            json: Effect.succeed({ error: 'Internal server error' }),
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
      const testLayer = AuditLogLive.pipe(Layer.provide(httpLayer))

      const params: AuditLogListParams = {
        limit: 10,
        offset: 0,
      }

      // When: listing entries
      const auditLog = await Effect.runPromise(
        Effect.gen(function* () {
          return yield* AuditLog
        }).pipe(Effect.provide(testLayer)),
      )

      // Then: should fail with error message
      await expect(
        Effect.runPromise(
          auditLog.list(params).pipe(Effect.provide(testLayer)),
        ),
      ).rejects.toThrow('Internal server error')
    })

    test('should handle error response without error field', async () => {
      // Given: endpoint returns 400 without error field
      const mockHttpClient = createMockHttpClient((request) => {
        if (request.url.includes('/api/dashboard/audit-log')) {
          return Effect.succeed({
            status: 400,
            json: Effect.succeed({ message: 'Bad request' }), // No 'error' field
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
      const testLayer = AuditLogLive.pipe(Layer.provide(httpLayer))

      const params: AuditLogListParams = {
        limit: 10,
        offset: 0,
      }

      // When: listing entries
      const auditLog = await Effect.runPromise(
        Effect.gen(function* () {
          return yield* AuditLog
        }).pipe(Effect.provide(testLayer)),
      )

      // Then: should fail with default error message
      await expect(
        Effect.runPromise(
          auditLog.list(params).pipe(Effect.provide(testLayer)),
        ),
      ).rejects.toThrow('Request failed.')
    })

    test('should only include defined optional parameters in URL', async () => {
      // Given: params with only required fields
      const params: AuditLogListParams = {
        limit: 10,
        offset: 0,
        // No optional filters
      }

      let capturedUrl = ''

      const mockHttpClient = createMockHttpClient((request) => {
        if (request.url.includes('/api/dashboard/audit-log')) {
          capturedUrl = request.url
          return Effect.succeed({
            status: 200,
            json: Effect.succeed({ entries: [], total: 0 }),
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
      const testLayer = AuditLogLive.pipe(Layer.provide(httpLayer))

      // When: listing with only required params
      const auditLog = await Effect.runPromise(
        Effect.gen(function* () {
          return yield* AuditLog
        }).pipe(Effect.provide(testLayer)),
      )

      await Effect.runPromise(
        auditLog.list(params).pipe(Effect.provide(testLayer)),
      )

      // Then: URL should only contain limit and offset
      expect(capturedUrl).toContain('limit=10')
      expect(capturedUrl).toContain('offset=0')
      expect(capturedUrl).not.toContain('from=')
      expect(capturedUrl).not.toContain('to=')
      expect(capturedUrl).not.toContain('outcome=')
      expect(capturedUrl).not.toContain('event_kind=')
      expect(capturedUrl).not.toContain('action=')
      expect(capturedUrl).not.toContain('reason=')
    })
  })
})
