/**
 * BDD tests for Users service
 * Tests verify service interface, types, and all CRUD operations
 */
import { describe, test, expect } from 'bun:test'
import { Effect } from 'effect'

import {
  Users,
  type UsersService,
} from './Users'
import { User, UserMembership } from '../domain/User'
import { createUsersMock, UsersMockLayer } from './UsersMock'

describe('Users service', () => {
  describe('service tag', () => {
    test('should export Users tag', () => {
      // Given the Users module
      // When I check the Users export
      // Then it should be a Context tag
      expect(Users).toBeDefined()
      expect(typeof Users).toBe('object')
    })
  })

  describe('service interface', () => {
    test('should export UsersService interface', () => {
      // Given the Users module
      // When I use UsersService type
      // Then it should be usable
      const service: UsersService = {
        list: () => Effect.succeed([]),
        create: () => Effect.succeed(undefined),
        update: () => Effect.succeed(undefined),
        delete: () => Effect.succeed(undefined),
      }
      expect(service).toBeDefined()
      expect(typeof service.list).toBe('function')
      expect(typeof service.create).toBe('function')
      expect(typeof service.update).toBe('function')
      expect(typeof service.delete).toBe('function')
    })

    test('should have list method returning Effect<User[]>', async () => {
      // Given a UsersService implementation
      // When I call list
      // Then it should return Effect<readonly User[], Error, never>
      const service: UsersService = {
        list: () => Effect.succeed([]),
        create: () => Effect.succeed(undefined),
        update: () => Effect.succeed(undefined),
        delete: () => Effect.succeed(undefined),
      }

      const result = await Effect.runPromise(service.list())
      expect(Array.isArray(result)).toBe(true)
    })

    test('should have create method accepting body', async () => {
      // Given a UsersService implementation
      // When I call create with body
      // Then it should return Effect<void, Error, never>
      const service: UsersService = {
        list: () => Effect.succeed([]),
        create: () => Effect.succeed(undefined),
        update: () => Effect.succeed(undefined),
        delete: () => Effect.succeed(undefined),
      }

      const result = await Effect.runPromise(
        service.create({
          email: 'test@example.com',
          password: 'password123',
          org_id: 'org-1',
          role_ids: ['role-1'],
        })
      )
      expect(result).toBeUndefined()
    })

    test('should have update method accepting body', async () => {
      // Given a UsersService implementation
      // When I call update with body
      // Then it should return Effect<void, Error, never>
      const service: UsersService = {
        list: () => Effect.succeed([]),
        create: () => Effect.succeed(undefined),
        update: () => Effect.succeed(undefined),
        delete: () => Effect.succeed(undefined),
      }

      const result = await Effect.runPromise(
        service.update({ id: 'user-1', email: 'updated@example.com' })
      )
      expect(result).toBeUndefined()
    })

    test('should have delete method accepting id', async () => {
      // Given a UsersService implementation
      // When I call delete with id
      // Then it should return Effect<void, Error, never>
      const service: UsersService = {
        list: () => Effect.succeed([]),
        create: () => Effect.succeed(undefined),
        update: () => Effect.succeed(undefined),
        delete: () => Effect.succeed(undefined),
      }

      const result = await Effect.runPromise(service.delete('user-1'))
      expect(result).toBeUndefined()
    })
  })

  describe('list method', () => {
    test('should return all users', async () => {
      // Given: mock service with multiple users
      const users = [
        new User({
          id: 'user-1',
          email: 'user1@example.com',
          is_active: true,
          is_admin: false,
          created_at: '2024-01-01T00:00:00Z',
          memberships: [],
        }),
        new User({
          id: 'user-2',
          email: 'user2@example.com',
          is_active: true,
          is_admin: false,
          created_at: '2024-01-02T00:00:00Z',
          memberships: [],
        }),
      ]

      const program = Effect.gen(function* () {
        const service = yield* Users
        return yield* service.list()
      })

      const result = await Effect.runPromise(
        program.pipe(Effect.provide(UsersMockLayer(users)))
      )

      expect(result.length).toBe(2)
      expect(result[0].id).toBe('user-1')
      expect(result[1].id).toBe('user-2')
    })

    test('should return empty array when no users', async () => {
      // Given: mock service with no users
      const program = Effect.gen(function* () {
        const service = yield* Users
        return yield* service.list()
      })

      const result = await Effect.runPromise(
        program.pipe(Effect.provide(UsersMockLayer([])))
      )

      expect(result.length).toBe(0)
    })

    test('should return readonly array', async () => {
      // Given: mock service with users
      const users = [
        new User({
          id: 'user-1',
          email: 'user1@example.com',
          is_active: true,
          is_admin: false,
          created_at: '2024-01-01T00:00:00Z',
          memberships: [],
        }),
      ]

      const program = Effect.gen(function* () {
        const service = yield* Users
        return yield* service.list()
      })

      const result = await Effect.runPromise(
        program.pipe(Effect.provide(UsersMockLayer(users)))
      )

      expect(Array.isArray(result)).toBe(true)
      expect(result.length).toBe(1)
    })

    test('should return users with memberships', async () => {
      // Given: mock service with user having memberships
      const membership = new UserMembership({
        org_id: 'org-1',
        org_name: 'Organization 1',
        roles: ['role-1', 'role-2'],
      })
      const users = [
        new User({
          id: 'user-1',
          email: 'user1@example.com',
          is_active: true,
          is_admin: false,
          created_at: '2024-01-01T00:00:00Z',
          memberships: [membership],
        }),
      ]

      const program = Effect.gen(function* () {
        const service = yield* Users
        return yield* service.list()
      })

      const result = await Effect.runPromise(
        program.pipe(Effect.provide(UsersMockLayer(users)))
      )

      expect(result[0].memberships.length).toBe(1)
      expect(result[0].memberships[0].org_id).toBe('org-1')
      expect(result[0].memberships[0].roles).toEqual(['role-1', 'role-2'])
    })
  })

  describe('create method', () => {
    test('should create a new user', async () => {
      // Given: mock service with no users
      const program = Effect.gen(function* () {
        const service = yield* Users
        yield* service.create({
          email: 'newuser@example.com',
          password: 'password123',
          org_id: 'org-1',
          role_ids: ['role-1'],
        })
        return yield* service.list()
      })

      const result = await Effect.runPromise(
        program.pipe(Effect.provide(UsersMockLayer([])))
      )

      expect(result.length).toBe(1)
      expect(result[0].email).toBe('newuser@example.com')
      expect(result[0].is_active).toBe(true)
      expect(result[0].is_admin).toBe(false)
      expect(result[0].id).toBeDefined()
    })

    test('should generate unique id for new user', async () => {
      // Given: mock service with no users
      const program = Effect.gen(function* () {
        const service = yield* Users
        yield* service.create({
          email: 'user1@example.com',
          password: 'password123',
          org_id: 'org-1',
          role_ids: [],
        })
        yield* service.create({
          email: 'user2@example.com',
          password: 'password123',
          org_id: 'org-1',
          role_ids: [],
        })
        return yield* service.list()
      })

      const result = await Effect.runPromise(
        program.pipe(Effect.provide(UsersMockLayer([])))
      )

      expect(result.length).toBe(2)
      expect(result[0].id).not.toBe(result[1].id)
    })

    test('should set is_active to true for new users', async () => {
      // Given: mock service with no users
      const program = Effect.gen(function* () {
        const service = yield* Users
        yield* service.create({
          email: 'newuser@example.com',
          password: 'password123',
          org_id: 'org-1',
          role_ids: [],
        })
        return yield* service.list()
      })

      const result = await Effect.runPromise(
        program.pipe(Effect.provide(UsersMockLayer([])))
      )

      expect(result[0].is_active).toBe(true)
    })

    test('should set is_admin to false for new users', async () => {
      // Given: mock service with no users
      const program = Effect.gen(function* () {
        const service = yield* Users
        yield* service.create({
          email: 'newuser@example.com',
          password: 'password123',
          org_id: 'org-1',
          role_ids: [],
        })
        return yield* service.list()
      })

      const result = await Effect.runPromise(
        program.pipe(Effect.provide(UsersMockLayer([])))
      )

      expect(result[0].is_admin).toBe(false)
    })

    test('should initialize memberships as empty array', async () => {
      // Given: mock service with no users
      const program = Effect.gen(function* () {
        const service = yield* Users
        yield* service.create({
          email: 'newuser@example.com',
          password: 'password123',
          org_id: 'org-1',
          role_ids: [],
        })
        return yield* service.list()
      })

      const result = await Effect.runPromise(
        program.pipe(Effect.provide(UsersMockLayer([])))
      )

      expect(result[0].memberships).toEqual([])
    })
  })

  describe('update method', () => {
    test('should update user email', async () => {
      // Given: mock service with existing user
      const users = [
        new User({
          id: 'user-1',
          email: 'old@example.com',
          is_active: true,
          is_admin: false,
          created_at: '2024-01-01T00:00:00Z',
          memberships: [],
        }),
      ]

      const program = Effect.gen(function* () {
        const service = yield* Users
        yield* service.update({
          id: 'user-1',
          email: 'new@example.com',
        })
        return yield* service.list()
      })

      const result = await Effect.runPromise(
        program.pipe(Effect.provide(UsersMockLayer(users)))
      )

      expect(result[0].email).toBe('new@example.com')
      expect(result[0].is_active).toBe(true) // Unchanged
    })

    test('should update user is_active', async () => {
      // Given: mock service with existing user
      const users = [
        new User({
          id: 'user-1',
          email: 'user@example.com',
          is_active: true,
          is_admin: false,
          created_at: '2024-01-01T00:00:00Z',
          memberships: [],
        }),
      ]

      const program = Effect.gen(function* () {
        const service = yield* Users
        yield* service.update({
          id: 'user-1',
          is_active: false,
        })
        return yield* service.list()
      })

      const result = await Effect.runPromise(
        program.pipe(Effect.provide(UsersMockLayer(users)))
      )

      expect(result[0].is_active).toBe(false)
      expect(result[0].email).toBe('user@example.com') // Unchanged
    })

    test('should update both email and is_active', async () => {
      // Given: mock service with existing user
      const users = [
        new User({
          id: 'user-1',
          email: 'old@example.com',
          is_active: true,
          is_admin: false,
          created_at: '2024-01-01T00:00:00Z',
          memberships: [],
        }),
      ]

      const program = Effect.gen(function* () {
        const service = yield* Users
        yield* service.update({
          id: 'user-1',
          email: 'new@example.com',
          is_active: false,
        })
        return yield* service.list()
      })

      const result = await Effect.runPromise(
        program.pipe(Effect.provide(UsersMockLayer(users)))
      )

      expect(result[0].email).toBe('new@example.com')
      expect(result[0].is_active).toBe(false)
    })

    test('should fail when updating non-existent user', async () => {
      // Given: mock service with no users
      const program = Effect.gen(function* () {
        const service = yield* Users
        return yield* service.update({
          id: 'non-existent',
          email: 'new@example.com',
        })
      })

      await expect(
        Effect.runPromise(program.pipe(Effect.provide(UsersMockLayer([]))))
      ).rejects.toThrow('User not found.')
    })

    test('should preserve is_admin when updating', async () => {
      // Given: mock service with existing admin user
      const users = [
        new User({
          id: 'user-1',
          email: 'admin@example.com',
          is_active: true,
          is_admin: true,
          created_at: '2024-01-01T00:00:00Z',
          memberships: [],
        }),
      ]

      const program = Effect.gen(function* () {
        const service = yield* Users
        yield* service.update({
          id: 'user-1',
          email: 'updated@example.com',
        })
        return yield* service.list()
      })

      const result = await Effect.runPromise(
        program.pipe(Effect.provide(UsersMockLayer(users)))
      )

      expect(result[0].is_admin).toBe(true) // Preserved
      expect(result[0].email).toBe('updated@example.com')
    })

    test('should preserve created_at when updating', async () => {
      // Given: mock service with existing user
      const users = [
        new User({
          id: 'user-1',
          email: 'user@example.com',
          is_active: true,
          is_admin: false,
          created_at: '2024-01-01T00:00:00Z',
          memberships: [],
        }),
      ]

      const program = Effect.gen(function* () {
        const service = yield* Users
        yield* service.update({
          id: 'user-1',
          email: 'updated@example.com',
        })
        return yield* service.list()
      })

      const result = await Effect.runPromise(
        program.pipe(Effect.provide(UsersMockLayer(users)))
      )

      expect(result[0].created_at).toBe('2024-01-01T00:00:00Z')
    })

    test('should preserve memberships when updating', async () => {
      // Given: mock service with existing user with memberships
      const membership = new UserMembership({
        org_id: 'org-1',
        org_name: 'Organization 1',
        roles: ['role-1'],
      })
      const users = [
        new User({
          id: 'user-1',
          email: 'user@example.com',
          is_active: true,
          is_admin: false,
          created_at: '2024-01-01T00:00:00Z',
          memberships: [membership],
        }),
      ]

      const program = Effect.gen(function* () {
        const service = yield* Users
        yield* service.update({
          id: 'user-1',
          email: 'updated@example.com',
        })
        return yield* service.list()
      })

      const result = await Effect.runPromise(
        program.pipe(Effect.provide(UsersMockLayer(users)))
      )

      expect(result[0].memberships.length).toBe(1)
      expect(result[0].memberships[0].org_id).toBe('org-1')
    })
  })

  describe('delete method', () => {
    test('should delete user by id', async () => {
      // Given: mock service with users
      const users = [
        new User({
          id: 'user-1',
          email: 'user1@example.com',
          is_active: true,
          is_admin: false,
          created_at: '2024-01-01T00:00:00Z',
          memberships: [],
        }),
        new User({
          id: 'user-2',
          email: 'user2@example.com',
          is_active: true,
          is_admin: false,
          created_at: '2024-01-02T00:00:00Z',
          memberships: [],
        }),
      ]

      const program = Effect.gen(function* () {
        const service = yield* Users
        yield* service.delete('user-1')
        return yield* service.list()
      })

      const result = await Effect.runPromise(
        program.pipe(Effect.provide(UsersMockLayer(users)))
      )

      expect(result.length).toBe(1)
      expect(result[0].id).toBe('user-2')
    })

    test('should fail when deleting non-existent user', async () => {
      // Given: mock service with users
      const users = [
        new User({
          id: 'user-1',
          email: 'user1@example.com',
          is_active: true,
          is_admin: false,
          created_at: '2024-01-01T00:00:00Z',
          memberships: [],
        }),
      ]

      const program = Effect.gen(function* () {
        const service = yield* Users
        return yield* service.delete('non-existent')
      })

      await expect(
        Effect.runPromise(program.pipe(Effect.provide(UsersMockLayer(users))))
      ).rejects.toThrow('User not found.')
    })

    test('should delete all users when called multiple times', async () => {
      // Given: mock service with multiple users
      const users = [
        new User({
          id: 'user-1',
          email: 'user1@example.com',
          is_active: true,
          is_admin: false,
          created_at: '2024-01-01T00:00:00Z',
          memberships: [],
        }),
        new User({
          id: 'user-2',
          email: 'user2@example.com',
          is_active: true,
          is_admin: false,
          created_at: '2024-01-02T00:00:00Z',
          memberships: [],
        }),
        new User({
          id: 'user-3',
          email: 'user3@example.com',
          is_active: true,
          is_admin: false,
          created_at: '2024-01-03T00:00:00Z',
          memberships: [],
        }),
      ]

      const program = Effect.gen(function* () {
        const service = yield* Users
        yield* service.delete('user-1')
        yield* service.delete('user-2')
        yield* service.delete('user-3')
        return yield* service.list()
      })

      const result = await Effect.runPromise(
        program.pipe(Effect.provide(UsersMockLayer(users)))
      )

      expect(result.length).toBe(0)
    })
  })

  describe('service layer integration', () => {
    test('should be usable with Effect.provide', async () => {
      // Given UsersMockLayer
      // When I provide it to an effect
      // Then I should be able to access the service
      const program = Effect.gen(function* () {
        const service = yield* Users
        expect(service).toBeDefined()
        expect(typeof service.list).toBe('function')
        return service
      })

      const service = await Effect.runPromise(
        program.pipe(Effect.provide(UsersMockLayer([])))
      )

      expect(service).toBeDefined()
      expect(typeof service.list).toBe('function')
    })

    test('should work with multiple service accesses', async () => {
      // Given UsersMockLayer
      // When I access the service multiple times
      // Then it should return the same instance (Layer.succeed behavior)
      const program = Effect.gen(function* () {
        const service1 = yield* Users
        const service2 = yield* Users
        return { service1, service2 }
      })

      const { service1, service2 } = await Effect.runPromise(
        program.pipe(Effect.provide(UsersMockLayer([])))
      )

      // Layer.succeed returns the same instance
      expect(service1).toBe(service2)
    })
  })

  describe('integration scenarios', () => {
    test('should support full CRUD workflow', async () => {
      // Given: empty mock service
      // When I perform create, list, update, delete operations
      // Then all operations should work correctly
      const program = Effect.gen(function* () {
        const service = yield* Users

        // Create
        yield* service.create({
          email: 'test@example.com',
          password: 'password123',
          org_id: 'org-1',
          role_ids: ['role-1'],
        })

        // List and verify
        const afterCreate = yield* service.list()
        expect(afterCreate.length).toBe(1)
        const createdId = afterCreate[0].id

        // Update
        yield* service.update({
          id: createdId,
          email: 'updated@example.com',
        })

        // List and verify update
        const afterUpdate = yield* service.list()
        expect(afterUpdate[0].email).toBe('updated@example.com')

        // Delete
        yield* service.delete(createdId)

        // List and verify deletion
        const afterDelete = yield* service.list()
        expect(afterDelete.length).toBe(0)

        return { success: true }
      })

      const result = await Effect.runPromise(
        program.pipe(Effect.provide(UsersMockLayer([])))
      )

      expect(result.success).toBe(true)
    })

    test('should maintain state across multiple operations', async () => {
      // Given: mock service
      // When I perform multiple operations
      // Then state should be maintained correctly
      const program = Effect.gen(function* () {
        const service = yield* Users

        // Create multiple users
        yield* service.create({
          email: 'user1@example.com',
          password: 'password123',
          org_id: 'org-1',
          role_ids: [],
        })
        yield* service.create({
          email: 'user2@example.com',
          password: 'password123',
          org_id: 'org-1',
          role_ids: [],
        })

        // List all
        const allUsers = yield* service.list()
        expect(allUsers.length).toBe(2)

        // Update one
        yield* service.update({
          id: allUsers[0].id,
          email: 'updated-user1@example.com',
        })

        // Delete one
        yield* service.delete(allUsers[1].id)

        // Verify final state
        const finalUsers = yield* service.list()
        expect(finalUsers.length).toBe(1)
        expect(finalUsers[0].email).toBe('updated-user1@example.com')

        return { success: true }
      })

      const result = await Effect.runPromise(
        program.pipe(Effect.provide(UsersMockLayer([])))
      )

      expect(result.success).toBe(true)
    })

    test('should handle users with memberships correctly', async () => {
      // Given: mock service with user having memberships
      const membership = new UserMembership({
        org_id: 'org-1',
        org_name: 'Organization 1',
        roles: ['role-1', 'role-2'],
      })
      const users = [
        new User({
          id: 'user-1',
          email: 'user@example.com',
          is_active: true,
          is_admin: false,
          created_at: '2024-01-01T00:00:00Z',
          memberships: [membership],
        }),
      ]

      const program = Effect.gen(function* () {
        const service = yield* Users
        const listed = yield* service.list()
        expect(listed[0].memberships.length).toBe(1)
        expect(listed[0].memberships[0].roles).toEqual(['role-1', 'role-2'])

        // Update should preserve memberships
        yield* service.update({
          id: 'user-1',
          email: 'updated@example.com',
        })

        const afterUpdate = yield* service.list()
        expect(afterUpdate[0].memberships.length).toBe(1)

        return { success: true }
      })

      const result = await Effect.runPromise(
        program.pipe(Effect.provide(UsersMockLayer(users)))
      )

      expect(result.success).toBe(true)
    })
  })
})
