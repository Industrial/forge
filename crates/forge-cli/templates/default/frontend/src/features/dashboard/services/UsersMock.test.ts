/**
 * BDD tests for UsersMock
 * Tests verify the behavior of the mock Users implementation for testing
 */

import { describe, it, expect, beforeEach } from 'bun:test'
import { Effect, Layer } from 'effect'
import { createUsersMock, UsersMockLayer } from './UsersMock'
import { Users } from './Users'
import { User, UserMembership } from '../domain/User'
import type { UsersService } from './Users'

describe('UsersMock', () => {
  describe('createUsersMock behavior', () => {
    it('should return a UsersService implementation', () => {
      // Given createUsersMock function
      // When I call it
      const mock = createUsersMock()

      // Then it should return a UsersService
      expect(mock).toBeDefined()
      expect(mock.list).toBeDefined()
      expect(mock.create).toBeDefined()
      expect(mock.update).toBeDefined()
      expect(mock.delete).toBeDefined()
      expect(typeof mock.list).toBe('function')
      expect(typeof mock.create).toBe('function')
      expect(typeof mock.update).toBe('function')
      expect(typeof mock.delete).toBe('function')
    })

    it('should initialize with empty users when no initial provided', async () => {
      // Given createUsersMock function
      // When I call it without initial users
      const mock = createUsersMock()
      const layer = Layer.succeed(Users, mock)

      // Then list should return empty array
      const program = Effect.gen(function* () {
        const users = yield* Users
        return yield* users.list()
      })

      const result = await Effect.runPromise(program.pipe(Effect.provide(layer)))
      expect(result).toEqual([])
    })

    it('should initialize with provided users', async () => {
      // Given initial users
      const initialUsers = [
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
          is_admin: true,
          created_at: '2024-01-02T00:00:00Z',
          memberships: [],
        }),
      ]

      // When I create mock with initial users
      const mock = createUsersMock(initialUsers)
      const layer = Layer.succeed(Users, mock)

      // Then list should return the initial users
      const program = Effect.gen(function* () {
        const users = yield* Users
        return yield* users.list()
      })

      const result = await Effect.runPromise(program.pipe(Effect.provide(layer)))
      expect(result).toHaveLength(2)
      expect(result[0]).toEqual(initialUsers[0])
      expect(result[1]).toEqual(initialUsers[1])
    })

    it('should convert plain objects to User instances', async () => {
      // Given plain object users
      const plainUsers = [
        {
          id: 'user-1',
          email: 'user1@example.com',
          is_active: true,
          is_admin: false,
          created_at: '2024-01-01T00:00:00Z',
          memberships: [],
        },
        {
          id: 'user-2',
          email: 'user2@example.com',
          is_active: false,
          is_admin: false,
          created_at: '2024-01-02T00:00:00Z',
          memberships: [],
        },
      ]

      // When I create mock with plain objects
      const mock = createUsersMock(plainUsers as any)
      const layer = Layer.succeed(Users, mock)

      // Then list should return User instances
      const program = Effect.gen(function* () {
        const users = yield* Users
        return yield* users.list()
      })

      const result = await Effect.runPromise(program.pipe(Effect.provide(layer)))
      expect(result).toHaveLength(2)
      expect(result[0]).toBeInstanceOf(User)
      expect(result[1]).toBeInstanceOf(User)
      expect(result[0].email).toBe('user1@example.com')
      expect(result[1].is_active).toBe(false)
    })
  })

  describe('list behavior', () => {
    let mock: UsersService
    let layer: Layer.Layer<Users>

    beforeEach(() => {
      mock = createUsersMock()
      layer = Layer.succeed(Users, mock)
    })

    it('should return empty array when no users', async () => {
      // Given a mock with no users
      // When I call list
      const program = Effect.gen(function* () {
        const users = yield* Users
        return yield* users.list()
      })

      // Then it should return empty array
      const result = await Effect.runPromise(program.pipe(Effect.provide(layer)))
      expect(result).toEqual([])
    })

    it('should return all users', async () => {
      // Given a mock with users
      const initialUsers = [
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
          is_admin: true,
          created_at: '2024-01-02T00:00:00Z',
          memberships: [],
        }),
      ]
      const mockWithUsers = createUsersMock(initialUsers)
      const layerWithUsers = Layer.succeed(Users, mockWithUsers)

      // When I call list
      const program = Effect.gen(function* () {
        const users = yield* Users
        return yield* users.list()
      })

      // Then it should return all users
      const result = await Effect.runPromise(
        program.pipe(Effect.provide(layerWithUsers)),
      )
      expect(result).toHaveLength(2)
      expect(result[0].email).toBe('user1@example.com')
      expect(result[1].email).toBe('user2@example.com')
    })

    it('should return a copy of users array', async () => {
      // Given a mock with users
      const initialUsers = [
        new User({
          id: 'user-1',
          email: 'user1@example.com',
          is_active: true,
          is_admin: false,
          created_at: '2024-01-01T00:00:00Z',
          memberships: [],
        }),
      ]
      const mockWithUsers = createUsersMock(initialUsers)
      const layerWithUsers = Layer.succeed(Users, mockWithUsers)

      // When I call list multiple times
      const program = Effect.gen(function* () {
        const users = yield* Users
        const list1 = yield* users.list()
        const list2 = yield* users.list()
        return { list1, list2 }
      })

      // Then it should return copies (not same reference)
      const result = await Effect.runPromise(
        program.pipe(Effect.provide(layerWithUsers)),
      )
      expect(result.list1).toEqual(result.list2)
      expect(result.list1).not.toBe(result.list2) // Different array references
    })
  })

  describe('create behavior', () => {
    let mock: UsersService
    let layer: Layer.Layer<Users>

    beforeEach(() => {
      mock = createUsersMock()
      layer = Layer.succeed(Users, mock)
    })

    it('should create a new user', async () => {
      // Given a mock with no users
      // When I create a user
      const program = Effect.gen(function* () {
        const users = yield* Users
        yield* users.create({
          email: 'newuser@example.com',
          password: 'password123',
          org_id: 'org-123',
          role_ids: ['role-1'],
        })
        return yield* users.list()
      })

      // Then it should add the user to the list
      const result = await Effect.runPromise(program.pipe(Effect.provide(layer)))
      expect(result).toHaveLength(1)
      expect(result[0].email).toBe('newuser@example.com')
      expect(result[0].is_active).toBe(true)
      expect(result[0].is_admin).toBe(false)
      expect(result[0].memberships).toEqual([])
      expect(result[0].id).toBeDefined()
      expect(result[0].created_at).toBeDefined()
    })

    it('should generate unique id for new user', async () => {
      // Given a mock
      // When I create multiple users
      const program = Effect.gen(function* () {
        const users = yield* Users
        yield* users.create({
          email: 'user1@example.com',
          password: 'password123',
          org_id: 'org-123',
          role_ids: ['role-1'],
        })
        yield* users.create({
          email: 'user2@example.com',
          password: 'password123',
          org_id: 'org-123',
          role_ids: ['role-1'],
        })
        return yield* users.list()
      })

      // Then each user should have a unique id
      const result = await Effect.runPromise(program.pipe(Effect.provide(layer)))
      expect(result).toHaveLength(2)
      expect(result[0].id).not.toBe(result[1].id)
    })

    it('should set is_active to true for new user', async () => {
      // Given a mock
      // When I create a user
      const program = Effect.gen(function* () {
        const users = yield* Users
        yield* users.create({
          email: 'newuser@example.com',
          password: 'password123',
          org_id: 'org-123',
          role_ids: ['role-1'],
        })
        return yield* users.list()
      })

      // Then the user should have is_active set to true
      const result = await Effect.runPromise(program.pipe(Effect.provide(layer)))
      expect(result[0].is_active).toBe(true)
    })

    it('should set is_admin to false for new user', async () => {
      // Given a mock
      // When I create a user
      const program = Effect.gen(function* () {
        const users = yield* Users
        yield* users.create({
          email: 'newuser@example.com',
          password: 'password123',
          org_id: 'org-123',
          role_ids: ['role-1'],
        })
        return yield* users.list()
      })

      // Then the user should have is_admin set to false
      const result = await Effect.runPromise(program.pipe(Effect.provide(layer)))
      expect(result[0].is_admin).toBe(false)
    })

    it('should set memberships to empty array for new user', async () => {
      // Given a mock
      // When I create a user
      const program = Effect.gen(function* () {
        const users = yield* Users
        yield* users.create({
          email: 'newuser@example.com',
          password: 'password123',
          org_id: 'org-123',
          role_ids: ['role-1'],
        })
        return yield* users.list()
      })

      // Then the user should have empty memberships
      const result = await Effect.runPromise(program.pipe(Effect.provide(layer)))
      expect(result[0].memberships).toEqual([])
    })

    it('should set created_at timestamp for new user', async () => {
      // Given a mock
      // When I create a user
      const program = Effect.gen(function* () {
        const users = yield* Users
        yield* users.create({
          email: 'newuser@example.com',
          password: 'password123',
          org_id: 'org-123',
          role_ids: ['role-1'],
        })
        return yield* users.list()
      })

      // Then the user should have created_at timestamp
      const result = await Effect.runPromise(program.pipe(Effect.provide(layer)))
      expect(result[0].created_at).toBeDefined()
      expect(typeof result[0].created_at).toBe('string')
      expect(new Date(result[0].created_at).getTime()).toBeGreaterThan(0)
    })
  })

  describe('update behavior', () => {
    let mock: UsersService
    let layer: Layer.Layer<Users>

    beforeEach(() => {
      const initialUsers = [
        new User({
          id: 'user-1',
          email: 'user1@example.com',
          is_active: true,
          is_admin: false,
          created_at: '2024-01-01T00:00:00Z',
          memberships: [],
        }),
      ]
      mock = createUsersMock(initialUsers)
      layer = Layer.succeed(Users, mock)
    })

    it('should update user email', async () => {
      // Given a mock with a user
      // When I update the user email
      const program = Effect.gen(function* () {
        const users = yield* Users
        yield* users.update({
          id: 'user-1',
          email: 'updated@example.com',
        })
        return yield* users.list()
      })

      // Then the user email should be updated
      const result = await Effect.runPromise(program.pipe(Effect.provide(layer)))
      expect(result[0].email).toBe('updated@example.com')
    })

    it('should update user is_active', async () => {
      // Given a mock with a user
      // When I update the user is_active
      const program = Effect.gen(function* () {
        const users = yield* Users
        yield* users.update({
          id: 'user-1',
          is_active: false,
        })
        return yield* users.list()
      })

      // Then the user is_active should be updated
      const result = await Effect.runPromise(program.pipe(Effect.provide(layer)))
      expect(result[0].is_active).toBe(false)
    })

    it('should update both email and is_active', async () => {
      // Given a mock with a user
      // When I update both email and is_active
      const program = Effect.gen(function* () {
        const users = yield* Users
        yield* users.update({
          id: 'user-1',
          email: 'updated@example.com',
          is_active: false,
        })
        return yield* users.list()
      })

      // Then both fields should be updated
      const result = await Effect.runPromise(program.pipe(Effect.provide(layer)))
      expect(result[0].email).toBe('updated@example.com')
      expect(result[0].is_active).toBe(false)
    })

    it('should preserve existing email when email not provided', async () => {
      // Given a mock with a user
      // When I update without email
      const program = Effect.gen(function* () {
        const users = yield* Users
        yield* users.update({
          id: 'user-1',
          is_active: false,
        })
        return yield* users.list()
      })

      // Then the email should be preserved
      const result = await Effect.runPromise(program.pipe(Effect.provide(layer)))
      expect(result[0].email).toBe('user1@example.com')
    })

    it('should preserve existing is_active when is_active not provided', async () => {
      // Given a mock with a user
      // When I update without is_active
      const program = Effect.gen(function* () {
        const users = yield* Users
        yield* users.update({
          id: 'user-1',
          email: 'updated@example.com',
        })
        return yield* users.list()
      })

      // Then the is_active should be preserved
      const result = await Effect.runPromise(program.pipe(Effect.provide(layer)))
      expect(result[0].is_active).toBe(true)
    })

    it('should preserve is_admin (cannot be updated)', async () => {
      // Given a mock with a user
      // When I update the user
      const program = Effect.gen(function* () {
        const users = yield* Users
        yield* users.update({
          id: 'user-1',
          email: 'updated@example.com',
        })
        return yield* users.list()
      })

      // Then is_admin should be preserved
      const result = await Effect.runPromise(program.pipe(Effect.provide(layer)))
      expect(result[0].is_admin).toBe(false)
    })

    it('should preserve created_at', async () => {
      // Given a mock with a user
      // When I update the user
      const program = Effect.gen(function* () {
        const users = yield* Users
        yield* users.update({
          id: 'user-1',
          email: 'updated@example.com',
        })
        return yield* users.list()
      })

      // Then created_at should be preserved
      const result = await Effect.runPromise(program.pipe(Effect.provide(layer)))
      expect(result[0].created_at).toBe('2024-01-01T00:00:00Z')
    })

    it('should preserve memberships', async () => {
      // Given a mock with a user with memberships
      const initialUsers = [
        new User({
          id: 'user-1',
          email: 'user1@example.com',
          is_active: true,
          is_admin: false,
          created_at: '2024-01-01T00:00:00Z',
          memberships: [
            new UserMembership({
              org_id: 'org-123',
              org_name: 'Test Org',
              roles: ['role-1'],
            }),
          ],
        }),
      ]
      const mockWithMemberships = createUsersMock(initialUsers)
      const layerWithMemberships = Layer.succeed(Users, mockWithMemberships)

      // When I update the user
      const program = Effect.gen(function* () {
        const users = yield* Users
        yield* users.update({
          id: 'user-1',
          email: 'updated@example.com',
        })
        return yield* users.list()
      })

      // Then memberships should be preserved
      const result = await Effect.runPromise(
        program.pipe(Effect.provide(layerWithMemberships)),
      )
      expect(result[0].memberships).toHaveLength(1)
      expect(result[0].memberships[0].org_id).toBe('org-123')
    })

    it('should fail when user not found', async () => {
      // Given a mock with a user
      // When I update a non-existent user
      const program = Effect.gen(function* () {
        const users = yield* Users
        return yield* users.update({
          id: 'non-existent',
          email: 'updated@example.com',
        })
      })

      // Then it should fail with error
      await expect(
        Effect.runPromise(program.pipe(Effect.provide(layer))),
      ).rejects.toThrow('User not found.')
    })
  })

  describe('delete behavior', () => {
    let mock: UsersService
    let layer: Layer.Layer<Users>

    beforeEach(() => {
      const initialUsers = [
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
      mock = createUsersMock(initialUsers)
      layer = Layer.succeed(Users, mock)
    })

    it('should delete a user by id', async () => {
      // Given a mock with users
      // When I delete a user
      const program = Effect.gen(function* () {
        const users = yield* Users
        yield* users.delete('user-1')
        return yield* users.list()
      })

      // Then the user should be removed
      const result = await Effect.runPromise(program.pipe(Effect.provide(layer)))
      expect(result).toHaveLength(1)
      expect(result[0].id).toBe('user-2')
    })

    it('should delete the correct user', async () => {
      // Given a mock with users
      // When I delete a specific user
      const program = Effect.gen(function* () {
        const users = yield* Users
        yield* users.delete('user-2')
        return yield* users.list()
      })

      // Then only that user should be removed
      const result = await Effect.runPromise(program.pipe(Effect.provide(layer)))
      expect(result).toHaveLength(1)
      expect(result[0].id).toBe('user-1')
    })

    it('should fail when user not found', async () => {
      // Given a mock with users
      // When I delete a non-existent user
      const program = Effect.gen(function* () {
        const users = yield* Users
        return yield* users.delete('non-existent')
      })

      // Then it should fail with error
      await expect(
        Effect.runPromise(program.pipe(Effect.provide(layer))),
      ).rejects.toThrow('User not found.')
    })

    it('should remove user from list', async () => {
      // Given a mock with users
      // When I delete a user
      const program = Effect.gen(function* () {
        const users = yield* Users
        const before = yield* users.list()
        yield* users.delete('user-1')
        const after = yield* users.list()
        return { before, after }
      })

      // Then the user should be removed from the list
      const result = await Effect.runPromise(program.pipe(Effect.provide(layer)))
      expect(result.before).toHaveLength(2)
      expect(result.after).toHaveLength(1)
      expect(result.after[0].id).toBe('user-2')
    })
  })

  describe('UsersMockLayer behavior', () => {
    it('should create a Layer with Users service', async () => {
      // Given UsersMockLayer function
      // When I create a layer
      const layer = UsersMockLayer()

      // Then it should provide Users service
      const program = Effect.gen(function* () {
        const users = yield* Users
        return yield* users.list()
      })

      const result = await Effect.runPromise(program.pipe(Effect.provide(layer)))
      expect(result).toEqual([])
    })

    it('should create a Layer with initial users', async () => {
      // Given initial users
      const initialUsers = [
        new User({
          id: 'user-1',
          email: 'user1@example.com',
          is_active: true,
          is_admin: false,
          created_at: '2024-01-01T00:00:00Z',
          memberships: [],
        }),
      ]

      // When I create a layer with initial users
      const layer = UsersMockLayer(initialUsers)

      // Then it should provide Users service with initial users
      const program = Effect.gen(function* () {
        const users = yield* Users
        return yield* users.list()
      })

      const result = await Effect.runPromise(program.pipe(Effect.provide(layer)))
      expect(result).toHaveLength(1)
      expect(result[0].email).toBe('user1@example.com')
    })

    it('should work with empty array', async () => {
      // Given empty array
      // When I create a layer with empty array
      const layer = UsersMockLayer([])

      // Then it should provide Users service with no users
      const program = Effect.gen(function* () {
        const users = yield* Users
        return yield* users.list()
      })

      const result = await Effect.runPromise(program.pipe(Effect.provide(layer)))
      expect(result).toEqual([])
    })
  })

  describe('integration scenarios', () => {
    it('should support full CRUD workflow', async () => {
      // Given a mock service
      const mock = createUsersMock()
      const layer = Layer.succeed(Users, mock)

      // When I perform create, read, update, delete
      const program = Effect.gen(function* () {
        const users = yield* Users

        // Create
        yield* users.create({
          email: 'newuser@example.com',
          password: 'password123',
          org_id: 'org-123',
          role_ids: ['role-1'],
        })

        // Read
        const listAfterCreate = yield* users.list()
        const userId = listAfterCreate[0].id

        // Update
        yield* users.update({
          id: userId,
          email: 'updated@example.com',
          is_active: false,
        })

        // Read after update
        const listAfterUpdate = yield* users.list()

        // Delete
        yield* users.delete(userId)

        // Read after delete
        const listAfterDelete = yield* users.list()

        return {
          listAfterCreate,
          listAfterUpdate,
          listAfterDelete,
        }
      })

      // Then all operations should work correctly
      const result = await Effect.runPromise(program.pipe(Effect.provide(layer)))
      expect(result.listAfterCreate).toHaveLength(1)
      expect(result.listAfterCreate[0].email).toBe('newuser@example.com')
      expect(result.listAfterUpdate).toHaveLength(1)
      expect(result.listAfterUpdate[0].email).toBe('updated@example.com')
      expect(result.listAfterUpdate[0].is_active).toBe(false)
      expect(result.listAfterDelete).toHaveLength(0)
    })

    it('should handle multiple users independently', async () => {
      // Given a mock service
      const mock = createUsersMock()
      const layer = Layer.succeed(Users, mock)

      // When I create multiple users and update one
      const program = Effect.gen(function* () {
        const users = yield* Users

        yield* users.create({
          email: 'user1@example.com',
          password: 'password123',
          org_id: 'org-123',
          role_ids: ['role-1'],
        })
        yield* users.create({
          email: 'user2@example.com',
          password: 'password123',
          org_id: 'org-123',
          role_ids: ['role-1'],
        })

        const list = yield* users.list()
        const user1Id = list[0].id
        const user2Id = list[1].id

        yield* users.update({
          id: user1Id,
          email: 'updated1@example.com',
        })

        yield* users.delete(user2Id)

        return yield* users.list()
      })

      // Then operations should affect only the intended users
      const result = await Effect.runPromise(program.pipe(Effect.provide(layer)))
      expect(result).toHaveLength(1)
      expect(result[0].email).toBe('updated1@example.com')
    })
  })
})
