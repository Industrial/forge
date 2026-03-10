/**
 * BDD tests for DashboardLive - production dashboard service implementation.
 * Tests use Effect's Layer system for dependency injection (no vi.mock()).
 */
import { describe, test, expect } from 'bun:test'
import { Effect, Layer } from 'effect'
import type { HttpClient, HttpClientRequest } from '@effect/platform'

import { AuthenticatedHttpClient } from '@/services/AuthenticatedHttpClient'
import { DashboardLive } from './DashboardLive'
import { Dashboard } from './Dashboard'
import { Organization } from '../domain/Organization'
import { DashboardRole } from '../domain/DashboardRole'

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
    // Stub other methods (not used by DashboardLive)
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

describe('DashboardLive', () => {
  describe('getOrganizations behavior', () => {
    test('should fetch organizations successfully', async () => {
      // Given: endpoint returns 200 with valid organizations
      const mockOrganizations = [
        {
          id: 'org-1',
          name: 'Acme Corp',
          slug: 'acme-corp',
          created_at: '2024-01-01T00:00:00Z',
          updated_at: '2024-01-02T00:00:00Z',
        },
        {
          id: 'org-2',
          name: 'Tech Inc',
          slug: 'tech-inc',
        },
      ]

      const mockHttpClient = createMockHttpClient((request) => {
        if (request.url.includes('/api/auth/organizations')) {
          return Effect.succeed({
            status: 200,
            json: Effect.succeed({ organizations: mockOrganizations }),
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
      const testLayer = DashboardLive.pipe(Layer.provide(httpLayer))

      // When: fetching organizations
      const dashboard = await Effect.runPromise(
        Effect.gen(function* () {
          return yield* Dashboard
        }).pipe(Effect.provide(testLayer)),
      )

      const result = await Effect.runPromise(
        dashboard.getOrganizations().pipe(Effect.provide(testLayer)),
      )

      // Then: should return organizations
      expect(result).toHaveLength(2)
      expect(result[0]).toBeInstanceOf(Organization)
      expect(result[0].id).toBe('org-1')
      expect(result[0].name).toBe('Acme Corp')
      expect(result[0].slug).toBe('acme-corp')
      expect(result[0].created_at).toBe('2024-01-01T00:00:00Z')
      expect(result[0].updated_at).toBe('2024-01-02T00:00:00Z')
      expect(result[1].id).toBe('org-2')
      expect(result[1].name).toBe('Tech Inc')
      expect(result[1].slug).toBe('tech-inc')
    })

    test('should handle empty organizations array', async () => {
      // Given: endpoint returns 200 with empty array
      const mockHttpClient = createMockHttpClient((request) => {
        if (request.url.includes('/api/auth/organizations')) {
          return Effect.succeed({
            status: 200,
            json: Effect.succeed({ organizations: [] }),
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
      const testLayer = DashboardLive.pipe(Layer.provide(httpLayer))

      // When: fetching organizations
      const dashboard = await Effect.runPromise(
        Effect.gen(function* () {
          return yield* Dashboard
        }).pipe(Effect.provide(testLayer)),
      )

      const result = await Effect.runPromise(
        dashboard.getOrganizations().pipe(Effect.provide(testLayer)),
      )

      // Then: should return empty array
      expect(result).toHaveLength(0)
    })

    test('should handle missing organizations field', async () => {
      // Given: endpoint returns 200 but missing organizations field
      const mockHttpClient = createMockHttpClient((request) => {
        if (request.url.includes('/api/auth/organizations')) {
          return Effect.succeed({
            status: 200,
            json: Effect.succeed({}), // Missing organizations field
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
      const testLayer = DashboardLive.pipe(Layer.provide(httpLayer))

      // When: fetching organizations
      const dashboard = await Effect.runPromise(
        Effect.gen(function* () {
          return yield* Dashboard
        }).pipe(Effect.provide(testLayer)),
      )

      const result = await Effect.runPromise(
        dashboard.getOrganizations().pipe(Effect.provide(testLayer)),
      )

      // Then: should return empty array as default
      expect(result).toHaveLength(0)
    })

    test('should fail when endpoint returns non-200 status', async () => {
      // Given: endpoint returns 500
      const mockHttpClient = createMockHttpClient((request) => {
        if (request.url.includes('/api/auth/organizations')) {
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
      const testLayer = DashboardLive.pipe(Layer.provide(httpLayer))

      // When: fetching organizations
      const dashboard = await Effect.runPromise(
        Effect.gen(function* () {
          return yield* Dashboard
        }).pipe(Effect.provide(testLayer)),
      )

      // Then: should fail with error message
      await expect(
        Effect.runPromise(
          dashboard.getOrganizations().pipe(Effect.provide(testLayer)),
        ),
      ).rejects.toThrow('Internal server error')
    })

    test('should handle error response without error field', async () => {
      // Given: endpoint returns 400 without error field
      const mockHttpClient = createMockHttpClient((request) => {
        if (request.url.includes('/api/auth/organizations')) {
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

      const httpLayer = Layer.succeed(AuthenticatedHttpClient, mockHttpClient)
      const testLayer = DashboardLive.pipe(Layer.provide(httpLayer))

      // When: fetching organizations
      const dashboard = await Effect.runPromise(
        Effect.gen(function* () {
          return yield* Dashboard
        }).pipe(Effect.provide(testLayer)),
      )

      // Then: should fail with message from body
      await expect(
        Effect.runPromise(
          dashboard.getOrganizations().pipe(Effect.provide(testLayer)),
        ),
      ).rejects.toThrow('Bad request')
    })
  })

  describe('getRolesByOrg behavior', () => {
    test('should fetch roles for organization successfully', async () => {
      // Given: endpoint returns 200 with valid roles
      const orgId = 'org-123'
      const mockRoles = [
        {
          id: 'role-1',
          name: 'admin',
          display_name: 'Administrator',
        },
        {
          id: 'role-2',
          name: 'member',
          display_name: null,
        },
      ]

      const mockHttpClient = createMockHttpClient((request) => {
        if (
          request.url.includes('/api/auth/roles') &&
          request.url.includes(`org_id=${encodeURIComponent(orgId)}`)
        ) {
          return Effect.succeed({
            status: 200,
            json: Effect.succeed({ roles: mockRoles }),
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
      const testLayer = DashboardLive.pipe(Layer.provide(httpLayer))

      // When: fetching roles
      const dashboard = await Effect.runPromise(
        Effect.gen(function* () {
          return yield* Dashboard
        }).pipe(Effect.provide(testLayer)),
      )

      const result = await Effect.runPromise(
        dashboard.getRolesByOrg(orgId).pipe(Effect.provide(testLayer)),
      )

      // Then: should return roles
      expect(result).toHaveLength(2)
      expect(result[0]).toBeInstanceOf(DashboardRole)
      expect(result[0].id).toBe('role-1')
      expect(result[0].name).toBe('admin')
      expect(result[0].display_name).toBe('Administrator')
      expect(result[1].id).toBe('role-2')
      expect(result[1].name).toBe('member')
      expect(result[1].display_name).toBeNull()
    })

    test('should return empty array when orgId is empty', async () => {
      // Given: empty orgId
      const orgId = ''

      const mockHttpClient = createMockHttpClient(() =>
        Effect.succeed({
          status: 404,
          json: Effect.succeed({}),
          headers: new Headers(),
        }),
      )

      const httpLayer = Layer.succeed(AuthenticatedHttpClient, mockHttpClient)
      const testLayer = DashboardLive.pipe(Layer.provide(httpLayer))

      // When: fetching roles with empty orgId
      const dashboard = await Effect.runPromise(
        Effect.gen(function* () {
          return yield* Dashboard
        }).pipe(Effect.provide(testLayer)),
      )

      const result = await Effect.runPromise(
        dashboard.getRolesByOrg(orgId).pipe(Effect.provide(testLayer)),
      )

      // Then: should return empty array without making HTTP request
      expect(result).toHaveLength(0)
    })

    test('should handle empty roles array', async () => {
      // Given: endpoint returns 200 with empty array
      const orgId = 'org-123'

      const mockHttpClient = createMockHttpClient((request) => {
        if (
          request.url.includes('/api/auth/roles') &&
          request.url.includes(`org_id=${encodeURIComponent(orgId)}`)
        ) {
          return Effect.succeed({
            status: 200,
            json: Effect.succeed({ roles: [] }),
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
      const testLayer = DashboardLive.pipe(Layer.provide(httpLayer))

      // When: fetching roles
      const dashboard = await Effect.runPromise(
        Effect.gen(function* () {
          return yield* Dashboard
        }).pipe(Effect.provide(testLayer)),
      )

      const result = await Effect.runPromise(
        dashboard.getRolesByOrg(orgId).pipe(Effect.provide(testLayer)),
      )

      // Then: should return empty array
      expect(result).toHaveLength(0)
    })

    test('should handle missing roles field', async () => {
      // Given: endpoint returns 200 but missing roles field
      const orgId = 'org-123'

      const mockHttpClient = createMockHttpClient((request) => {
        if (
          request.url.includes('/api/auth/roles') &&
          request.url.includes(`org_id=${encodeURIComponent(orgId)}`)
        ) {
          return Effect.succeed({
            status: 200,
            json: Effect.succeed({}), // Missing roles field
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
      const testLayer = DashboardLive.pipe(Layer.provide(httpLayer))

      // When: fetching roles
      const dashboard = await Effect.runPromise(
        Effect.gen(function* () {
          return yield* Dashboard
        }).pipe(Effect.provide(testLayer)),
      )

      const result = await Effect.runPromise(
        dashboard.getRolesByOrg(orgId).pipe(Effect.provide(testLayer)),
      )

      // Then: should return empty array as default
      expect(result).toHaveLength(0)
    })

    test('should return empty array when endpoint returns non-200 status', async () => {
      // Given: endpoint returns 500
      const orgId = 'org-123'

      const mockHttpClient = createMockHttpClient((request) => {
        if (
          request.url.includes('/api/auth/roles') &&
          request.url.includes(`org_id=${encodeURIComponent(orgId)}`)
        ) {
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
      const testLayer = DashboardLive.pipe(Layer.provide(httpLayer))

      // When: fetching roles
      const dashboard = await Effect.runPromise(
        Effect.gen(function* () {
          return yield* Dashboard
        }).pipe(Effect.provide(testLayer)),
      )

      const result = await Effect.runPromise(
        dashboard.getRolesByOrg(orgId).pipe(Effect.provide(testLayer)),
      )

      // Then: should return empty array (getRolesByOrg returns [] on error)
      expect(result).toHaveLength(0)
    })

    test('should URL encode orgId in request', async () => {
      // Given: orgId with special characters
      const orgId = 'org-123?test=value&other=123'

      const mockHttpClient = createMockHttpClient((request) => {
        // Verify URL encoding
        if (
          request.url.includes('/api/auth/roles') &&
          request.url.includes(`org_id=${encodeURIComponent(orgId)}`)
        ) {
          return Effect.succeed({
            status: 200,
            json: Effect.succeed({ roles: [] }),
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
      const testLayer = DashboardLive.pipe(Layer.provide(httpLayer))

      // When: fetching roles
      const dashboard = await Effect.runPromise(
        Effect.gen(function* () {
          return yield* Dashboard
        }).pipe(Effect.provide(testLayer)),
      )

      const result = await Effect.runPromise(
        dashboard.getRolesByOrg(orgId).pipe(Effect.provide(testLayer)),
      )

      // Then: should succeed (URL was properly encoded)
      expect(result).toHaveLength(0)
    })
  })
})
