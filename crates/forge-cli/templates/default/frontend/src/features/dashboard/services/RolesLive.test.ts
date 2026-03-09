/**
 * BDD tests for RolesLive - production roles service implementation.
 * Tests use Effect's Layer system for dependency injection (no vi.mock()).
 */
import { describe, test, expect } from 'bun:test'
import { Effect, Layer } from 'effect'
import { HttpClient, HttpClientRequest } from '@effect/platform'

import { AuthenticatedHttpClient } from '@/services/AuthenticatedHttpClient'
import { RolesLive } from './RolesLive'
import { Roles } from './Roles'
import { Role } from '../domain/Role'
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
    // Stub other methods (not used by RolesLive)
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

describe('RolesLive', () => {
  describe('list behavior', () => {
    test('should fetch roles successfully', async () => {
      // Given: endpoint returns 200 with valid roles data
      const mockRoles = [
        {
          id: 'role-1',
          org_id: 'org-1',
          name: 'admin',
          display_name: 'Administrator',
          created_at: '2024-01-01T00:00:00Z',
          updated_at: '2024-01-02T00:00:00Z',
        },
        {
          id: 'role-2',
          org_id: 'org-1',
          name: 'member',
          display_name: null,
        },
      ]

      const mockHttpClient = createMockHttpClient((request) => {
        if (request.url === '/api/dashboard/roles') {
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
      const testLayer = RolesLive.pipe(Layer.provide(httpLayer))

      // When: fetching roles
      const roles = await Effect.runPromise(
        Effect.gen(function* () {
          return yield* Roles
        }).pipe(Effect.provide(testLayer)),
      )

      const result = await Effect.runPromise(
        roles.list().pipe(Effect.provide(testLayer)),
      )

      // Then: should return Role instances
      expect(result).toHaveLength(2)
      expect(result[0]).toBeInstanceOf(Role)
      expect(result[0].id).toBe('role-1')
      expect(result[0].name).toBe('admin')
      expect(result[0].display_name).toBe('Administrator')
      expect(result[1].name).toBe('member')
      expect(result[1].display_name).toBeNull()
    })

    test('should handle empty roles array', async () => {
      // Given: endpoint returns 200 with empty roles array
      const mockHttpClient = createMockHttpClient((request) => {
        if (request.url === '/api/dashboard/roles') {
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
      const testLayer = RolesLive.pipe(Layer.provide(httpLayer))

      // When: fetching roles
      const roles = await Effect.runPromise(
        Effect.gen(function* () {
          return yield* Roles
        }).pipe(Effect.provide(testLayer)),
      )

      const result = await Effect.runPromise(
        roles.list().pipe(Effect.provide(testLayer)),
      )

      // Then: should return empty array
      expect(result).toHaveLength(0)
    })

    test('should handle missing roles field', async () => {
      // Given: endpoint returns 200 but missing roles field
      const mockHttpClient = createMockHttpClient((request) => {
        if (request.url === '/api/dashboard/roles') {
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
      const testLayer = RolesLive.pipe(Layer.provide(httpLayer))

      // When: fetching roles
      const roles = await Effect.runPromise(
        Effect.gen(function* () {
          return yield* Roles
        }).pipe(Effect.provide(testLayer)),
      )

      const result = await Effect.runPromise(
        roles.list().pipe(Effect.provide(testLayer)),
      )

      // Then: should return empty array as default
      expect(result).toHaveLength(0)
    })

    test('should fail when endpoint returns 403', async () => {
      // Given: endpoint returns 403
      const mockHttpClient = createMockHttpClient((request) => {
        if (request.url === '/api/dashboard/roles') {
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
      const testLayer = RolesLive.pipe(Layer.provide(httpLayer))

      // When: fetching roles
      const roles = await Effect.runPromise(
        Effect.gen(function* () {
          return yield* Roles
        }).pipe(Effect.provide(testLayer)),
      )

      // Then: should fail with permission error
      await expect(
        Effect.runPromise(roles.list().pipe(Effect.provide(testLayer))),
      ).rejects.toThrow('You do not have permission to view roles.')
    })

    test('should fail when endpoint returns non-200 status', async () => {
      // Given: endpoint returns 500
      const mockHttpClient = createMockHttpClient((request) => {
        if (request.url === '/api/dashboard/roles') {
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
      const testLayer = RolesLive.pipe(Layer.provide(httpLayer))

      // When: fetching roles
      const roles = await Effect.runPromise(
        Effect.gen(function* () {
          return yield* Roles
        }).pipe(Effect.provide(testLayer)),
      )

      // Then: should fail with error message
      await expect(
        Effect.runPromise(roles.list().pipe(Effect.provide(testLayer))),
      ).rejects.toThrow('Internal server error')
    })
  })

  describe('listByOrg behavior', () => {
    test('should fetch roles by organization successfully', async () => {
      // Given: endpoint returns 200 with valid roles data
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
          request.url ===
          `/api/dashboard/roles?org_id=${encodeURIComponent(orgId)}`
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
      const testLayer = RolesLive.pipe(Layer.provide(httpLayer))

      // When: fetching roles by org
      const roles = await Effect.runPromise(
        Effect.gen(function* () {
          return yield* Roles
        }).pipe(Effect.provide(testLayer)),
      )

      const result = await Effect.runPromise(
        roles.listByOrg(orgId).pipe(Effect.provide(testLayer)),
      )

      // Then: should return DashboardRole instances
      expect(result).toHaveLength(2)
      expect(result[0]).toBeInstanceOf(DashboardRole)
      expect(result[0].id).toBe('role-1')
      expect(result[0].name).toBe('admin')
      expect(result[0].display_name).toBe('Administrator')
      expect(result[1].name).toBe('member')
      expect(result[1].display_name).toBeNull()
    })

    test('should return empty array when orgId is empty', async () => {
      // Given: empty orgId
      const mockHttpClient = createMockHttpClient(() => {
        return Effect.succeed({
          status: 404,
          json: Effect.succeed({}),
          headers: new Headers(),
        })
      })

      const httpLayer = Layer.succeed(AuthenticatedHttpClient, mockHttpClient)
      const testLayer = RolesLive.pipe(Layer.provide(httpLayer))

      // When: fetching roles with empty orgId
      const roles = await Effect.runPromise(
        Effect.gen(function* () {
          return yield* Roles
        }).pipe(Effect.provide(testLayer)),
      )

      const result = await Effect.runPromise(
        roles.listByOrg('').pipe(Effect.provide(testLayer)),
      )

      // Then: should return empty array without making request
      expect(result).toHaveLength(0)
    })

    test('should return empty array when endpoint returns error', async () => {
      // Given: endpoint returns 500
      const orgId = 'org-123'
      const mockHttpClient = createMockHttpClient((request) => {
        if (
          request.url ===
          `/api/dashboard/roles?org_id=${encodeURIComponent(orgId)}`
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
      const testLayer = RolesLive.pipe(Layer.provide(httpLayer))

      // When: fetching roles by org
      const roles = await Effect.runPromise(
        Effect.gen(function* () {
          return yield* Roles
        }).pipe(Effect.provide(testLayer)),
      )

      const result = await Effect.runPromise(
        roles.listByOrg(orgId).pipe(Effect.provide(testLayer)),
      )

      // Then: should return empty array (graceful degradation)
      expect(result).toHaveLength(0)
    })

    test('should handle missing roles field', async () => {
      // Given: endpoint returns 200 but missing roles field
      const orgId = 'org-123'
      const mockHttpClient = createMockHttpClient((request) => {
        if (
          request.url ===
          `/api/dashboard/roles?org_id=${encodeURIComponent(orgId)}`
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
      const testLayer = RolesLive.pipe(Layer.provide(httpLayer))

      // When: fetching roles by org
      const roles = await Effect.runPromise(
        Effect.gen(function* () {
          return yield* Roles
        }).pipe(Effect.provide(testLayer)),
      )

      const result = await Effect.runPromise(
        roles.listByOrg(orgId).pipe(Effect.provide(testLayer)),
      )

      // Then: should return empty array as default
      expect(result).toHaveLength(0)
    })
  })

  describe('create behavior', () => {
    test('should create role successfully', async () => {
      // Given: endpoint returns 200
      const createBody = {
        org_id: 'org-123',
        name: 'new-role',
        display_name: 'New Role',
      }

      const mockHttpClient = createMockHttpClient((request) => {
        if (
          request.url === '/api/dashboard/roles' &&
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
      const testLayer = RolesLive.pipe(Layer.provide(httpLayer))

      // When: creating role
      const roles = await Effect.runPromise(
        Effect.gen(function* () {
          return yield* Roles
        }).pipe(Effect.provide(testLayer)),
      )

      // Then: should succeed without error
      await expect(
        Effect.runPromise(
          roles.create(createBody).pipe(Effect.provide(testLayer)),
        ),
      ).resolves.toBeUndefined()
    })

    test('should fail when endpoint returns 403', async () => {
      // Given: endpoint returns 403
      const createBody = {
        org_id: 'org-123',
        name: 'new-role',
      }

      const mockHttpClient = createMockHttpClient((request) => {
        if (
          request.url === '/api/dashboard/roles' &&
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
      const testLayer = RolesLive.pipe(Layer.provide(httpLayer))

      // When: creating role
      const roles = await Effect.runPromise(
        Effect.gen(function* () {
          return yield* Roles
        }).pipe(Effect.provide(testLayer)),
      )

      // Then: should fail with permission error
      await expect(
        Effect.runPromise(
          roles.create(createBody).pipe(Effect.provide(testLayer)),
        ),
      ).rejects.toThrow('You do not have permission to create roles.')
    })

    test('should fail when endpoint returns non-200 status', async () => {
      // Given: endpoint returns 400
      const createBody = {
        org_id: 'org-123',
        name: 'new-role',
      }

      const mockHttpClient = createMockHttpClient((request) => {
        if (
          request.url === '/api/dashboard/roles' &&
          request.method === 'POST'
        ) {
          return Effect.succeed({
            status: 400,
            json: Effect.succeed({ error: 'Invalid role name' }),
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
      const testLayer = RolesLive.pipe(Layer.provide(httpLayer))

      // When: creating role
      const roles = await Effect.runPromise(
        Effect.gen(function* () {
          return yield* Roles
        }).pipe(Effect.provide(testLayer)),
      )

      // Then: should fail with error message
      await expect(
        Effect.runPromise(
          roles.create(createBody).pipe(Effect.provide(testLayer)),
        ),
      ).rejects.toThrow('Invalid role name')
    })
  })

  describe('update behavior', () => {
    test('should update role successfully', async () => {
      // Given: endpoint returns 200
      const updateBody = {
        id: 'role-1',
        name: 'updated-role',
        display_name: 'Updated Role',
      }

      const mockHttpClient = createMockHttpClient((request) => {
        if (
          request.url === '/api/dashboard/roles' &&
          request.method === 'PATCH'
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
      const testLayer = RolesLive.pipe(Layer.provide(httpLayer))

      // When: updating role
      const roles = await Effect.runPromise(
        Effect.gen(function* () {
          return yield* Roles
        }).pipe(Effect.provide(testLayer)),
      )

      // Then: should succeed without error
      await expect(
        Effect.runPromise(
          roles.update(updateBody).pipe(Effect.provide(testLayer)),
        ),
      ).resolves.toBeUndefined()
    })

    test('should fail when endpoint returns 403', async () => {
      // Given: endpoint returns 403
      const updateBody = {
        id: 'role-1',
        name: 'updated-role',
      }

      const mockHttpClient = createMockHttpClient((request) => {
        if (
          request.url === '/api/dashboard/roles' &&
          request.method === 'PATCH'
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
      const testLayer = RolesLive.pipe(Layer.provide(httpLayer))

      // When: updating role
      const roles = await Effect.runPromise(
        Effect.gen(function* () {
          return yield* Roles
        }).pipe(Effect.provide(testLayer)),
      )

      // Then: should fail with permission error
      await expect(
        Effect.runPromise(
          roles.update(updateBody).pipe(Effect.provide(testLayer)),
        ),
      ).rejects.toThrow('You do not have permission to update roles.')
    })

    test('should fail when endpoint returns non-200 status', async () => {
      // Given: endpoint returns 404
      const updateBody = {
        id: 'role-1',
        name: 'updated-role',
      }

      const mockHttpClient = createMockHttpClient((request) => {
        if (
          request.url === '/api/dashboard/roles' &&
          request.method === 'PATCH'
        ) {
          return Effect.succeed({
            status: 404,
            json: Effect.succeed({ error: 'Role not found' }),
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
      const testLayer = RolesLive.pipe(Layer.provide(httpLayer))

      // When: updating role
      const roles = await Effect.runPromise(
        Effect.gen(function* () {
          return yield* Roles
        }).pipe(Effect.provide(testLayer)),
      )

      // Then: should fail with error message
      await expect(
        Effect.runPromise(
          roles.update(updateBody).pipe(Effect.provide(testLayer)),
        ),
      ).rejects.toThrow('Role not found')
    })
  })

  describe('delete behavior', () => {
    test('should delete role successfully', async () => {
      // Given: endpoint returns 200
      const roleId = 'role-1'

      const mockHttpClient = createMockHttpClient((request) => {
        if (
          request.url === '/api/dashboard/roles' &&
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
      const testLayer = RolesLive.pipe(Layer.provide(httpLayer))

      // When: deleting role
      const roles = await Effect.runPromise(
        Effect.gen(function* () {
          return yield* Roles
        }).pipe(Effect.provide(testLayer)),
      )

      // Then: should succeed without error
      await expect(
        Effect.runPromise(roles.delete(roleId).pipe(Effect.provide(testLayer))),
      ).resolves.toBeUndefined()
    })

    test('should fail when endpoint returns 403', async () => {
      // Given: endpoint returns 403
      const roleId = 'role-1'

      const mockHttpClient = createMockHttpClient((request) => {
        if (
          request.url === '/api/dashboard/roles' &&
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
      const testLayer = RolesLive.pipe(Layer.provide(httpLayer))

      // When: deleting role
      const roles = await Effect.runPromise(
        Effect.gen(function* () {
          return yield* Roles
        }).pipe(Effect.provide(testLayer)),
      )

      // Then: should fail with permission error
      await expect(
        Effect.runPromise(roles.delete(roleId).pipe(Effect.provide(testLayer))),
      ).rejects.toThrow('You do not have permission to delete roles.')
    })

    test('should fail when endpoint returns non-200 status', async () => {
      // Given: endpoint returns 404
      const roleId = 'role-1'

      const mockHttpClient = createMockHttpClient((request) => {
        if (
          request.url === '/api/dashboard/roles' &&
          request.method === 'DELETE'
        ) {
          return Effect.succeed({
            status: 404,
            json: Effect.succeed({ error: 'Role not found' }),
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
      const testLayer = RolesLive.pipe(Layer.provide(httpLayer))

      // When: deleting role
      const roles = await Effect.runPromise(
        Effect.gen(function* () {
          return yield* Roles
        }).pipe(Effect.provide(testLayer)),
      )

      // Then: should fail with error message
      await expect(
        Effect.runPromise(roles.delete(roleId).pipe(Effect.provide(testLayer))),
      ).rejects.toThrow('Role not found')
    })
  })
})
