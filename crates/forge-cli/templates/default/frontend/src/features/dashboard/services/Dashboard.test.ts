/**
 * BDD tests for Dashboard service
 * Tests verify service interface, types, and both methods (getOrganizations, getRolesByOrg)
 */
import { describe, test, expect } from 'bun:test'
import { Effect } from 'effect'

import { Dashboard, type DashboardService } from './Dashboard'
import { Organization } from '../domain/Organization'
import { DashboardRole } from '../domain/DashboardRole'
import { DashboardMockLayer } from './DashboardMock'

describe('Dashboard service', () => {
  describe('service tag', () => {
    test('should export Dashboard tag', () => {
      // Given the Dashboard module
      // When I check the Dashboard export
      // Then it should be a Context tag
      expect(Dashboard).toBeDefined()
      expect(typeof Dashboard).toBe('object')
    })
  })

  describe('service interface', () => {
    test('should export DashboardService interface', () => {
      // Given the Dashboard module
      // When I use DashboardService type
      // Then it should be usable
      const service: DashboardService = {
        getOrganizations: () => Effect.succeed([]),
        getRolesByOrg: () => Effect.succeed([]),
      }
      expect(service).toBeDefined()
      expect(typeof service.getOrganizations).toBe('function')
      expect(typeof service.getRolesByOrg).toBe('function')
    })

    test('should have getOrganizations method returning Effect<Organization[]>', async () => {
      // Given a DashboardService implementation
      // When I call getOrganizations
      // Then it should return Effect<readonly Organization[], Error, never>
      const service: DashboardService = {
        getOrganizations: () => Effect.succeed([]),
        getRolesByOrg: () => Effect.succeed([]),
      }

      const result = await Effect.runPromise(service.getOrganizations())
      expect(Array.isArray(result)).toBe(true)
    })

    test('should have getRolesByOrg method accepting orgId', async () => {
      // Given a DashboardService implementation
      // When I call getRolesByOrg with orgId
      // Then it should return Effect<readonly DashboardRole[], Error, never>
      const service: DashboardService = {
        getOrganizations: () => Effect.succeed([]),
        getRolesByOrg: () => Effect.succeed([]),
      }

      const result = await Effect.runPromise(service.getRolesByOrg('org-1'))
      expect(Array.isArray(result)).toBe(true)
    })
  })

  describe('getOrganizations method', () => {
    test('should return all organizations', async () => {
      // Given: mock service with multiple organizations
      const organizations = [
        new Organization({
          id: 'org-1',
          name: 'Organization 1',
          slug: 'org-1',
        }),
        new Organization({
          id: 'org-2',
          name: 'Organization 2',
          slug: 'org-2',
        }),
      ]

      const program = Effect.gen(function* () {
        const service = yield* Dashboard
        return yield* service.getOrganizations()
      })

      const result = await Effect.runPromise(
        program.pipe(Effect.provide(DashboardMockLayer(organizations))),
      )

      expect(result.length).toBe(2)
      expect(result[0].id).toBe('org-1')
      expect(result[1].id).toBe('org-2')
    })

    test('should return empty array when no organizations', async () => {
      // Given: mock service with no organizations
      const program = Effect.gen(function* () {
        const service = yield* Dashboard
        return yield* service.getOrganizations()
      })

      const result = await Effect.runPromise(
        program.pipe(Effect.provide(DashboardMockLayer([]))),
      )

      expect(result.length).toBe(0)
    })

    test('should return readonly array', async () => {
      // Given: mock service with organizations
      const organizations = [
        new Organization({
          id: 'org-1',
          name: 'Organization 1',
          slug: 'org-1',
        }),
      ]

      const program = Effect.gen(function* () {
        const service = yield* Dashboard
        return yield* service.getOrganizations()
      })

      const result = await Effect.runPromise(
        program.pipe(Effect.provide(DashboardMockLayer(organizations))),
      )

      expect(Array.isArray(result)).toBe(true)
      expect(result.length).toBe(1)
    })

    test('should return organizations with all properties', async () => {
      // Given: mock service with organization having all properties
      const organizations = [
        new Organization({
          id: 'org-1',
          name: 'Organization 1',
          slug: 'org-1',
          created_at: '2024-01-01T00:00:00Z',
          updated_at: '2024-01-02T00:00:00Z',
        }),
      ]

      const program = Effect.gen(function* () {
        const service = yield* Dashboard
        return yield* service.getOrganizations()
      })

      const result = await Effect.runPromise(
        program.pipe(Effect.provide(DashboardMockLayer(organizations))),
      )

      expect(result[0].id).toBe('org-1')
      expect(result[0].name).toBe('Organization 1')
      expect(result[0].slug).toBe('org-1')
      expect(result[0].created_at).toBe('2024-01-01T00:00:00Z')
      expect(result[0].updated_at).toBe('2024-01-02T00:00:00Z')
    })

    test('should return organizations without optional timestamps', async () => {
      // Given: mock service with organization without timestamps
      const organizations = [
        new Organization({
          id: 'org-1',
          name: 'Organization 1',
          slug: 'org-1',
        }),
      ]

      const program = Effect.gen(function* () {
        const service = yield* Dashboard
        return yield* service.getOrganizations()
      })

      const result = await Effect.runPromise(
        program.pipe(Effect.provide(DashboardMockLayer(organizations))),
      )

      expect(result[0].id).toBe('org-1')
      expect(result[0].name).toBe('Organization 1')
      expect(result[0].slug).toBe('org-1')
    })
  })

  describe('getRolesByOrg method', () => {
    test('should return roles for specific organization', async () => {
      // Given: mock service with roles mapped by org
      const rolesByOrg = {
        'org-1': [
          new DashboardRole({
            id: 'role-1',
            name: 'admin',
            display_name: 'Administrator',
          }),
          new DashboardRole({
            id: 'role-2',
            name: 'user',
            display_name: 'User',
          }),
        ],
        'org-2': [
          new DashboardRole({
            id: 'role-3',
            name: 'manager',
            display_name: 'Manager',
          }),
        ],
      }

      const program = Effect.gen(function* () {
        const service = yield* Dashboard
        return yield* service.getRolesByOrg('org-1')
      })

      const result = await Effect.runPromise(
        program.pipe(Effect.provide(DashboardMockLayer([], rolesByOrg))),
      )

      expect(result.length).toBe(2)
      expect(result[0].id).toBe('role-1')
      expect(result[1].id).toBe('role-2')
      expect(result[0]).toBeInstanceOf(DashboardRole)
    })

    test('should return empty array when no roles for org', async () => {
      // Given: mock service with roles for different org
      const rolesByOrg = {
        'org-1': [
          new DashboardRole({
            id: 'role-1',
            name: 'admin',
            display_name: 'Administrator',
          }),
        ],
      }

      const program = Effect.gen(function* () {
        const service = yield* Dashboard
        return yield* service.getRolesByOrg('org-2')
      })

      const result = await Effect.runPromise(
        program.pipe(Effect.provide(DashboardMockLayer([], rolesByOrg))),
      )

      expect(result.length).toBe(0)
    })

    test('should return empty array when org not in map', async () => {
      // Given: mock service with empty rolesByOrg map
      const program = Effect.gen(function* () {
        const service = yield* Dashboard
        return yield* service.getRolesByOrg('org-1')
      })

      const result = await Effect.runPromise(
        program.pipe(Effect.provide(DashboardMockLayer([], {}))),
      )

      expect(result.length).toBe(0)
    })

    test('should return DashboardRole instances', async () => {
      // Given: mock service with roles
      const rolesByOrg = {
        'org-1': [
          new DashboardRole({
            id: 'role-1',
            name: 'admin',
            display_name: 'Administrator',
          }),
        ],
      }

      const program = Effect.gen(function* () {
        const service = yield* Dashboard
        return yield* service.getRolesByOrg('org-1')
      })

      const result = await Effect.runPromise(
        program.pipe(Effect.provide(DashboardMockLayer([], rolesByOrg))),
      )

      expect(result[0]).toBeInstanceOf(DashboardRole)
      expect(result[0].id).toBe('role-1')
      expect(result[0].name).toBe('admin')
      expect(result[0].display_name).toBe('Administrator')
    })

    test('should handle roles with null display_name', async () => {
      // Given: mock service with role having null display_name
      const rolesByOrg = {
        'org-1': [
          new DashboardRole({
            id: 'role-1',
            name: 'admin',
            display_name: null,
          }),
        ],
      }

      const program = Effect.gen(function* () {
        const service = yield* Dashboard
        return yield* service.getRolesByOrg('org-1')
      })

      const result = await Effect.runPromise(
        program.pipe(Effect.provide(DashboardMockLayer([], rolesByOrg))),
      )

      expect(result[0].display_name).toBeNull()
    })

    test('should convert plain objects to DashboardRole instances', async () => {
      // Given: mock service with plain role objects
      const rolesByOrg = {
        'org-1': [
          {
            id: 'role-1',
            name: 'admin',
            display_name: 'Administrator',
          } as DashboardRole,
        ],
      }

      const program = Effect.gen(function* () {
        const service = yield* Dashboard
        return yield* service.getRolesByOrg('org-1')
      })

      const result = await Effect.runPromise(
        program.pipe(Effect.provide(DashboardMockLayer([], rolesByOrg))),
      )

      expect(result[0]).toBeInstanceOf(DashboardRole)
      expect(result[0].id).toBe('role-1')
    })
  })

  describe('service layer integration', () => {
    test('should be usable with Effect.provide', async () => {
      // Given DashboardMockLayer
      // When I provide it to an effect
      // Then I should be able to access the service
      const program = Effect.gen(function* () {
        const service = yield* Dashboard
        expect(service).toBeDefined()
        expect(typeof service.getOrganizations).toBe('function')
        expect(typeof service.getRolesByOrg).toBe('function')
        return service
      })

      const service = await Effect.runPromise(
        program.pipe(Effect.provide(DashboardMockLayer([]))),
      )

      expect(service).toBeDefined()
      expect(typeof service.getOrganizations).toBe('function')
      expect(typeof service.getRolesByOrg).toBe('function')
    })

    test('should work with multiple service accesses', async () => {
      // Given DashboardMockLayer
      // When I access the service multiple times
      // Then it should return the same instance (Layer.succeed behavior)
      const program = Effect.gen(function* () {
        const service1 = yield* Dashboard
        const service2 = yield* Dashboard
        return { service1, service2 }
      })

      const { service1, service2 } = await Effect.runPromise(
        program.pipe(Effect.provide(DashboardMockLayer([]))),
      )

      // Layer.succeed returns the same instance
      expect(service1).toBe(service2)
    })
  })

  describe('integration scenarios', () => {
    test('should support getting organizations and roles together', async () => {
      // Given: mock service with organizations and roles
      const organizations = [
        new Organization({
          id: 'org-1',
          name: 'Organization 1',
          slug: 'org-1',
        }),
        new Organization({
          id: 'org-2',
          name: 'Organization 2',
          slug: 'org-2',
        }),
      ]

      const rolesByOrg = {
        'org-1': [
          new DashboardRole({
            id: 'role-1',
            name: 'admin',
            display_name: 'Administrator',
          }),
        ],
        'org-2': [
          new DashboardRole({
            id: 'role-2',
            name: 'user',
            display_name: 'User',
          }),
        ],
      }

      const program = Effect.gen(function* () {
        const service = yield* Dashboard

        // Get organizations
        const orgs = yield* service.getOrganizations()
        expect(orgs.length).toBe(2)

        // Get roles for first org
        const org1Roles = yield* service.getRolesByOrg('org-1')
        expect(org1Roles.length).toBe(1)
        expect(org1Roles[0].id).toBe('role-1')

        // Get roles for second org
        const org2Roles = yield* service.getRolesByOrg('org-2')
        expect(org2Roles.length).toBe(1)
        expect(org2Roles[0].id).toBe('role-2')

        return { success: true }
      })

      const result = await Effect.runPromise(
        program.pipe(
          Effect.provide(DashboardMockLayer(organizations, rolesByOrg)),
        ),
      )

      expect(result.success).toBe(true)
    })

    test('should handle multiple calls to getRolesByOrg', async () => {
      // Given: mock service with roles
      const rolesByOrg = {
        'org-1': [
          new DashboardRole({
            id: 'role-1',
            name: 'admin',
            display_name: 'Administrator',
          }),
          new DashboardRole({
            id: 'role-2',
            name: 'user',
            display_name: 'User',
          }),
        ],
      }

      const program = Effect.gen(function* () {
        const service = yield* Dashboard

        // Call multiple times
        const roles1 = yield* service.getRolesByOrg('org-1')
        const roles2 = yield* service.getRolesByOrg('org-1')
        const roles3 = yield* service.getRolesByOrg('org-2') // Different org

        expect(roles1.length).toBe(2)
        expect(roles2.length).toBe(2)
        expect(roles3.length).toBe(0)

        return { success: true }
      })

      const result = await Effect.runPromise(
        program.pipe(Effect.provide(DashboardMockLayer([], rolesByOrg))),
      )

      expect(result.success).toBe(true)
    })

    test('should work with empty organizations and roles', async () => {
      // Given: mock service with no data
      const program = Effect.gen(function* () {
        const service = yield* Dashboard

        const orgs = yield* service.getOrganizations()
        expect(orgs.length).toBe(0)

        const roles = yield* service.getRolesByOrg('org-1')
        expect(roles.length).toBe(0)

        return { success: true }
      })

      const result = await Effect.runPromise(
        program.pipe(Effect.provide(DashboardMockLayer([], {}))),
      )

      expect(result.success).toBe(true)
    })

    test('should maintain separate state for organizations and roles', async () => {
      // Given: mock service
      // When I call both methods
      // Then they should work independently
      const organizations = [
        new Organization({
          id: 'org-1',
          name: 'Organization 1',
          slug: 'org-1',
        }),
      ]

      const rolesByOrg = {
        'org-1': [
          new DashboardRole({
            id: 'role-1',
            name: 'admin',
            display_name: 'Administrator',
          }),
        ],
      }

      const program = Effect.gen(function* () {
        const service = yield* Dashboard

        const orgs = yield* service.getOrganizations()
        const roles = yield* service.getRolesByOrg('org-1')

        // Organizations and roles are independent
        expect(orgs.length).toBe(1)
        expect(roles.length).toBe(1)
        expect(orgs[0].id).toBe('org-1')
        expect(roles[0].id).toBe('role-1')

        return { success: true }
      })

      const result = await Effect.runPromise(
        program.pipe(
          Effect.provide(DashboardMockLayer(organizations, rolesByOrg)),
        ),
      )

      expect(result.success).toBe(true)
    })
  })

  describe('type safety', () => {
    test('should enforce DashboardService interface', () => {
      // Given the DashboardService interface
      // When I create a service implementation
      // Then it must have getOrganizations and getRolesByOrg methods
      const validService: DashboardService = {
        getOrganizations: () => Effect.succeed([]),
        getRolesByOrg: () => Effect.succeed([]),
      }

      expect(validService).toBeDefined()
      expect(typeof validService.getOrganizations).toBe('function')
      expect(typeof validService.getRolesByOrg).toBe('function')
    })

    test('should enforce return type of getOrganizations', async () => {
      // Given a DashboardService implementation
      // When I call getOrganizations
      // Then it must return Effect<readonly Organization[], Error, never>
      const service: DashboardService = {
        getOrganizations: () => Effect.succeed([]),
        getRolesByOrg: () => Effect.succeed([]),
      }

      const result = await Effect.runPromise(service.getOrganizations())
      expect(Array.isArray(result)).toBe(true)
    })

    test('should enforce return type of getRolesByOrg', async () => {
      // Given a DashboardService implementation
      // When I call getRolesByOrg
      // Then it must return Effect<readonly DashboardRole[], Error, never>
      const service: DashboardService = {
        getOrganizations: () => Effect.succeed([]),
        getRolesByOrg: () => Effect.succeed([]),
      }

      const result = await Effect.runPromise(service.getRolesByOrg('org-1'))
      expect(Array.isArray(result)).toBe(true)
    })
  })
})
