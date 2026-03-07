/**
 * BDD tests for RolesMock
 * Tests verify the behavior of the mock Roles implementation for testing
 */

import { describe, it, expect, beforeEach } from 'bun:test'
import { Effect, Layer } from 'effect'
import { createRolesMock, RolesMockLayer } from './RolesMock'
import { Roles } from './Roles'
import { Role } from '../domain/Role'
import { DashboardRole } from '../domain/DashboardRole'
import type { RolesService } from './Roles'

describe('RolesMock', () => {
  describe('createRolesMock behavior', () => {
    it('should return a RolesService implementation', () => {
      // Given createRolesMock function
      // When I call it
      const mock = createRolesMock()

      // Then it should return a RolesService
      expect(mock).toBeDefined()
      expect(mock.list).toBeDefined()
      expect(mock.listByOrg).toBeDefined()
      expect(mock.create).toBeDefined()
      expect(mock.update).toBeDefined()
      expect(mock.delete).toBeDefined()
      expect(typeof mock.list).toBe('function')
      expect(typeof mock.listByOrg).toBe('function')
      expect(typeof mock.create).toBe('function')
      expect(typeof mock.update).toBe('function')
      expect(typeof mock.delete).toBe('function')
    })

    it('should initialize with empty roles when no initial provided', async () => {
      // Given createRolesMock function
      // When I call it without initial roles
      const mock = createRolesMock()
      const layer = Layer.succeed(Roles, mock)

      // Then list should return empty array
      const program = Effect.gen(function* () {
        const roles = yield* Roles
        return yield* roles.list()
      })

      const result = await Effect.runPromise(program.pipe(Effect.provide(layer)))
      expect(result).toEqual([])
    })

    it('should initialize with provided roles', async () => {
      // Given initial roles
      const initialRoles = [
        new Role({
          id: 'role-1',
          org_id: 'org-1',
          name: 'admin',
          display_name: 'Administrator',
        }),
        new Role({
          id: 'role-2',
          org_id: 'org-1',
          name: 'user',
          display_name: 'User',
        }),
      ]

      // When I create mock with initial roles
      const mock = createRolesMock(initialRoles)
      const layer = Layer.succeed(Roles, mock)

      // Then list should return the initial roles
      const program = Effect.gen(function* () {
        const roles = yield* Roles
        return yield* roles.list()
      })

      const result = await Effect.runPromise(program.pipe(Effect.provide(layer)))
      expect(result).toHaveLength(2)
      expect(result[0]).toEqual(initialRoles[0])
      expect(result[1]).toEqual(initialRoles[1])
    })

    it('should convert plain objects to Role instances', async () => {
      // Given plain object roles
      const plainRoles = [
        {
          id: 'role-1',
          org_id: 'org-1',
          name: 'admin',
          display_name: 'Administrator',
        },
        {
          id: 'role-2',
          org_id: 'org-2',
          name: 'user',
          display_name: null,
        },
      ]

      // When I create mock with plain objects
      const mock = createRolesMock(plainRoles as any)
      const layer = Layer.succeed(Roles, mock)

      // Then list should return Role instances
      const program = Effect.gen(function* () {
        const roles = yield* Roles
        return yield* roles.list()
      })

      const result = await Effect.runPromise(program.pipe(Effect.provide(layer)))
      expect(result).toHaveLength(2)
      expect(result[0]).toBeInstanceOf(Role)
      expect(result[1]).toBeInstanceOf(Role)
      expect(result[0].name).toBe('admin')
      expect(result[1].org_id).toBe('org-2')
    })
  })

  describe('list behavior', () => {
    let mock: RolesService
    let layer: Layer.Layer<Roles>

    beforeEach(() => {
      mock = createRolesMock()
      layer = Layer.succeed(Roles, mock)
    })

    it('should return empty array when no roles', async () => {
      // Given a mock with no roles
      // When I call list
      const program = Effect.gen(function* () {
        const roles = yield* Roles
        return yield* roles.list()
      })

      // Then it should return empty array
      const result = await Effect.runPromise(program.pipe(Effect.provide(layer)))
      expect(result).toEqual([])
    })

    it('should return all roles', async () => {
      // Given a mock with roles
      const initialRoles = [
        new Role({
          id: 'role-1',
          org_id: 'org-1',
          name: 'admin',
          display_name: 'Administrator',
        }),
        new Role({
          id: 'role-2',
          org_id: 'org-1',
          name: 'user',
          display_name: 'User',
        }),
      ]
      const mockWithRoles = createRolesMock(initialRoles)
      const layerWithRoles = Layer.succeed(Roles, mockWithRoles)

      // When I call list
      const program = Effect.gen(function* () {
        const roles = yield* Roles
        return yield* roles.list()
      })

      // Then it should return all roles
      const result = await Effect.runPromise(
        program.pipe(Effect.provide(layerWithRoles)),
      )
      expect(result).toHaveLength(2)
      expect(result).toEqual(initialRoles)
    })

    it('should return copies of roles', async () => {
      // Given a mock with roles
      const initialRoles = [
        new Role({
          id: 'role-1',
          org_id: 'org-1',
          name: 'admin',
          display_name: 'Administrator',
        }),
      ]
      const mockWithRoles = createRolesMock(initialRoles)
      const layerWithRoles = Layer.succeed(Roles, mockWithRoles)

      // When I call list multiple times
      const program = Effect.gen(function* () {
        const roles = yield* Roles
        return yield* roles.list()
      })

      const result1 = await Effect.runPromise(
        program.pipe(Effect.provide(layerWithRoles)),
      )
      const result2 = await Effect.runPromise(
        program.pipe(Effect.provide(layerWithRoles)),
      )

      // Then results should be separate arrays (copies)
      expect(result1).not.toBe(result2)
      expect(result1).toEqual(result2)
    })
  })

  describe('listByOrg behavior', () => {
    let mock: RolesService
    let layer: Layer.Layer<Roles>

    beforeEach(() => {
      mock = createRolesMock([
        new Role({
          id: 'role-1',
          org_id: 'org-1',
          name: 'admin',
          display_name: 'Administrator',
        }),
        new Role({
          id: 'role-2',
          org_id: 'org-1',
          name: 'user',
          display_name: 'User',
        }),
        new Role({
          id: 'role-3',
          org_id: 'org-2',
          name: 'viewer',
          display_name: 'Viewer',
        }),
      ])
      layer = Layer.succeed(Roles, mock)
    })

    it('should return roles filtered by org_id', async () => {
      // Given a mock with roles from multiple orgs
      // When I call listByOrg for org-1
      const program = Effect.gen(function* () {
        const roles = yield* Roles
        return yield* roles.listByOrg('org-1')
      })

      // Then it should return only roles for org-1
      const result = await Effect.runPromise(program.pipe(Effect.provide(layer)))
      expect(result).toHaveLength(2)
      expect(result[0].id).toBe('role-1')
      expect(result[1].id).toBe('role-2')
    })

    it('should return empty array when no roles for org', async () => {
      // Given a mock with roles
      // When I call listByOrg for non-existent org
      const program = Effect.gen(function* () {
        const roles = yield* Roles
        return yield* roles.listByOrg('org-999')
      })

      // Then it should return empty array
      const result = await Effect.runPromise(program.pipe(Effect.provide(layer)))
      expect(result).toEqual([])
    })

    it('should return DashboardRole instances', async () => {
      // Given a mock with roles
      // When I call listByOrg
      const program = Effect.gen(function* () {
        const roles = yield* Roles
        return yield* roles.listByOrg('org-1')
      })

      // Then it should return DashboardRole instances
      const result = await Effect.runPromise(program.pipe(Effect.provide(layer)))
      expect(result[0]).toBeInstanceOf(DashboardRole)
      expect(result[1]).toBeInstanceOf(DashboardRole)
    })

    it('should map Role to DashboardRole correctly', async () => {
      // Given a mock with roles
      // When I call listByOrg
      const program = Effect.gen(function* () {
        const roles = yield* Roles
        return yield* roles.listByOrg('org-1')
      })

      // Then DashboardRole should have correct properties
      const result = await Effect.runPromise(program.pipe(Effect.provide(layer)))
      expect(result[0].id).toBe('role-1')
      expect(result[0].name).toBe('admin')
      expect(result[0].display_name).toBe('Administrator')
      expect(result[0]).not.toHaveProperty('org_id')
      expect(result[0]).not.toHaveProperty('created_at')
    })
  })

  describe('create behavior', () => {
    let mock: RolesService
    let layer: Layer.Layer<Roles>

    beforeEach(() => {
      mock = createRolesMock()
      layer = Layer.succeed(Roles, mock)
    })

    it('should create a new role', async () => {
      // Given a mock with no roles
      // When I create a role
      const program = Effect.gen(function* () {
        const roles = yield* Roles
        yield* roles.create({
          org_id: 'org-1',
          name: 'admin',
          display_name: 'Administrator',
        })
        return yield* roles.list()
      })

      // Then the role should be added
      const result = await Effect.runPromise(program.pipe(Effect.provide(layer)))
      expect(result).toHaveLength(1)
      expect(result[0].org_id).toBe('org-1')
      expect(result[0].name).toBe('admin')
      expect(result[0].display_name).toBe('Administrator')
      expect(result[0]).toBeInstanceOf(Role)
      expect(result[0].id).toBeDefined()
    })

    it('should generate unique IDs for created roles', async () => {
      // Given a mock
      // When I create multiple roles
      const program = Effect.gen(function* () {
        const roles = yield* Roles
        yield* roles.create({
          org_id: 'org-1',
          name: 'admin',
        })
        yield* roles.create({
          org_id: 'org-1',
          name: 'user',
        })
        return yield* roles.list()
      })

      // Then each role should have a unique ID
      const result = await Effect.runPromise(program.pipe(Effect.provide(layer)))
      expect(result).toHaveLength(2)
      expect(result[0].id).not.toBe(result[1].id)
    })

    it('should handle null display_name', async () => {
      // Given a mock
      // When I create a role without display_name
      const program = Effect.gen(function* () {
        const roles = yield* Roles
        yield* roles.create({
          org_id: 'org-1',
          name: 'admin',
        })
        return yield* roles.list()
      })

      // Then display_name should be null
      const result = await Effect.runPromise(program.pipe(Effect.provide(layer)))
      expect(result[0].display_name).toBeNull()
    })

    it('should handle optional display_name', async () => {
      // Given a mock
      // When I create a role with display_name
      const program = Effect.gen(function* () {
        const roles = yield* Roles
        yield* roles.create({
          org_id: 'org-1',
          name: 'admin',
          display_name: 'Administrator',
        })
        return yield* roles.list()
      })

      // Then display_name should be set
      const result = await Effect.runPromise(program.pipe(Effect.provide(layer)))
      expect(result[0].display_name).toBe('Administrator')
    })

    it('should return void Effect', async () => {
      // Given a mock
      // When I create a role
      const program = Effect.gen(function* () {
        const roles = yield* Roles
        return yield* roles.create({
          org_id: 'org-1',
          name: 'admin',
        })
      })

      // Then it should complete successfully
      await expect(
        Effect.runPromise(program.pipe(Effect.provide(layer))),
      ).resolves.toBeUndefined()
    })
  })

  describe('update behavior', () => {
    let mock: RolesService
    let layer: Layer.Layer<Roles>

    beforeEach(() => {
      mock = createRolesMock([
        new Role({
          id: 'role-1',
          org_id: 'org-1',
          name: 'admin',
          display_name: 'Administrator',
          created_at: '2024-01-01T00:00:00Z',
          updated_at: '2024-01-01T00:00:00Z',
        }),
      ])
      layer = Layer.succeed(Roles, mock)
    })

    it('should update role name', async () => {
      // Given a mock with a role
      // When I update the role name
      const program = Effect.gen(function* () {
        const roles = yield* Roles
        yield* roles.update({
          id: 'role-1',
          name: 'superadmin',
        })
        return yield* roles.list()
      })

      // Then the role should be updated
      const result = await Effect.runPromise(program.pipe(Effect.provide(layer)))
      expect(result[0].name).toBe('superadmin')
      expect(result[0].display_name).toBe('Administrator') // Unchanged
    })

    it('should update role display_name', async () => {
      // Given a mock with a role
      // When I update the display_name
      const program = Effect.gen(function* () {
        const roles = yield* Roles
        yield* roles.update({
          id: 'role-1',
          display_name: 'Super Administrator',
        })
        return yield* roles.list()
      })

      // Then the role should be updated
      const result = await Effect.runPromise(program.pipe(Effect.provide(layer)))
      expect(result[0].display_name).toBe('Super Administrator')
      expect(result[0].name).toBe('admin') // Unchanged
    })

    it('should update both name and display_name', async () => {
      // Given a mock with a role
      // When I update both fields
      const program = Effect.gen(function* () {
        const roles = yield* Roles
        yield* roles.update({
          id: 'role-1',
          name: 'superadmin',
          display_name: 'Super Administrator',
        })
        return yield* roles.list()
      })

      // Then both fields should be updated
      const result = await Effect.runPromise(program.pipe(Effect.provide(layer)))
      expect(result[0].name).toBe('superadmin')
      expect(result[0].display_name).toBe('Super Administrator')
    })

    it('should preserve existing fields when updating', async () => {
      // Given a mock with a role
      // When I update only display_name
      const program = Effect.gen(function* () {
        const roles = yield* Roles
        yield* roles.update({
          id: 'role-1',
          display_name: 'New Display Name',
        })
        return yield* roles.list()
      })

      // Then other fields should be preserved
      const result = await Effect.runPromise(program.pipe(Effect.provide(layer)))
      expect(result[0].id).toBe('role-1')
      expect(result[0].org_id).toBe('org-1')
      expect(result[0].name).toBe('admin')
      expect(result[0].created_at).toBe('2024-01-01T00:00:00Z')
      expect(result[0].updated_at).toBe('2024-01-01T00:00:00Z')
    })

    it('should fail when role not found', async () => {
      // Given a mock with roles
      // When I update a non-existent role
      const program = Effect.gen(function* () {
        const roles = yield* Roles
        return yield* roles.update({
          id: 'nonexistent',
          name: 'new name',
        })
      })

      // Then it should fail with error
      await expect(
        Effect.runPromise(program.pipe(Effect.provide(layer))),
      ).rejects.toThrow('Role not found')
    })

    it('should return void Effect on success', async () => {
      // Given a mock with a role
      // When I update the role
      const program = Effect.gen(function* () {
        const roles = yield* Roles
        return yield* roles.update({
          id: 'role-1',
          name: 'updated',
        })
      })

      // Then it should complete successfully
      await expect(
        Effect.runPromise(program.pipe(Effect.provide(layer))),
      ).resolves.toBeUndefined()
    })
  })

  describe('delete behavior', () => {
    let mock: RolesService
    let layer: Layer.Layer<Roles>

    beforeEach(() => {
      mock = createRolesMock([
        new Role({
          id: 'role-1',
          org_id: 'org-1',
          name: 'admin',
          display_name: 'Administrator',
        }),
        new Role({
          id: 'role-2',
          org_id: 'org-1',
          name: 'user',
          display_name: 'User',
        }),
      ])
      layer = Layer.succeed(Roles, mock)
    })

    it('should delete an existing role', async () => {
      // Given a mock with roles
      // When I delete a role
      const program = Effect.gen(function* () {
        const roles = yield* Roles
        yield* roles.delete('role-1')
        return yield* roles.list()
      })

      // Then the role should be removed
      const result = await Effect.runPromise(program.pipe(Effect.provide(layer)))
      expect(result).toHaveLength(1)
      expect(result[0].id).toBe('role-2')
    })

    it('should fail when role not found', async () => {
      // Given a mock with roles
      // When I delete a non-existent role
      const program = Effect.gen(function* () {
        const roles = yield* Roles
        return yield* roles.delete('nonexistent')
      })

      // Then it should fail with error
      await expect(
        Effect.runPromise(program.pipe(Effect.provide(layer))),
      ).rejects.toThrow('Role not found')
    })

    it('should return void Effect on success', async () => {
      // Given a mock with roles
      // When I delete a role
      const program = Effect.gen(function* () {
        const roles = yield* Roles
        return yield* roles.delete('role-1')
      })

      // Then it should complete successfully
      await expect(
        Effect.runPromise(program.pipe(Effect.provide(layer))),
      ).resolves.toBeUndefined()
    })
  })

  describe('RolesMockLayer behavior', () => {
    it('should provide a valid Roles service', async () => {
      // Given RolesMockLayer
      // When I use it in an Effect program
      const program = Effect.gen(function* () {
        const roles = yield* Roles
        return roles
      })

      // Then it should provide a valid Roles service
      const result = await Effect.runPromise(
        program.pipe(Effect.provide(RolesMockLayer())),
      )
      expect(result).toBeDefined()
      expect(result.list).toBeDefined()
      expect(result.listByOrg).toBeDefined()
      expect(result.create).toBeDefined()
      expect(result.update).toBeDefined()
      expect(result.delete).toBeDefined()
    })

    it('should accept initial roles', async () => {
      // Given RolesMockLayer with initial roles
      const initialRoles = [
        new Role({
          id: 'role-1',
          org_id: 'org-1',
          name: 'admin',
          display_name: 'Administrator',
        }),
      ]

      // When I use it in an Effect program
      const program = Effect.gen(function* () {
        const roles = yield* Roles
        return yield* roles.list()
      })

      // Then it should return the initial roles
      const result = await Effect.runPromise(
        program.pipe(Effect.provide(RolesMockLayer(initialRoles))),
      )
      expect(result).toHaveLength(1)
      expect(result[0]).toEqual(initialRoles[0])
    })

    it('should allow calling methods', async () => {
      // Given RolesMockLayer
      // When I call methods on the service
      const program = Effect.gen(function* () {
        const roles = yield* Roles
        const initialList = yield* roles.list()
        yield* roles.create({
          org_id: 'org-1',
          name: 'admin',
        })
        const updatedList = yield* roles.list()
        return { initial: initialList, updated: updatedList }
      })

      // Then it should execute successfully
      const result = await Effect.runPromise(
        program.pipe(Effect.provide(RolesMockLayer())),
      )
      expect(result.initial).toHaveLength(0)
      expect(result.updated).toHaveLength(1)
    })
  })

  describe('integration behavior', () => {
    it('should support full CRUD flow', async () => {
      // Given a mock Roles service
      const mock = createRolesMock()
      const layer = Layer.succeed(Roles, mock)

      // When I perform a full CRUD flow
      const program = Effect.gen(function* () {
        const roles = yield* Roles

        // Create
        yield* roles.create({
          org_id: 'org-1',
          name: 'admin',
          display_name: 'Administrator',
        })
        yield* roles.create({
          org_id: 'org-1',
          name: 'user',
          display_name: 'User',
        })

        // Read
        const list1 = yield* roles.list()
        expect(list1).toHaveLength(2)

        const byOrg = yield* roles.listByOrg('org-1')
        expect(byOrg).toHaveLength(2)

        // Update
        yield* roles.update({
          id: list1[0].id,
          name: 'superadmin',
        })

        // Delete
        yield* roles.delete(list1[1].id)

        // Read again
        const list2 = yield* roles.list()
        return list2
      })

      // Then it should complete successfully
      const result = await Effect.runPromise(program.pipe(Effect.provide(layer)))
      expect(result).toHaveLength(1)
      expect(result[0].name).toBe('superadmin')
    })

    it('should maintain separate state per mock instance', async () => {
      // Given two separate mock instances
      const mock1 = createRolesMock([
        new Role({
          id: 'role-1',
          org_id: 'org-1',
          name: 'admin',
          display_name: 'Administrator',
        }),
      ])
      const mock2 = createRolesMock([
        new Role({
          id: 'role-2',
          org_id: 'org-2',
          name: 'user',
          display_name: 'User',
        }),
      ])

      const layer1 = Layer.succeed(Roles, mock1)
      const layer2 = Layer.succeed(Roles, mock2)

      // When I get list from both
      const program = Effect.gen(function* () {
        const roles = yield* Roles
        return yield* roles.list()
      })

      const result1 = await Effect.runPromise(program.pipe(Effect.provide(layer1)))
      const result2 = await Effect.runPromise(program.pipe(Effect.provide(layer2)))

      // Then they should have separate state
      expect(result1).toHaveLength(1)
      expect(result1[0].id).toBe('role-1')
      expect(result2).toHaveLength(1)
      expect(result2[0].id).toBe('role-2')
    })
  })
})
