/**
 * BDD tests for PermissionsMock
 * Tests verify the behavior of the mock Permissions implementation for testing
 */

import { describe, it, expect, beforeEach } from 'bun:test'
import { Effect, Layer } from 'effect'
import { createPermissionsMock, PermissionsMockLayer } from './PermissionsMock'
import { Permissions } from './Permissions'
import { Assignment } from '../domain/Assignment'
import type { PermissionsService, PermissionsData } from './Permissions'

describe('PermissionsMock', () => {
  describe('createPermissionsMock behavior', () => {
    it('should return a PermissionsService implementation', () => {
      // Given createPermissionsMock function
      // When I call it
      const mock = createPermissionsMock()

      // Then it should return a PermissionsService
      expect(mock).toBeDefined()
      expect(mock.getData).toBeDefined()
      expect(mock.add).toBeDefined()
      expect(mock.delete).toBeDefined()
      expect(typeof mock.getData).toBe('function')
      expect(typeof mock.add).toBe('function')
      expect(typeof mock.delete).toBe('function')
    })

    it('should initialize with empty data when no initial provided', async () => {
      // Given createPermissionsMock function
      // When I call it without initial data
      const mock = createPermissionsMock()
      const layer = Layer.succeed(Permissions, mock)

      // Then getData should return empty arrays
      const program = Effect.gen(function* () {
        const permissions = yield* Permissions
        return yield* permissions.getData()
      })

      const result = await Effect.runPromise(
        program.pipe(Effect.provide(layer)),
      )
      expect(result.assignments).toEqual([])
      expect(result.permissions).toEqual([])
    })

    it('should initialize with provided assignments', async () => {
      // Given initial assignments
      const initialAssignments = [
        new Assignment({
          scope: 'global',
          role_name: 'admin',
          permission_key: 'read',
        }),
        new Assignment({
          scope: 'org',
          role_name: 'user',
          permission_key: 'write',
          org_id: 'org-123',
        }),
      ]

      // When I create mock with initial assignments
      const mock = createPermissionsMock({ assignments: initialAssignments })
      const layer = Layer.succeed(Permissions, mock)

      // Then getData should return the initial assignments
      const program = Effect.gen(function* () {
        const permissions = yield* Permissions
        return yield* permissions.getData()
      })

      const result = await Effect.runPromise(
        program.pipe(Effect.provide(layer)),
      )
      expect(result.assignments).toHaveLength(2)
      expect(result.assignments[0]).toEqual(initialAssignments[0])
      expect(result.assignments[1]).toEqual(initialAssignments[1])
    })

    it('should initialize with provided permissions', async () => {
      // Given initial permissions
      const initialPermissions = ['read', 'write', 'delete']

      // When I create mock with initial permissions
      const mock = createPermissionsMock({ permissions: initialPermissions })
      const layer = Layer.succeed(Permissions, mock)

      // Then getData should return the initial permissions
      const program = Effect.gen(function* () {
        const permissions = yield* Permissions
        return yield* permissions.getData()
      })

      const result = await Effect.runPromise(
        program.pipe(Effect.provide(layer)),
      )
      expect(result.permissions).toEqual(initialPermissions)
    })

    it('should convert plain objects to Assignment instances', async () => {
      // Given plain object assignments
      const plainAssignments = [
        {
          scope: 'global',
          role_name: 'admin',
          permission_key: 'read',
        },
        {
          scope: 'org',
          role_name: 'user',
          permission_key: 'write',
          org_id: 'org-123',
        },
      ]

      // When I create mock with plain objects
      const mock = createPermissionsMock({
        assignments: plainAssignments as any,
      })
      const layer = Layer.succeed(Permissions, mock)

      // Then getData should return Assignment instances
      const program = Effect.gen(function* () {
        const permissions = yield* Permissions
        return yield* permissions.getData()
      })

      const result = await Effect.runPromise(
        program.pipe(Effect.provide(layer)),
      )
      expect(result.assignments).toHaveLength(2)
      expect(result.assignments[0]).toBeInstanceOf(Assignment)
      expect(result.assignments[1]).toBeInstanceOf(Assignment)
      expect(result.assignments[0].scope).toBe('global')
      expect(result.assignments[1].org_id).toBe('org-123')
    })
  })

  describe('getData behavior', () => {
    let mock: PermissionsService
    let layer: Layer.Layer<Permissions>

    beforeEach(() => {
      mock = createPermissionsMock()
      layer = Layer.succeed(Permissions, mock)
    })

    it('should return empty data when no assignments or permissions', async () => {
      // Given a mock with no initial data
      // When I call getData
      const program = Effect.gen(function* () {
        const permissions = yield* Permissions
        return yield* permissions.getData()
      })

      // Then it should return empty arrays
      const result = await Effect.runPromise(
        program.pipe(Effect.provide(layer)),
      )
      expect(result.assignments).toEqual([])
      expect(result.permissions).toEqual([])
    })

    it('should return copies of assignments and permissions', async () => {
      // Given a mock with initial data
      const initialAssignments = [
        new Assignment({
          scope: 'global',
          role_name: 'admin',
          permission_key: 'read',
        }),
      ]
      const initialPermissions = ['read', 'write']

      const mockWithData = createPermissionsMock({
        assignments: initialAssignments,
        permissions: initialPermissions,
      })
      const layerWithData = Layer.succeed(Permissions, mockWithData)

      // When I call getData multiple times
      const program = Effect.gen(function* () {
        const permissions = yield* Permissions
        return yield* permissions.getData()
      })

      const result1 = await Effect.runPromise(
        program.pipe(Effect.provide(layerWithData)),
      )
      const result2 = await Effect.runPromise(
        program.pipe(Effect.provide(layerWithData)),
      )

      // Then results should be separate arrays (copies)
      expect(result1.assignments).not.toBe(result2.assignments)
      expect(result1.permissions).not.toBe(result2.permissions)
      expect(result1.assignments).toEqual(result2.assignments)
      expect(result1.permissions).toEqual(result2.permissions)
    })

    it('should return PermissionsData structure', async () => {
      // Given a mock
      // When I call getData
      const program = Effect.gen(function* () {
        const permissions = yield* Permissions
        return yield* permissions.getData()
      })

      // Then it should return PermissionsData structure
      const result = await Effect.runPromise(
        program.pipe(Effect.provide(layer)),
      )
      expect(result).toHaveProperty('assignments')
      expect(result).toHaveProperty('permissions')
      expect(Array.isArray(result.assignments)).toBe(true)
      expect(Array.isArray(result.permissions)).toBe(true)
    })
  })

  describe('add behavior', () => {
    let mock: PermissionsService
    let layer: Layer.Layer<Permissions>

    beforeEach(() => {
      mock = createPermissionsMock()
      layer = Layer.succeed(Permissions, mock)
    })

    it('should add a new assignment', async () => {
      // Given a mock with no assignments
      // When I add an assignment
      const program = Effect.gen(function* () {
        const permissions = yield* Permissions
        yield* permissions.add({
          scope: 'global',
          role_name: 'admin',
          permission_key: 'read',
        })
        return yield* permissions.getData()
      })

      // Then the assignment should be added
      const result = await Effect.runPromise(
        program.pipe(Effect.provide(layer)),
      )
      expect(result.assignments).toHaveLength(1)
      expect(result.assignments[0].scope).toBe('global')
      expect(result.assignments[0].role_name).toBe('admin')
      expect(result.assignments[0].permission_key).toBe('read')
      expect(result.assignments[0]).toBeInstanceOf(Assignment)
    })

    it('should create Assignment instance from body', async () => {
      // Given a mock
      // When I add an assignment
      const program = Effect.gen(function* () {
        const permissions = yield* Permissions
        yield* permissions.add({
          scope: 'org',
          role_name: 'user',
          permission_key: 'write',
        })
        return yield* permissions.getData()
      })

      // Then it should create an Assignment instance
      const result = await Effect.runPromise(
        program.pipe(Effect.provide(layer)),
      )
      expect(result.assignments[0]).toBeInstanceOf(Assignment)
    })

    it('should accumulate multiple assignments', async () => {
      // Given a mock
      // When I add multiple assignments
      const program = Effect.gen(function* () {
        const permissions = yield* Permissions
        yield* permissions.add({
          scope: 'global',
          role_name: 'admin',
          permission_key: 'read',
        })
        yield* permissions.add({
          scope: 'org',
          role_name: 'user',
          permission_key: 'write',
        })
        return yield* permissions.getData()
      })

      // Then all assignments should be present
      const result = await Effect.runPromise(
        program.pipe(Effect.provide(layer)),
      )
      expect(result.assignments).toHaveLength(2)
    })

    it('should return void Effect', async () => {
      // Given a mock
      // When I add an assignment
      const program = Effect.gen(function* () {
        const permissions = yield* Permissions
        return yield* permissions.add({
          scope: 'global',
          role_name: 'admin',
          permission_key: 'read',
        })
      })

      // Then it should complete successfully
      await expect(
        Effect.runPromise(program.pipe(Effect.provide(layer))),
      ).resolves.toBeUndefined()
    })
  })

  describe('delete behavior', () => {
    let mock: PermissionsService
    let layer: Layer.Layer<Permissions>

    beforeEach(() => {
      mock = createPermissionsMock({
        assignments: [
          new Assignment({
            scope: 'global',
            role_name: 'admin',
            permission_key: 'read',
          }),
          new Assignment({
            scope: 'org',
            role_name: 'user',
            permission_key: 'write',
            org_id: 'org-123',
          }),
        ],
      })
      layer = Layer.succeed(Permissions, mock)
    })

    it('should delete an existing assignment', async () => {
      // Given a mock with assignments
      // When I delete an assignment
      const program = Effect.gen(function* () {
        const permissions = yield* Permissions
        yield* permissions.delete({
          scope: 'global',
          role_name: 'admin',
          permission_key: 'read',
        })
        return yield* permissions.getData()
      })

      // Then the assignment should be removed
      const result = await Effect.runPromise(
        program.pipe(Effect.provide(layer)),
      )
      expect(result.assignments).toHaveLength(1)
      expect(result.assignments[0].scope).toBe('org')
    })

    it('should delete assignment matching org_id', async () => {
      // Given a mock with assignments including org_id
      // When I delete an assignment with org_id
      const program = Effect.gen(function* () {
        const permissions = yield* Permissions
        yield* permissions.delete({
          scope: 'org',
          role_name: 'user',
          permission_key: 'write',
          org_id: 'org-123',
        })
        return yield* permissions.getData()
      })

      // Then the assignment should be removed
      const result = await Effect.runPromise(
        program.pipe(Effect.provide(layer)),
      )
      expect(result.assignments).toHaveLength(1)
      expect(result.assignments[0].scope).toBe('global')
    })

    it('should handle null org_id matching', async () => {
      // Given a mock with assignment without org_id
      const mockWithNull = createPermissionsMock({
        assignments: [
          new Assignment({
            scope: 'global',
            role_name: 'admin',
            permission_key: 'read',
            org_id: null,
          }),
        ],
      })
      const layerWithNull = Layer.succeed(Permissions, mockWithNull)

      // When I delete with null org_id
      const program = Effect.gen(function* () {
        const permissions = yield* Permissions
        yield* permissions.delete({
          scope: 'global',
          role_name: 'admin',
          permission_key: 'read',
          org_id: null,
        })
        return yield* permissions.getData()
      })

      // Then the assignment should be removed
      const result = await Effect.runPromise(
        program.pipe(Effect.provide(layerWithNull)),
      )
      expect(result.assignments).toHaveLength(0)
    })

    it('should handle undefined org_id matching', async () => {
      // Given a mock with assignment without org_id
      const mockWithoutOrgId = createPermissionsMock({
        assignments: [
          new Assignment({
            scope: 'global',
            role_name: 'admin',
            permission_key: 'read',
          }),
        ],
      })
      const layerWithoutOrgId = Layer.succeed(Permissions, mockWithoutOrgId)

      // When I delete without org_id
      const program = Effect.gen(function* () {
        const permissions = yield* Permissions
        yield* permissions.delete({
          scope: 'global',
          role_name: 'admin',
          permission_key: 'read',
        })
        return yield* permissions.getData()
      })

      // Then the assignment should be removed
      const result = await Effect.runPromise(
        program.pipe(Effect.provide(layerWithoutOrgId)),
      )
      expect(result.assignments).toHaveLength(0)
    })

    it('should fail when assignment not found', async () => {
      // Given a mock with assignments
      // When I delete a non-existent assignment
      const program = Effect.gen(function* () {
        const permissions = yield* Permissions
        return yield* permissions.delete({
          scope: 'nonexistent',
          role_name: 'admin',
          permission_key: 'read',
        })
      })

      // Then it should fail with error
      await expect(
        Effect.runPromise(program.pipe(Effect.provide(layer))),
      ).rejects.toThrow('Assignment not found')
    })

    it('should return void Effect on success', async () => {
      // Given a mock with assignments
      // When I delete an existing assignment
      const program = Effect.gen(function* () {
        const permissions = yield* Permissions
        return yield* permissions.delete({
          scope: 'global',
          role_name: 'admin',
          permission_key: 'read',
        })
      })

      // Then it should complete successfully
      await expect(
        Effect.runPromise(program.pipe(Effect.provide(layer))),
      ).resolves.toBeUndefined()
    })
  })

  describe('PermissionsMockLayer behavior', () => {
    it('should provide a valid Permissions service', async () => {
      // Given PermissionsMockLayer
      // When I use it in an Effect program
      const program = Effect.gen(function* () {
        const permissions = yield* Permissions
        return permissions
      })

      // Then it should provide a valid Permissions service
      const result = await Effect.runPromise(
        program.pipe(Effect.provide(PermissionsMockLayer())),
      )
      expect(result).toBeDefined()
      expect(result.getData).toBeDefined()
      expect(result.add).toBeDefined()
      expect(result.delete).toBeDefined()
    })

    it('should accept initial data', async () => {
      // Given PermissionsMockLayer with initial data
      const initialAssignments = [
        new Assignment({
          scope: 'global',
          role_name: 'admin',
          permission_key: 'read',
        }),
      ]
      const initialPermissions = ['read', 'write']

      // When I use it in an Effect program
      const program = Effect.gen(function* () {
        const permissions = yield* Permissions
        return yield* permissions.getData()
      })

      // Then it should return the initial data
      const result = await Effect.runPromise(
        program.pipe(
          Effect.provide(
            PermissionsMockLayer({
              assignments: initialAssignments,
              permissions: initialPermissions,
            }),
          ),
        ),
      )
      expect(result.assignments).toHaveLength(1)
      expect(result.permissions).toEqual(initialPermissions)
    })

    it('should allow calling methods', async () => {
      // Given PermissionsMockLayer
      // When I call methods on the service
      const program = Effect.gen(function* () {
        const permissions = yield* Permissions
        const data = yield* permissions.getData()
        yield* permissions.add({
          scope: 'global',
          role_name: 'admin',
          permission_key: 'read',
        })
        const updatedData = yield* permissions.getData()
        return { initial: data, updated: updatedData }
      })

      // Then it should execute successfully
      const result = await Effect.runPromise(
        program.pipe(Effect.provide(PermissionsMockLayer())),
      )
      expect(result.initial.assignments).toHaveLength(0)
      expect(result.updated.assignments).toHaveLength(1)
    })
  })

  describe('integration behavior', () => {
    it('should support full CRUD flow', async () => {
      // Given a mock Permissions service
      const mock = createPermissionsMock()
      const layer = Layer.succeed(Permissions, mock)

      // When I perform a full CRUD flow
      const program = Effect.gen(function* () {
        const permissions = yield* Permissions

        // Create
        yield* permissions.add({
          scope: 'global',
          role_name: 'admin',
          permission_key: 'read',
        })
        yield* permissions.add({
          scope: 'org',
          role_name: 'user',
          permission_key: 'write',
        })

        // Read
        const data1 = yield* permissions.getData()
        expect(data1.assignments).toHaveLength(2)

        // Delete
        yield* permissions.delete({
          scope: 'global',
          role_name: 'admin',
          permission_key: 'read',
        })

        // Read again
        const data2 = yield* permissions.getData()
        return data2
      })

      // Then it should complete successfully
      const result = await Effect.runPromise(
        program.pipe(Effect.provide(layer)),
      )
      expect(result.assignments).toHaveLength(1)
      expect(result.assignments[0].scope).toBe('org')
    })

    it('should maintain separate state per mock instance', async () => {
      // Given two separate mock instances
      const mock1 = createPermissionsMock({
        assignments: [
          new Assignment({
            scope: 'global',
            role_name: 'admin',
            permission_key: 'read',
          }),
        ],
      })
      const mock2 = createPermissionsMock({
        assignments: [
          new Assignment({
            scope: 'org',
            role_name: 'user',
            permission_key: 'write',
          }),
        ],
      })

      const layer1 = Layer.succeed(Permissions, mock1)
      const layer2 = Layer.succeed(Permissions, mock2)

      // When I get data from both
      const program = Effect.gen(function* () {
        const permissions = yield* Permissions
        return yield* permissions.getData()
      })

      const result1 = await Effect.runPromise(
        program.pipe(Effect.provide(layer1)),
      )
      const result2 = await Effect.runPromise(
        program.pipe(Effect.provide(layer2)),
      )

      // Then they should have separate state
      expect(result1.assignments).toHaveLength(1)
      expect(result1.assignments[0].scope).toBe('global')
      expect(result2.assignments).toHaveLength(1)
      expect(result2.assignments[0].scope).toBe('org')
    })
  })
})
