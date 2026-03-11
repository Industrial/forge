/**
 * BDD tests for UsersLive - production users service implementation.
 * Tests use Effect's Layer system for dependency injection (no vi.mock()).
 */
import { describe, test, expect } from 'bun:test'
import { Effect, Layer } from 'effect'
import type { HttpClient, HttpClientRequest } from '@effect/platform'

import { AuthenticatedHttpClient } from '../../../services/AuthenticatedHttpClient'
import { UsersLive } from './UsersLive'
import { Users } from './Users'
import { User, UserMembership } from '../domain/User'

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
    // Stub other methods (not used by UsersLive)
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

describe('UsersLive', () => {
  describe('list behavior', () => {
    test('should fetch users successfully', async () => {
      // Given: endpoint returns 200 with valid users
      const mockUsers = [
        {
          id: 'user-1',
          email: 'user1@example.com',
          is_active: true,
          is_admin: false,
          created_at: '2024-01-01T00:00:00Z',
          memberships: [
            {
              org_id: 'org-1',
              org_name: 'Acme Corp',
              roles: ['admin', 'member'],
            },
          ],
        },
        {
          id: 'user-2',
          email: 'user2@example.com',
          is_active: false,
          is_admin: true,
          created_at: '2024-01-02T00:00:00Z',
        },
      ]

      const mockHttpClient = createMockHttpClient((request) => {
        if (
          request.url.includes('/api/auth/users') &&
          request.method === 'GET'
        ) {
          return Effect.succeed({
            status: 200,
            json: Effect.succeed({ users: mockUsers }),
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
      const testLayer = UsersLive.pipe(Layer.provide(httpLayer))

      // When: fetching users
      const users = await Effect.runPromise(
        Effect.gen(function* () {
          return yield* Users
        }).pipe(Effect.provide(testLayer)),
      )

      const result = await Effect.runPromise(
        users.list().pipe(Effect.provide(testLayer)),
      )

      // Then: should return users
      expect(result).toHaveLength(2)
      expect(result[0]).toBeInstanceOf(User)
      expect(result[0].id).toBe('user-1')
      expect(result[0].email).toBe('user1@example.com')
      expect(result[0].is_active).toBe(true)
      expect(result[0].is_admin).toBe(false)
      expect(result[0].created_at).toBe('2024-01-01T00:00:00Z')
      expect(result[0].memberships).toHaveLength(1)
      expect(result[0].memberships[0]).toBeInstanceOf(UserMembership)
      expect(result[0].memberships[0].org_id).toBe('org-1')
      expect(result[0].memberships[0].org_name).toBe('Acme Corp')
      expect(result[0].memberships[0].roles).toEqual(['admin', 'member'])

      expect(result[1].id).toBe('user-2')
      expect(result[1].email).toBe('user2@example.com')
      expect(result[1].is_active).toBe(false)
      expect(result[1].is_admin).toBe(true)
      expect(result[1].memberships).toHaveLength(0)
    })

    test('should handle empty users array', async () => {
      // Given: endpoint returns 200 with empty array
      const mockHttpClient = createMockHttpClient((request) => {
        if (
          request.url.includes('/api/auth/users') &&
          request.method === 'GET'
        ) {
          return Effect.succeed({
            status: 200,
            json: Effect.succeed({ users: [] }),
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
      const testLayer = UsersLive.pipe(Layer.provide(httpLayer))

      // When: fetching users
      const users = await Effect.runPromise(
        Effect.gen(function* () {
          return yield* Users
        }).pipe(Effect.provide(testLayer)),
      )

      const result = await Effect.runPromise(
        users.list().pipe(Effect.provide(testLayer)),
      )

      // Then: should return empty array
      expect(result).toHaveLength(0)
    })

    test('should handle missing users field', async () => {
      // Given: endpoint returns 200 but missing users field
      const mockHttpClient = createMockHttpClient((request) => {
        if (
          request.url.includes('/api/auth/users') &&
          request.method === 'GET'
        ) {
          return Effect.succeed({
            status: 200,
            json: Effect.succeed({}), // Missing users field
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
      const testLayer = UsersLive.pipe(Layer.provide(httpLayer))

      // When: fetching users
      const users = await Effect.runPromise(
        Effect.gen(function* () {
          return yield* Users
        }).pipe(Effect.provide(testLayer)),
      )

      const result = await Effect.runPromise(
        users.list().pipe(Effect.provide(testLayer)),
      )

      // Then: should return empty array as default
      expect(result).toHaveLength(0)
    })

    test('should fail with 401 error message', async () => {
      // Given: endpoint returns 401
      const mockHttpClient = createMockHttpClient((request) => {
        if (
          request.url.includes('/api/auth/users') &&
          request.method === 'GET'
        ) {
          return Effect.succeed({
            status: 401,
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
      const testLayer = UsersLive.pipe(Layer.provide(httpLayer))

      // When: fetching users
      const users = await Effect.runPromise(
        Effect.gen(function* () {
          return yield* Users
        }).pipe(Effect.provide(testLayer)),
      )

      // Then: should fail with session expired message
      await expect(
        Effect.runPromise(users.list().pipe(Effect.provide(testLayer))),
      ).rejects.toThrow(
        'Session expired or not logged in. Please log in again.',
      )
    })

    test('should fail with 403 permission error', async () => {
      // Given: endpoint returns 403
      const mockHttpClient = createMockHttpClient((request) => {
        if (
          request.url.includes('/api/auth/users') &&
          request.method === 'GET'
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
      const testLayer = UsersLive.pipe(Layer.provide(httpLayer))

      // When: fetching users
      const users = await Effect.runPromise(
        Effect.gen(function* () {
          return yield* Users
        }).pipe(Effect.provide(testLayer)),
      )

      // Then: should fail with permission error
      await expect(
        Effect.runPromise(users.list().pipe(Effect.provide(testLayer))),
      ).rejects.toThrow('You do not have permission to view users.')
    })

    test('should fail with error message from response body', async () => {
      // Given: endpoint returns 500 with error field
      const mockHttpClient = createMockHttpClient((request) => {
        if (
          request.url.includes('/api/auth/users') &&
          request.method === 'GET'
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
      const testLayer = UsersLive.pipe(Layer.provide(httpLayer))

      // When: fetching users
      const users = await Effect.runPromise(
        Effect.gen(function* () {
          return yield* Users
        }).pipe(Effect.provide(testLayer)),
      )

      // Then: should fail with error message
      await expect(
        Effect.runPromise(users.list().pipe(Effect.provide(testLayer))),
      ).rejects.toThrow('Internal server error')
    })

    test('should fail with HTTP status when no error field', async () => {
      // Given: endpoint returns 500 without error field
      const mockHttpClient = createMockHttpClient((request) => {
        if (
          request.url.includes('/api/auth/users') &&
          request.method === 'GET'
        ) {
          return Effect.succeed({
            status: 500,
            json: Effect.succeed({ message: 'Something went wrong' }),
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
      const testLayer = UsersLive.pipe(Layer.provide(httpLayer))

      // When: fetching users
      const users = await Effect.runPromise(
        Effect.gen(function* () {
          return yield* Users
        }).pipe(Effect.provide(testLayer)),
      )

      // Then: should fail with HTTP status message
      await expect(
        Effect.runPromise(users.list().pipe(Effect.provide(testLayer))),
      ).rejects.toThrow('HTTP 500')
    })
  })

  describe('create behavior', () => {
    test('should create user successfully', async () => {
      // Given: endpoint returns 200
      const createBody = {
        email: 'newuser@example.com',
        password: 'password123',
        org_id: 'org-123',
        role_ids: ['role-1', 'role-2'] as readonly string[],
      }

      const mockHttpClient = createMockHttpClient((request) => {
        if (
          request.url.includes('/api/auth/users') &&
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
      const testLayer = UsersLive.pipe(Layer.provide(httpLayer))

      // When: creating user
      const users = await Effect.runPromise(
        Effect.gen(function* () {
          return yield* Users
        }).pipe(Effect.provide(testLayer)),
      )

      const result = await Effect.runPromise(
        users.create(createBody).pipe(Effect.provide(testLayer)),
      )

      // Then: should succeed (void return)
      expect(result).toBeUndefined()
    })

    test('should fail with 403 permission error', async () => {
      // Given: endpoint returns 403
      const createBody = {
        email: 'newuser@example.com',
        password: 'password123',
        org_id: 'org-123',
        role_ids: ['role-1'] as readonly string[],
      }

      const mockHttpClient = createMockHttpClient((request) => {
        if (
          request.url.includes('/api/auth/users') &&
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
      const testLayer = UsersLive.pipe(Layer.provide(httpLayer))

      // When: creating user
      const users = await Effect.runPromise(
        Effect.gen(function* () {
          return yield* Users
        }).pipe(Effect.provide(testLayer)),
      )

      // Then: should fail with permission error
      await expect(
        Effect.runPromise(
          users.create(createBody).pipe(Effect.provide(testLayer)),
        ),
      ).rejects.toThrow('You do not have permission to create users.')
    })

    test('should fail with error message from response', async () => {
      // Given: endpoint returns 400 with error
      const createBody = {
        email: 'invalid-email',
        password: 'short',
        org_id: 'org-123',
        role_ids: [] as readonly string[],
      }

      const mockHttpClient = createMockHttpClient((request) => {
        if (
          request.url.includes('/api/auth/users') &&
          request.method === 'POST'
        ) {
          return Effect.succeed({
            status: 400,
            json: Effect.succeed({ error: 'Validation failed' }),
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
      const testLayer = UsersLive.pipe(Layer.provide(httpLayer))

      // When: creating user
      const users = await Effect.runPromise(
        Effect.gen(function* () {
          return yield* Users
        }).pipe(Effect.provide(testLayer)),
      )

      // Then: should fail with error message
      await expect(
        Effect.runPromise(
          users.create(createBody).pipe(Effect.provide(testLayer)),
        ),
      ).rejects.toThrow('Validation failed')
    })
  })

  describe('update behavior', () => {
    test('should update user successfully', async () => {
      // Given: endpoint returns 200
      const updateBody = {
        id: 'user-123',
        email: 'updated@example.com',
        is_active: false,
      }

      const mockHttpClient = createMockHttpClient((request) => {
        if (
          request.url.includes('/api/auth/users') &&
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
      const testLayer = UsersLive.pipe(Layer.provide(httpLayer))

      // When: updating user
      const users = await Effect.runPromise(
        Effect.gen(function* () {
          return yield* Users
        }).pipe(Effect.provide(testLayer)),
      )

      const result = await Effect.runPromise(
        users.update(updateBody).pipe(Effect.provide(testLayer)),
      )

      // Then: should succeed (void return)
      expect(result).toBeUndefined()
    })

    test('should update user with only email', async () => {
      // Given: endpoint returns 200 with only email update
      const updateBody = {
        id: 'user-123',
        email: 'newemail@example.com',
      }

      const mockHttpClient = createMockHttpClient((request) => {
        if (
          request.url.includes('/api/auth/users') &&
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
      const testLayer = UsersLive.pipe(Layer.provide(httpLayer))

      // When: updating user
      const users = await Effect.runPromise(
        Effect.gen(function* () {
          return yield* Users
        }).pipe(Effect.provide(testLayer)),
      )

      const result = await Effect.runPromise(
        users.update(updateBody).pipe(Effect.provide(testLayer)),
      )

      // Then: should succeed
      expect(result).toBeUndefined()
    })

    test('should fail with 403 permission error', async () => {
      // Given: endpoint returns 403
      const updateBody = {
        id: 'user-123',
        email: 'updated@example.com',
      }

      const mockHttpClient = createMockHttpClient((request) => {
        if (
          request.url.includes('/api/auth/users') &&
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
      const testLayer = UsersLive.pipe(Layer.provide(httpLayer))

      // When: updating user
      const users = await Effect.runPromise(
        Effect.gen(function* () {
          return yield* Users
        }).pipe(Effect.provide(testLayer)),
      )

      // Then: should fail with permission error
      await expect(
        Effect.runPromise(
          users.update(updateBody).pipe(Effect.provide(testLayer)),
        ),
      ).rejects.toThrow('You do not have permission to update users.')
    })

    test('should fail with error message from response', async () => {
      // Given: endpoint returns 400 with error
      const updateBody = {
        id: 'user-123',
        email: 'invalid-email',
      }

      const mockHttpClient = createMockHttpClient((request) => {
        if (
          request.url.includes('/api/auth/users') &&
          request.method === 'PATCH'
        ) {
          return Effect.succeed({
            status: 400,
            json: Effect.succeed({ error: 'Invalid email format' }),
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
      const testLayer = UsersLive.pipe(Layer.provide(httpLayer))

      // When: updating user
      const users = await Effect.runPromise(
        Effect.gen(function* () {
          return yield* Users
        }).pipe(Effect.provide(testLayer)),
      )

      // Then: should fail with error message
      await expect(
        Effect.runPromise(
          users.update(updateBody).pipe(Effect.provide(testLayer)),
        ),
      ).rejects.toThrow('Invalid email format')
    })
  })

  describe('delete behavior', () => {
    test('should delete user successfully', async () => {
      // Given: endpoint returns 200
      const userId = 'user-123'

      const mockHttpClient = createMockHttpClient((request) => {
        if (
          request.url.includes('/api/auth/users') &&
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
      const testLayer = UsersLive.pipe(Layer.provide(httpLayer))

      // When: deleting user
      const users = await Effect.runPromise(
        Effect.gen(function* () {
          return yield* Users
        }).pipe(Effect.provide(testLayer)),
      )

      const result = await Effect.runPromise(
        users.delete(userId).pipe(Effect.provide(testLayer)),
      )

      // Then: should succeed (void return)
      expect(result).toBeUndefined()
    })

    test('should fail with 403 permission error', async () => {
      // Given: endpoint returns 403
      const userId = 'user-123'

      const mockHttpClient = createMockHttpClient((request) => {
        if (
          request.url.includes('/api/auth/users') &&
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
      const testLayer = UsersLive.pipe(Layer.provide(httpLayer))

      // When: deleting user
      const users = await Effect.runPromise(
        Effect.gen(function* () {
          return yield* Users
        }).pipe(Effect.provide(testLayer)),
      )

      // Then: should fail with permission error
      await expect(
        Effect.runPromise(users.delete(userId).pipe(Effect.provide(testLayer))),
      ).rejects.toThrow('You do not have permission to delete users.')
    })

    test('should fail with error message from response', async () => {
      // Given: endpoint returns 404 with error
      const userId = 'user-123'

      const mockHttpClient = createMockHttpClient((request) => {
        if (
          request.url.includes('/api/auth/users') &&
          request.method === 'DELETE'
        ) {
          return Effect.succeed({
            status: 404,
            json: Effect.succeed({ error: 'User not found' }),
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
      const testLayer = UsersLive.pipe(Layer.provide(httpLayer))

      // When: deleting user
      const users = await Effect.runPromise(
        Effect.gen(function* () {
          return yield* Users
        }).pipe(Effect.provide(testLayer)),
      )

      // Then: should fail with error message
      await expect(
        Effect.runPromise(users.delete(userId).pipe(Effect.provide(testLayer))),
      ).rejects.toThrow('User not found')
    })
  })
})
