/**
 * BDD tests for Roles service
 * Tests verify service interface, types, and all CRUD operations
 */
import { describe, test, expect } from 'bun:test'
import { Effect } from 'effect'

import {
  Roles,
  type RolesService,
} from './Roles'
import { Role } from '../domain/Role'
import { DashboardRole } from '../domain/DashboardRole'
import { createRolesMock, RolesMockLayer } from './RolesMock'

describe('Roles service', () => {
  describe('service tag', () => {
    test('should export Roles tag', () => {
      // Given the Roles module
      // When I check the Roles export
      // Then it should be a Context tag
      expect(Roles).toBeDefined()
      expect(typeof Roles).toBe('object')
    })
  })

  describe('service interface', () => {
    test('should export RolesService interface', () => {
      // Given the Roles module
      // When I use RolesService type
      // Then it should be usable
      const service: RolesService = {
        list: () => Effect.succeed([]),
        listByOrg: () => Effect.succeed([]),
        create: () => Effect.succeed(undefined),
        update: () => Effect.succeed(undefined),
        delete: () => Effect.succeed(undefined),
      }
      expect(service).toBeDefined()
      expect(typeof service.list).toBe('function')
      expect(typeof service.listByOrg).toBe('function')
      expect(typeof service.create).toBe('function')
      expect(typeof service.update).toBe('function')
      expect(typeof service.delete).toBe('function')
    })

    test('should have list method returning Effect<Role[]>', async () => {
      // Given a RolesService implementation
      // When I call list
      // Then it should return Effect<readonly Role[], Error, never>
      const service: RolesService = {
        list: () => Effect.succeed([]),
        listByOrg: () => Effect.succeed([]),
        create: () => Effect.succeed(undefined),
        update: () => Effect.succeed(undefined),
        delete: () => Effect.succeed(undefined),
      }

      const result = await Effect.runPromise(service.list())
      expect(Array.isArray(result)).toBe(true)
    })

    test('should have listByOrg method accepting orgId', async () => {
      // Given a RolesService implementation
      // When I call listByOrg with orgId
      // Then it should return Effect<readonly DashboardRole[], Error, never>
      const service: RolesService = {
        list: () => Effect.succeed([]),
        listByOrg: () => Effect.succeed([]),
        create: () => Effect.succeed(undefined),
        update: () => Effect.succeed(undefined),
        delete: () => Effect.succeed(undefined),
      }

      const result = await Effect.runPromise(service.listByOrg('org-1'))
      expect(Array.isArray(result)).toBe(true)
    })

    test('should have create method accepting body', async () => {
      // Given a RolesService implementation
      // When I call create with body
      // Then it should return Effect<void, Error, never>
      const service: RolesService = {
        list: () => Effect.succeed([]),
        listByOrg: () => Effect.succeed([]),
        create: () => Effect.succeed(undefined),
        update: () => Effect.succeed(undefined),
        delete: () => Effect.succeed(undefined),
      }

      const result = await Effect.runPromise(
        service.create({ org_id: 'org-1', name: 'test-role' })
      )
      expect(result).toBeUndefined()
    })

    test('should have update method accepting body', async () => {
      // Given a RolesService implementation
      // When I call update with body
      // Then it should return Effect<void, Error, never>
      const service: RolesService = {
        list: () => Effect.succeed([]),
        listByOrg: () => Effect.succeed([]),
        create: () => Effect.succeed(undefined),
        update: () => Effect.succeed(undefined),
        delete: () => Effect.succeed(undefined),
      }

      const result = await Effect.runPromise(
        service.update({ id: 'role-1', name: 'updated-name' })
      )
      expect(result).toBeUndefined()
    })

    test('should have delete method accepting id', async () => {
      // Given a RolesService implementation
      // When I call delete with id
      // Then it should return Effect<void, Error, never>
      const service: RolesService = {
        list: () => Effect.succeed([]),
        listByOrg: () => Effect.succeed([]),
        create: () => Effect.succeed(undefined),
        update: () => Effect.succeed(undefined),
        delete: () => Effect.succeed(undefined),
      }

      const result = await Effect.runPromise(service.delete('role-1'))
      expect(result).toBeUndefined()
    })
  })

  describe('list method', () => {
    test('should return all roles', async () => {
      // Given: mock service with multiple roles
      const roles = [
        new Role({
          id: 'role-1',
          org_id: 'org-1',
          name: 'admin',
          display_name: 'Administrator',
        }),
        new Role({
          id: 'role-2',
          org_id: 'org-2',
          name: 'user',
          display_name: 'User',
        }),
      ]

      const program = Effect.gen(function* () {
        const service = yield* Roles
        return yield* service.list()
      })

      const result = await Effect.runPromise(
        program.pipe(Effect.provide(RolesMockLayer(roles)))
      )

      expect(result.length).toBe(2)
      expect(result[0].id).toBe('role-1')
      expect(result[1].id).toBe('role-2')
    })

    test('should return empty array when no roles', async () => {
      // Given: mock service with no roles
      const program = Effect.gen(function* () {
        const service = yield* Roles
        return yield* service.list()
      })

      const result = await Effect.runPromise(
        program.pipe(Effect.provide(RolesMockLayer([])))
      )

      expect(result.length).toBe(0)
    })

    test('should return readonly array', async () => {
      // Given: mock service with roles
      const roles = [
        new Role({
          id: 'role-1',
          org_id: 'org-1',
          name: 'admin',
          display_name: 'Administrator',
        }),
      ]

      const program = Effect.gen(function* () {
        const service = yield* Roles
        return yield* service.list()
      })

      const result = await Effect.runPromise(
        program.pipe(Effect.provide(RolesMockLayer(roles)))
      )

      expect(Array.isArray(result)).toBe(true)
      expect(result.length).toBe(1)
    })
  })

  describe('listByOrg method', () => {
    test('should return roles for specific organization', async () => {
      // Given: mock service with roles from multiple orgs
      const roles = [
        new Role({
          id: 'role-1',
          org_id: 'org-1',
          name: 'admin',
          display_name: 'Administrator',
        }),
        new Role({
          id: 'role-2',
          org_id: 'org-2',
          name: 'user',
          display_name: 'User',
        }),
        new Role({
          id: 'role-3',
          org_id: 'org-1',
          name: 'manager',
          display_name: 'Manager',
        }),
      ]

      const program = Effect.gen(function* () {
        const service = yield* Roles
        return yield* service.listByOrg('org-1')
      })

      const result = await Effect.runPromise(
        program.pipe(Effect.provide(RolesMockLayer(roles)))
      )

      expect(result.length).toBe(2)
      expect(result[0].id).toBe('role-1')
      expect(result[1].id).toBe('role-3')
      expect(result[0]).toBeInstanceOf(DashboardRole)
    })

    test('should return empty array when no roles for org', async () => {
      // Given: mock service with roles from different org
      const roles = [
        new Role({
          id: 'role-1',
          org_id: 'org-1',
          name: 'admin',
          display_name: 'Administrator',
        }),
      ]

      const program = Effect.gen(function* () {
        const service = yield* Roles
        return yield* service.listByOrg('org-2')
      })

      const result = await Effect.runPromise(
        program.pipe(Effect.provide(RolesMockLayer(roles)))
      )

      expect(result.length).toBe(0)
    })

    test('should return DashboardRole instances', async () => {
      // Given: mock service with roles
      const roles = [
        new Role({
          id: 'role-1',
          org_id: 'org-1',
          name: 'admin',
          display_name: 'Administrator',
        }),
      ]

      const program = Effect.gen(function* () {
        const service = yield* Roles
        return yield* service.listByOrg('org-1')
      })

      const result = await Effect.runPromise(
        program.pipe(Effect.provide(RolesMockLayer(roles)))
      )

      expect(result[0]).toBeInstanceOf(DashboardRole)
      expect(result[0].id).toBe('role-1')
      expect(result[0].name).toBe('admin')
      expect(result[0].display_name).toBe('Administrator')
      // DashboardRole doesn't have org_id
      expect(result[0]).not.toHaveProperty('org_id')
    })
  })

  describe('create method', () => {
    test('should create a new role', async () => {
      // Given: mock service with no roles
      const program = Effect.gen(function* () {
        const service = yield* Roles
        yield* service.create({
          org_id: 'org-1',
          name: 'new-role',
          display_name: 'New Role',
        })
        return yield* service.list()
      })

      const result = await Effect.runPromise(
        program.pipe(Effect.provide(RolesMockLayer([])))
      )

      expect(result.length).toBe(1)
      expect(result[0].org_id).toBe('org-1')
      expect(result[0].name).toBe('new-role')
      expect(result[0].display_name).toBe('New Role')
    })

    test('should create role without display_name', async () => {
      // Given: mock service with no roles
      const program = Effect.gen(function* () {
        const service = yield* Roles
        yield* service.create({
          org_id: 'org-1',
          name: 'new-role',
        })
        return yield* service.list()
      })

      const result = await Effect.runPromise(
        program.pipe(Effect.provide(RolesMockLayer([])))
      )

      expect(result.length).toBe(1)
      expect(result[0].name).toBe('new-role')
      expect(result[0].display_name).toBeNull()
    })

    test('should generate unique id for new role', async () => {
      // Given: mock service with no roles
      const program = Effect.gen(function* () {
        const service = yield* Roles
        yield* service.create({
          org_id: 'org-1',
          name: 'role-1',
        })
        yield* service.create({
          org_id: 'org-1',
          name: 'role-2',
        })
        return yield* service.list()
      })

      const result = await Effect.runPromise(
        program.pipe(Effect.provide(RolesMockLayer([])))
      )

      expect(result.length).toBe(2)
      expect(result[0].id).not.toBe(result[1].id)
    })
  })

  describe('update method', () => {
    test('should update role name', async () => {
      // Given: mock service with existing role
      const roles = [
        new Role({
          id: 'role-1',
          org_id: 'org-1',
          name: 'old-name',
          display_name: 'Old Name',
        }),
      ]

      const program = Effect.gen(function* () {
        const service = yield* Roles
        yield* service.update({
          id: 'role-1',
          name: 'new-name',
        })
        return yield* service.list()
      })

      const result = await Effect.runPromise(
        program.pipe(Effect.provide(RolesMockLayer(roles)))
      )

      expect(result[0].name).toBe('new-name')
      expect(result[0].display_name).toBe('Old Name') // Unchanged
    })

    test('should update role display_name', async () => {
      // Given: mock service with existing role
      const roles = [
        new Role({
          id: 'role-1',
          org_id: 'org-1',
          name: 'admin',
          display_name: 'Old Display',
        }),
      ]

      const program = Effect.gen(function* () {
        const service = yield* Roles
        yield* service.update({
          id: 'role-1',
          display_name: 'New Display',
        })
        return yield* service.list()
      })

      const result = await Effect.runPromise(
        program.pipe(Effect.provide(RolesMockLayer(roles)))
      )

      expect(result[0].name).toBe('admin') // Unchanged
      expect(result[0].display_name).toBe('New Display')
    })

    test('should update both name and display_name', async () => {
      // Given: mock service with existing role
      const roles = [
        new Role({
          id: 'role-1',
          org_id: 'org-1',
          name: 'old-name',
          display_name: 'Old Display',
        }),
      ]

      const program = Effect.gen(function* () {
        const service = yield* Roles
        yield* service.update({
          id: 'role-1',
          name: 'new-name',
          display_name: 'New Display',
        })
        return yield* service.list()
      })

      const result = await Effect.runPromise(
        program.pipe(Effect.provide(RolesMockLayer(roles)))
      )

      expect(result[0].name).toBe('new-name')
      expect(result[0].display_name).toBe('New Display')
    })

    test('should fail when updating non-existent role', async () => {
      // Given: mock service with no roles
      const program = Effect.gen(function* () {
        const service = yield* Roles
        return yield* service.update({
          id: 'non-existent',
          name: 'new-name',
        })
      })

      await expect(
        Effect.runPromise(program.pipe(Effect.provide(RolesMockLayer([]))))
      ).rejects.toThrow('Role not found.')
    })

    test('should preserve org_id when updating', async () => {
      // Given: mock service with existing role
      const roles = [
        new Role({
          id: 'role-1',
          org_id: 'org-1',
          name: 'admin',
          display_name: 'Administrator',
        }),
      ]

      const program = Effect.gen(function* () {
        const service = yield* Roles
        yield* service.update({
          id: 'role-1',
          name: 'updated-name',
        })
        return yield* service.list()
      })

      const result = await Effect.runPromise(
        program.pipe(Effect.provide(RolesMockLayer(roles)))
      )

      expect(result[0].org_id).toBe('org-1') // Preserved
      expect(result[0].name).toBe('updated-name')
    })
  })

  describe('delete method', () => {
    test('should delete role by id', async () => {
      // Given: mock service with roles
      const roles = [
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

      const program = Effect.gen(function* () {
        const service = yield* Roles
        yield* service.delete('role-1')
        return yield* service.list()
      })

      const result = await Effect.runPromise(
        program.pipe(Effect.provide(RolesMockLayer(roles)))
      )

      expect(result.length).toBe(1)
      expect(result[0].id).toBe('role-2')
    })

    test('should fail when deleting non-existent role', async () => {
      // Given: mock service with roles
      const roles = [
        new Role({
          id: 'role-1',
          org_id: 'org-1',
          name: 'admin',
          display_name: 'Administrator',
        }),
      ]

      const program = Effect.gen(function* () {
        const service = yield* Roles
        return yield* service.delete('non-existent')
      })

      await expect(
        Effect.runPromise(program.pipe(Effect.provide(RolesMockLayer(roles))))
      ).rejects.toThrow('Role not found.')
    })

    test('should delete all roles when called multiple times', async () => {
      // Given: mock service with multiple roles
      const roles = [
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
          org_id: 'org-1',
          name: 'manager',
          display_name: 'Manager',
        }),
      ]

      const program = Effect.gen(function* () {
        const service = yield* Roles
        yield* service.delete('role-1')
        yield* service.delete('role-2')
        yield* service.delete('role-3')
        return yield* service.list()
      })

      const result = await Effect.runPromise(
        program.pipe(Effect.provide(RolesMockLayer(roles)))
      )

      expect(result.length).toBe(0)
    })
  })

  describe('service layer integration', () => {
    test('should be usable with Effect.provide', async () => {
      // Given RolesMockLayer
      // When I provide it to an effect
      // Then I should be able to access the service
      const program = Effect.gen(function* () {
        const service = yield* Roles
        expect(service).toBeDefined()
        expect(typeof service.list).toBe('function')
        return service
      })

      const service = await Effect.runPromise(
        program.pipe(Effect.provide(RolesMockLayer([])))
      )

      expect(service).toBeDefined()
      expect(typeof service.list).toBe('function')
    })

    test('should work with multiple service accesses', async () => {
      // Given RolesMockLayer
      // When I access the service multiple times
      // Then it should return the same instance (Layer.succeed behavior)
      const program = Effect.gen(function* () {
        const service1 = yield* Roles
        const service2 = yield* Roles
        return { service1, service2 }
      })

      const { service1, service2 } = await Effect.runPromise(
        program.pipe(Effect.provide(RolesMockLayer([])))
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
        const service = yield* Roles

        // Create
        yield* service.create({
          org_id: 'org-1',
          name: 'test-role',
          display_name: 'Test Role',
        })

        // List and verify
        const afterCreate = yield* service.list()
        expect(afterCreate.length).toBe(1)
        const createdId = afterCreate[0].id

        // Update
        yield* service.update({
          id: createdId,
          name: 'updated-role',
        })

        // List and verify update
        const afterUpdate = yield* service.list()
        expect(afterUpdate[0].name).toBe('updated-role')

        // Delete
        yield* service.delete(createdId)

        // List and verify deletion
        const afterDelete = yield* service.list()
        expect(afterDelete.length).toBe(0)

        return { success: true }
      })

      const result = await Effect.runPromise(
        program.pipe(Effect.provide(RolesMockLayer([])))
      )

      expect(result.success).toBe(true)
    })

    test('should filter by org correctly in listByOrg', async () => {
      // Given: mock service with roles from multiple orgs
      const roles = [
        new Role({
          id: 'role-1',
          org_id: 'org-1',
          name: 'admin',
          display_name: 'Admin',
        }),
        new Role({
          id: 'role-2',
          org_id: 'org-2',
          name: 'user',
          display_name: 'User',
        }),
        new Role({
          id: 'role-3',
          org_id: 'org-1',
          name: 'manager',
          display_name: 'Manager',
        }),
      ]

      const program = Effect.gen(function* () {
        const service = yield* Roles
        const org1Roles = yield* service.listByOrg('org-1')
        const org2Roles = yield* service.listByOrg('org-2')
        return { org1Roles, org2Roles }
      })

      const { org1Roles, org2Roles } = await Effect.runPromise(
        program.pipe(Effect.provide(RolesMockLayer(roles)))
      )

      expect(org1Roles.length).toBe(2)
      expect(org2Roles.length).toBe(1)
      expect(org1Roles[0].id).toBe('role-1')
      expect(org1Roles[1].id).toBe('role-3')
      expect(org2Roles[0].id).toBe('role-2')
    })
  })
})
