/**
 * BDD tests for PermissionsLive - production permissions service implementation.
 * Tests use Effect's Layer system for dependency injection (no vi.mock()).
 */
import { describe, test, expect } from 'bun:test'
import { Effect, Layer } from 'effect'
import { HttpClient, HttpClientRequest } from '@effect/platform'

import { AuthenticatedHttpClient } from '@/services/AuthenticatedHttpClient'
import { PermissionsLive } from './PermissionsLive'
import { Permissions } from './Permissions'
import { Assignment } from '../domain/Assignment'
import type { PermissionsData } from './Permissions'

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
    // Stub other methods (not used by PermissionsLive)
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

describe('PermissionsLive', () => {
  describe('getData behavior', () => {
    test('should fetch assignments and permissions successfully', async () => {
      // Given: both endpoints return 200 with valid data
      const mockAssignments = [
        {
          scope: 'global',
          role_name: 'admin',
          permission_key: 'read:users',
          org_id: null,
        },
        {
          scope: 'org',
          role_name: 'member',
          permission_key: 'write:posts',
          org_id: 'org-123',
        },
      ]

      const mockPermissions = [
        'read:users',
        'write:users',
        'read:posts',
        'write:posts',
      ]

      const mockHttpClient = createMockHttpClient((request) => {
        if (request.url.includes('/api/dashboard/role-permissions')) {
          return Effect.succeed({
            status: 200,
            json: Effect.succeed({ assignments: mockAssignments }),
            headers: new Headers(),
          })
        }
        if (request.url.includes('/api/dashboard/permissions')) {
          return Effect.succeed({
            status: 200,
            json: Effect.succeed({ permissions: mockPermissions }),
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
      const testLayer = PermissionsLive.pipe(Layer.provide(httpLayer))

      // When: fetching data
      const permissions = await Effect.runPromise(
        Effect.gen(function* () {
          return yield* Permissions
        }).pipe(Effect.provide(testLayer)),
      )

      const result = await Effect.runPromise(
        permissions.getData().pipe(Effect.provide(testLayer)),
      )

      // Then: should return assignments and permissions
      expect(result.assignments).toHaveLength(2)
      expect(result.assignments[0]).toBeInstanceOf(Assignment)
      expect(result.assignments[0].scope).toBe('global')
      expect(result.assignments[0].role_name).toBe('admin')
      expect(result.assignments[0].permission_key).toBe('read:users')
      expect(result.assignments[1].org_id).toBe('org-123')
      expect(result.permissions).toEqual(mockPermissions)
    })

    test('should handle empty arrays', async () => {
      // Given: both endpoints return 200 with empty arrays
      const mockHttpClient = createMockHttpClient((request) => {
        if (request.url.includes('/api/dashboard/role-permissions')) {
          return Effect.succeed({
            status: 200,
            json: Effect.succeed({ assignments: [] }),
            headers: new Headers(),
          })
        }
        if (request.url.includes('/api/dashboard/permissions')) {
          return Effect.succeed({
            status: 200,
            json: Effect.succeed({ permissions: [] }),
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
      const testLayer = PermissionsLive.pipe(Layer.provide(httpLayer))

      // When: fetching data
      const permissions = await Effect.runPromise(
        Effect.gen(function* () {
          return yield* Permissions
        }).pipe(Effect.provide(testLayer)),
      )

      const result = await Effect.runPromise(
        permissions.getData().pipe(Effect.provide(testLayer)),
      )

      // Then: should return empty arrays
      expect(result.assignments).toHaveLength(0)
      expect(result.permissions).toHaveLength(0)
    })

    test('should handle missing assignments or permissions fields', async () => {
      // Given: endpoints return 200 but missing fields
      const mockHttpClient = createMockHttpClient((request) => {
        if (request.url.includes('/api/dashboard/role-permissions')) {
          return Effect.succeed({
            status: 200,
            json: Effect.succeed({}), // Missing assignments field
            headers: new Headers(),
          })
        }
        if (request.url.includes('/api/dashboard/permissions')) {
          return Effect.succeed({
            status: 200,
            json: Effect.succeed({}), // Missing permissions field
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
      const testLayer = PermissionsLive.pipe(Layer.provide(httpLayer))

      // When: fetching data
      const permissions = await Effect.runPromise(
        Effect.gen(function* () {
          return yield* Permissions
        }).pipe(Effect.provide(testLayer)),
      )

      const result = await Effect.runPromise(
        permissions.getData().pipe(Effect.provide(testLayer)),
      )

      // Then: should return empty arrays as defaults
      expect(result.assignments).toHaveLength(0)
      expect(result.permissions).toHaveLength(0)
    })

    test('should fail when assignments endpoint returns 403', async () => {
      // Given: assignments endpoint returns 403
      const mockHttpClient = createMockHttpClient((request) => {
        if (request.url.includes('/api/dashboard/role-permissions')) {
          return Effect.succeed({
            status: 403,
            json: Effect.succeed({}),
            headers: new Headers(),
          })
        }
        if (request.url.includes('/api/dashboard/permissions')) {
          return Effect.succeed({
            status: 200,
            json: Effect.succeed({ permissions: [] }),
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
      const testLayer = PermissionsLive.pipe(Layer.provide(httpLayer))

      // When: fetching data
      const permissions = await Effect.runPromise(
        Effect.gen(function* () {
          return yield* Permissions
        }).pipe(Effect.provide(testLayer)),
      )

      // Then: should fail with permission error
      await expect(
        Effect.runPromise(
          permissions.getData().pipe(Effect.provide(testLayer)),
        ),
      ).rejects.toThrow('You do not have permission to manage permissions.')
    })

    test('should fail when permissions endpoint returns 403', async () => {
      // Given: permissions endpoint returns 403
      const mockHttpClient = createMockHttpClient((request) => {
        if (request.url.includes('/api/dashboard/role-permissions')) {
          return Effect.succeed({
            status: 200,
            json: Effect.succeed({ assignments: [] }),
            headers: new Headers(),
          })
        }
        if (request.url.includes('/api/dashboard/permissions')) {
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

      const httpLayer = Layer.succeed(AuthenticatedHttpClient, mockHttpClient)
      const testLayer = PermissionsLive.pipe(Layer.provide(httpLayer))

      // When: fetching data
      const permissions = await Effect.runPromise(
        Effect.gen(function* () {
          return yield* Permissions
        }).pipe(Effect.provide(testLayer)),
      )

      // Then: should fail with permission error
      await expect(
        Effect.runPromise(
          permissions.getData().pipe(Effect.provide(testLayer)),
        ),
      ).rejects.toThrow('You do not have permission to manage permissions.')
    })

    test('should fail when assignments endpoint returns non-200 status', async () => {
      // Given: assignments endpoint returns 500
      const mockHttpClient = createMockHttpClient((request) => {
        if (request.url.includes('/api/dashboard/role-permissions')) {
          return Effect.succeed({
            status: 500,
            json: Effect.succeed({ error: 'Internal server error' }),
            headers: new Headers(),
          })
        }
        if (request.url.includes('/api/dashboard/permissions')) {
          return Effect.succeed({
            status: 200,
            json: Effect.succeed({ permissions: [] }),
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
      const testLayer = PermissionsLive.pipe(Layer.provide(httpLayer))

      // When: fetching data
      const permissions = await Effect.runPromise(
        Effect.gen(function* () {
          return yield* Permissions
        }).pipe(Effect.provide(testLayer)),
      )

      // Then: should fail with error
      await expect(
        Effect.runPromise(
          permissions.getData().pipe(Effect.provide(testLayer)),
        ),
      ).rejects.toThrow('Failed to load data.')
    })

    test('should fail when permissions endpoint returns non-200 status', async () => {
      // Given: permissions endpoint returns 500
      const mockHttpClient = createMockHttpClient((request) => {
        if (request.url.includes('/api/dashboard/role-permissions')) {
          return Effect.succeed({
            status: 200,
            json: Effect.succeed({ assignments: [] }),
            headers: new Headers(),
          })
        }
        if (request.url.includes('/api/dashboard/permissions')) {
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

      const httpLayer = Layer.succeed(AuthenticatedHttpClient, mockHttpClient)
      const testLayer = PermissionsLive.pipe(Layer.provide(httpLayer))

      // When: fetching data
      const permissions = await Effect.runPromise(
        Effect.gen(function* () {
          return yield* Permissions
        }).pipe(Effect.provide(testLayer)),
      )

      // Then: should fail with error
      await expect(
        Effect.runPromise(
          permissions.getData().pipe(Effect.provide(testLayer)),
        ),
      ).rejects.toThrow('Failed to load data.')
    })
  })

  describe('add behavior', () => {
    test('should add assignment successfully', async () => {
      // Given: POST endpoint returns 200
      const body = {
        scope: 'global',
        role_name: 'admin',
        permission_key: 'read:users',
      }

      const mockHttpClient = createMockHttpClient((request) => {
        if (
          request.url.includes('/api/dashboard/role-permissions') &&
          request.method === 'POST'
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
      const testLayer = PermissionsLive.pipe(Layer.provide(httpLayer))

      // When: adding assignment
      const permissions = await Effect.runPromise(
        Effect.gen(function* () {
          return yield* Permissions
        }).pipe(Effect.provide(testLayer)),
      )

      const result = await Effect.runPromise(
        permissions.add(body).pipe(Effect.provide(testLayer)),
      )

      // Then: should succeed (returns void)
      expect(result).toBeUndefined()
    })

    test('should fail when endpoint returns 403', async () => {
      // Given: POST endpoint returns 403
      const body = {
        scope: 'global',
        role_name: 'admin',
        permission_key: 'read:users',
      }

      const mockHttpClient = createMockHttpClient((request) => {
        if (
          request.url.includes('/api/dashboard/role-permissions') &&
          request.method === 'POST'
        ) {
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

      const httpLayer = Layer.succeed(AuthenticatedHttpClient, mockHttpClient)
      const testLayer = PermissionsLive.pipe(Layer.provide(httpLayer))

      // When: adding assignment
      const permissions = await Effect.runPromise(
        Effect.gen(function* () {
          return yield* Permissions
        }).pipe(Effect.provide(testLayer)),
      )

      // Then: should fail with permission error
      await expect(
        Effect.runPromise(
          permissions.add(body).pipe(Effect.provide(testLayer)),
        ),
      ).rejects.toThrow('You do not have permission to manage permissions.')
    })

    test('should fail when endpoint returns non-200 status', async () => {
      // Given: POST endpoint returns 400
      const body = {
        scope: 'global',
        role_name: 'admin',
        permission_key: 'read:users',
      }

      const mockHttpClient = createMockHttpClient((request) => {
        if (
          request.url.includes('/api/dashboard/role-permissions') &&
          request.method === 'POST'
        ) {
          return Effect.succeed({
            status: 400,
            json: Effect.succeed({ error: 'Invalid request' }),
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
      const testLayer = PermissionsLive.pipe(Layer.provide(httpLayer))

      // When: adding assignment
      const permissions = await Effect.runPromise(
        Effect.gen(function* () {
          return yield* Permissions
        }).pipe(Effect.provide(testLayer)),
      )

      // Then: should fail with error message
      await expect(
        Effect.runPromise(
          permissions.add(body).pipe(Effect.provide(testLayer)),
        ),
      ).rejects.toThrow('Invalid request')
    })

    test('should handle error response without error field', async () => {
      // Given: POST endpoint returns 500 without error field
      const body = {
        scope: 'global',
        role_name: 'admin',
        permission_key: 'read:users',
      }

      const mockHttpClient = createMockHttpClient((request) => {
        if (
          request.url.includes('/api/dashboard/role-permissions') &&
          request.method === 'POST'
        ) {
          return Effect.succeed({
            status: 500,
            json: Effect.succeed({ message: 'Server error' }), // No 'error' field
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
      const testLayer = PermissionsLive.pipe(Layer.provide(httpLayer))

      // When: adding assignment
      const permissions = await Effect.runPromise(
        Effect.gen(function* () {
          return yield* Permissions
        }).pipe(Effect.provide(testLayer)),
      )

      // Then: should fail with message from body
      await expect(
        Effect.runPromise(
          permissions.add(body).pipe(Effect.provide(testLayer)),
        ),
      ).rejects.toThrow('Server error')
    })
  })

  describe('delete behavior', () => {
    test('should delete assignment successfully', async () => {
      // Given: DELETE endpoint returns 200
      const body = {
        scope: 'global',
        role_name: 'admin',
        permission_key: 'read:users',
        org_id: null,
      }

      const mockHttpClient = createMockHttpClient((request) => {
        if (
          request.url.includes('/api/dashboard/role-permissions') &&
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
      const testLayer = PermissionsLive.pipe(Layer.provide(httpLayer))

      // When: deleting assignment
      const permissions = await Effect.runPromise(
        Effect.gen(function* () {
          return yield* Permissions
        }).pipe(Effect.provide(testLayer)),
      )

      const result = await Effect.runPromise(
        permissions.delete(body).pipe(Effect.provide(testLayer)),
      )

      // Then: should succeed (returns void)
      expect(result).toBeUndefined()
    })

    test('should delete assignment with org_id successfully', async () => {
      // Given: DELETE endpoint returns 200 for org-scoped assignment
      const body = {
        scope: 'org',
        role_name: 'member',
        permission_key: 'write:posts',
        org_id: 'org-123',
      }

      const mockHttpClient = createMockHttpClient((request) => {
        if (
          request.url.includes('/api/dashboard/role-permissions') &&
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
      const testLayer = PermissionsLive.pipe(Layer.provide(httpLayer))

      // When: deleting assignment
      const permissions = await Effect.runPromise(
        Effect.gen(function* () {
          return yield* Permissions
        }).pipe(Effect.provide(testLayer)),
      )

      const result = await Effect.runPromise(
        permissions.delete(body).pipe(Effect.provide(testLayer)),
      )

      // Then: should succeed (returns void)
      expect(result).toBeUndefined()
    })

    test('should fail when endpoint returns 403', async () => {
      // Given: DELETE endpoint returns 403
      const body = {
        scope: 'global',
        role_name: 'admin',
        permission_key: 'read:users',
        org_id: null,
      }

      const mockHttpClient = createMockHttpClient((request) => {
        if (
          request.url.includes('/api/dashboard/role-permissions') &&
          request.method === 'DELETE'
        ) {
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

      const httpLayer = Layer.succeed(AuthenticatedHttpClient, mockHttpClient)
      const testLayer = PermissionsLive.pipe(Layer.provide(httpLayer))

      // When: deleting assignment
      const permissions = await Effect.runPromise(
        Effect.gen(function* () {
          return yield* Permissions
        }).pipe(Effect.provide(testLayer)),
      )

      // Then: should fail with permission error
      await expect(
        Effect.runPromise(
          permissions.delete(body).pipe(Effect.provide(testLayer)),
        ),
      ).rejects.toThrow('You do not have permission to manage permissions.')
    })

    test('should fail when endpoint returns non-200 status', async () => {
      // Given: DELETE endpoint returns 404
      const body = {
        scope: 'global',
        role_name: 'admin',
        permission_key: 'read:users',
        org_id: null,
      }

      const mockHttpClient = createMockHttpClient((request) => {
        if (
          request.url.includes('/api/dashboard/role-permissions') &&
          request.method === 'DELETE'
        ) {
          return Effect.succeed({
            status: 404,
            json: Effect.succeed({ error: 'Assignment not found' }),
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
      const testLayer = PermissionsLive.pipe(Layer.provide(httpLayer))

      // When: deleting assignment
      const permissions = await Effect.runPromise(
        Effect.gen(function* () {
          return yield* Permissions
        }).pipe(Effect.provide(testLayer)),
      )

      // Then: should fail with error message
      await expect(
        Effect.runPromise(
          permissions.delete(body).pipe(Effect.provide(testLayer)),
        ),
      ).rejects.toThrow('Assignment not found')
    })

    test('should handle error response without error field', async () => {
      // Given: DELETE endpoint returns 500 without error field
      const body = {
        scope: 'global',
        role_name: 'admin',
        permission_key: 'read:users',
        org_id: null,
      }

      const mockHttpClient = createMockHttpClient((request) => {
        if (
          request.url.includes('/api/dashboard/role-permissions') &&
          request.method === 'DELETE'
        ) {
          return Effect.succeed({
            status: 500,
            json: Effect.succeed({ message: 'Server error' }), // No 'error' field
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
      const testLayer = PermissionsLive.pipe(Layer.provide(httpLayer))

      // When: deleting assignment
      const permissions = await Effect.runPromise(
        Effect.gen(function* () {
          return yield* Permissions
        }).pipe(Effect.provide(testLayer)),
      )

      // Then: should fail with message from body
      await expect(
        Effect.runPromise(
          permissions.delete(body).pipe(Effect.provide(testLayer)),
        ),
      ).rejects.toThrow('Server error')
    })
  })
})
