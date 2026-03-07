/**
 * BDD tests for DashboardMock - tests the mock implementation itself.
 * Tests use Effect's Layer system for dependency injection (no vi.mock()).
 */
import { describe, test, expect } from 'bun:test'
import { Effect, Layer } from 'effect'

import { Dashboard } from './Dashboard'
import { createDashboardMock, DashboardMockLayer } from './DashboardMock'
import { Organization } from '../domain/Organization'
import { DashboardRole } from '../domain/DashboardRole'

describe('DashboardMock', () => {
  describe('createDashboardMock behavior', () => {
    test('should create mock service with empty defaults', async () => {
      // Given: createDashboardMock called without arguments
      const mock = createDashboardMock()
      const mockLayer = Layer.succeed(Dashboard, mock)

      // When: getting organizations and roles
      const dashboard = await Effect.runPromise(
        Effect.gen(function* () {
          return yield* Dashboard
        }).pipe(Effect.provide(mockLayer)),
      )

      const orgs = await Effect.runPromise(
        dashboard.getOrganizations().pipe(Effect.provide(mockLayer)),
      )

      const roles = await Effect.runPromise(
        dashboard.getRolesByOrg('org-1').pipe(Effect.provide(mockLayer)),
      )

      // Then: should return empty arrays
      expect(orgs).toHaveLength(0)
      expect(roles).toHaveLength(0)
    })

    test('should create mock service with initial organizations', async () => {
      // Given: createDashboardMock called with organizations
      const organizations = [
        new Organization({
          id: 'org-1',
          name: 'Acme Corp',
          slug: 'acme-corp',
          created_at: '2024-01-01T00:00:00Z',
          updated_at: '2024-01-02T00:00:00Z',
        }),
        new Organization({
          id: 'org-2',
          name: 'Tech Inc',
          slug: 'tech-inc',
        }),
      ]

      const mock = createDashboardMock(organizations)
      const mockLayer = Layer.succeed(Dashboard, mock)

      // When: getting organizations
      const dashboard = await Effect.runPromise(
        Effect.gen(function* () {
          return yield* Dashboard
        }).pipe(Effect.provide(mockLayer)),
      )

      const result = await Effect.runPromise(
        dashboard.getOrganizations().pipe(Effect.provide(mockLayer)),
      )

      // Then: should return initial organizations
      expect(result).toHaveLength(2)
      expect(result[0]).toBeInstanceOf(Organization)
      expect(result[0].id).toBe('org-1')
      expect(result[0].name).toBe('Acme Corp')
      expect(result[1].id).toBe('org-2')
    })

    test('should normalize plain objects to Organization instances', async () => {
      // Given: createDashboardMock called with plain objects
      const plainOrgs = [
        {
          id: 'org-1',
          name: 'Acme Corp',
          slug: 'acme-corp',
        },
      ]

      const mock = createDashboardMock(plainOrgs as any)
      const mockLayer = Layer.succeed(Dashboard, mock)

      // When: getting organizations
      const dashboard = await Effect.runPromise(
        Effect.gen(function* () {
          return yield* Dashboard
        }).pipe(Effect.provide(mockLayer)),
      )

      const result = await Effect.runPromise(
        dashboard.getOrganizations().pipe(Effect.provide(mockLayer)),
      )

      // Then: should normalize to Organization instances
      expect(result).toHaveLength(1)
      expect(result[0]).toBeInstanceOf(Organization)
      expect(result[0].id).toBe('org-1')
      expect(result[0].name).toBe('Acme Corp')
    })

    test('should preserve Organization instances', async () => {
      // Given: createDashboardMock called with Organization instances
      const organizations = [
        new Organization({
          id: 'org-1',
          name: 'Acme Corp',
          slug: 'acme-corp',
        }),
      ]

      const mock = createDashboardMock(organizations)
      const mockLayer = Layer.succeed(Dashboard, mock)

      // When: getting organizations
      const dashboard = await Effect.runPromise(
        Effect.gen(function* () {
          return yield* Dashboard
        }).pipe(Effect.provide(mockLayer)),
      )

      const result = await Effect.runPromise(
        dashboard.getOrganizations().pipe(Effect.provide(mockLayer)),
      )

      // Then: should preserve instances
      expect(result[0]).toBeInstanceOf(Organization)
      expect(result[0].id).toBe('org-1')
    })

    test('should create mock service with rolesByOrg', async () => {
      // Given: createDashboardMock called with rolesByOrg
      const rolesByOrg = {
        'org-1': [
          new DashboardRole({
            id: 'role-1',
            name: 'admin',
            display_name: 'Administrator',
          }),
          new DashboardRole({
            id: 'role-2',
            name: 'member',
            display_name: null,
          }),
        ],
      }

      const mock = createDashboardMock([], rolesByOrg)
      const mockLayer = Layer.succeed(Dashboard, mock)

      // When: getting roles by org
      const dashboard = await Effect.runPromise(
        Effect.gen(function* () {
          return yield* Dashboard
        }).pipe(Effect.provide(mockLayer)),
      )

      const result = await Effect.runPromise(
        dashboard.getRolesByOrg('org-1').pipe(Effect.provide(mockLayer)),
      )

      // Then: should return roles for that org
      expect(result).toHaveLength(2)
      expect(result[0]).toBeInstanceOf(DashboardRole)
      expect(result[0].id).toBe('role-1')
      expect(result[1].id).toBe('role-2')
    })

    test('should return empty array when orgId not in rolesByOrg', async () => {
      // Given: createDashboardMock with rolesByOrg for different org
      const rolesByOrg = {
        'org-1': [
          new DashboardRole({
            id: 'role-1',
            name: 'admin',
            display_name: 'Administrator',
          }),
        ],
      }

      const mock = createDashboardMock([], rolesByOrg)
      const mockLayer = Layer.succeed(Dashboard, mock)

      // When: getting roles for non-existent org
      const dashboard = await Effect.runPromise(
        Effect.gen(function* () {
          return yield* Dashboard
        }).pipe(Effect.provide(mockLayer)),
      )

      const result = await Effect.runPromise(
        dashboard.getRolesByOrg('org-999').pipe(Effect.provide(mockLayer)),
      )

      // Then: should return empty array
      expect(result).toHaveLength(0)
    })

    test('should normalize plain objects to DashboardRole instances', async () => {
      // Given: createDashboardMock called with plain object roles
      const rolesByOrg = {
        'org-1': [
          {
            id: 'role-1',
            name: 'admin',
            display_name: 'Administrator',
          },
        ] as any,
      }

      const mock = createDashboardMock([], rolesByOrg)
      const mockLayer = Layer.succeed(Dashboard, mock)

      // When: getting roles by org
      const dashboard = await Effect.runPromise(
        Effect.gen(function* () {
          return yield* Dashboard
        }).pipe(Effect.provide(mockLayer)),
      )

      const result = await Effect.runPromise(
        dashboard.getRolesByOrg('org-1').pipe(Effect.provide(mockLayer)),
      )

      // Then: should normalize to DashboardRole instances
      expect(result).toHaveLength(1)
      expect(result[0]).toBeInstanceOf(DashboardRole)
      expect(result[0].id).toBe('role-1')
      expect(result[0].name).toBe('admin')
    })

    test('should preserve DashboardRole instances', async () => {
      // Given: createDashboardMock called with DashboardRole instances
      const rolesByOrg = {
        'org-1': [
          new DashboardRole({
            id: 'role-1',
            name: 'admin',
            display_name: 'Administrator',
          }),
        ],
      }

      const mock = createDashboardMock([], rolesByOrg)
      const mockLayer = Layer.succeed(Dashboard, mock)

      // When: getting roles by org
      const dashboard = await Effect.runPromise(
        Effect.gen(function* () {
          return yield* Dashboard
        }).pipe(Effect.provide(mockLayer)),
      )

      const result = await Effect.runPromise(
        dashboard.getRolesByOrg('org-1').pipe(Effect.provide(mockLayer)),
      )

      // Then: should preserve instances
      expect(result[0]).toBeInstanceOf(DashboardRole)
      expect(result[0].id).toBe('role-1')
    })

    test('should return copies of organizations array', async () => {
      // Given: createDashboardMock with organizations
      const organizations = [
        new Organization({
          id: 'org-1',
          name: 'Acme Corp',
          slug: 'acme-corp',
        }),
      ]

      const mock = createDashboardMock(organizations)
      const mockLayer = Layer.succeed(Dashboard, mock)

      // When: getting organizations multiple times
      const dashboard = await Effect.runPromise(
        Effect.gen(function* () {
          return yield* Dashboard
        }).pipe(Effect.provide(mockLayer)),
      )

      const result1 = await Effect.runPromise(
        dashboard.getOrganizations().pipe(Effect.provide(mockLayer)),
      )

      const result2 = await Effect.runPromise(
        dashboard.getOrganizations().pipe(Effect.provide(mockLayer)),
      )

      // Then: should return separate arrays (copies)
      expect(result1).not.toBe(result2)
      expect(result1).toEqual(result2)
    })

    test('should handle multiple organizations with different roles', async () => {
      // Given: createDashboardMock with roles for multiple orgs
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
            name: 'member',
            display_name: 'Member',
          }),
        ],
      }

      const mock = createDashboardMock([], rolesByOrg)
      const mockLayer = Layer.succeed(Dashboard, mock)

      // When: getting roles for different orgs
      const dashboard = await Effect.runPromise(
        Effect.gen(function* () {
          return yield* Dashboard
        }).pipe(Effect.provide(mockLayer)),
      )

      const result1 = await Effect.runPromise(
        dashboard.getRolesByOrg('org-1').pipe(Effect.provide(mockLayer)),
      )

      const result2 = await Effect.runPromise(
        dashboard.getRolesByOrg('org-2').pipe(Effect.provide(mockLayer)),
      )

      // Then: should return correct roles for each org
      expect(result1).toHaveLength(1)
      expect(result1[0].id).toBe('role-1')
      expect(result2).toHaveLength(1)
      expect(result2[0].id).toBe('role-2')
    })
  })

  describe('DashboardMockLayer behavior', () => {
    test('should create layer with empty defaults', async () => {
      // Given: DashboardMockLayer called without arguments
      const mockLayer = DashboardMockLayer()

      // When: getting organizations and roles
      const dashboard = await Effect.runPromise(
        Effect.gen(function* () {
          return yield* Dashboard
        }).pipe(Effect.provide(mockLayer)),
      )

      const orgs = await Effect.runPromise(
        dashboard.getOrganizations().pipe(Effect.provide(mockLayer)),
      )

      const roles = await Effect.runPromise(
        dashboard.getRolesByOrg('org-1').pipe(Effect.provide(mockLayer)),
      )

      // Then: should return empty arrays
      expect(orgs).toHaveLength(0)
      expect(roles).toHaveLength(0)
    })

    test('should create layer with initial data', async () => {
      // Given: DashboardMockLayer called with initial data
      const organizations = [
        new Organization({
          id: 'org-1',
          name: 'Acme Corp',
          slug: 'acme-corp',
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

      const mockLayer = DashboardMockLayer(organizations, rolesByOrg)

      // When: getting data
      const dashboard = await Effect.runPromise(
        Effect.gen(function* () {
          return yield* Dashboard
        }).pipe(Effect.provide(mockLayer)),
      )

      const orgs = await Effect.runPromise(
        dashboard.getOrganizations().pipe(Effect.provide(mockLayer)),
      )

      const roles = await Effect.runPromise(
        dashboard.getRolesByOrg('org-1').pipe(Effect.provide(mockLayer)),
      )

      // Then: should return initial data
      expect(orgs).toHaveLength(1)
      expect(roles).toHaveLength(1)
    })

    test('should provide Dashboard service via Layer.succeed', async () => {
      // Given: DashboardMockLayer
      const mockLayer = DashboardMockLayer()

      // When: accessing service multiple times
      const program = Effect.gen(function* () {
        const service1 = yield* Dashboard
        const service2 = yield* Dashboard
        return { service1, service2 }
      })

      const { service1, service2 } = await Effect.runPromise(
        program.pipe(Effect.provide(mockLayer)),
      )

      // Then: should return same instance (Layer.succeed behavior)
      expect(service1).toBe(service2)
      expect(typeof service1.getOrganizations).toBe('function')
      expect(typeof service2.getRolesByOrg).toBe('function')
    })

    test('should be composable with other layers', async () => {
      // Given: DashboardMockLayer and another layer
      const organizations = [
        new Organization({
          id: 'org-1',
          name: 'Acme Corp',
          slug: 'acme-corp',
        }),
      ]

      const mockLayer = DashboardMockLayer(organizations)
      const combinedLayer = Layer.mergeAll(mockLayer)

      // When: using combined layer
      const dashboard = await Effect.runPromise(
        Effect.gen(function* () {
          return yield* Dashboard
        }).pipe(Effect.provide(combinedLayer)),
      )

      const result = await Effect.runPromise(
        dashboard.getOrganizations().pipe(Effect.provide(combinedLayer)),
      )

      // Then: should work correctly
      expect(result).toHaveLength(1)
      expect(result[0].id).toBe('org-1')
    })
  })

  describe('mock isolation', () => {
    test('should maintain separate state per mock instance', async () => {
      // Given: two separate mock instances
      const orgs1 = [
        new Organization({
          id: 'org-1',
          name: 'Acme Corp',
          slug: 'acme-corp',
        }),
      ]

      const orgs2 = [
        new Organization({
          id: 'org-2',
          name: 'Tech Inc',
          slug: 'tech-inc',
        }),
      ]

      const rolesByOrg1 = {
        'org-1': [
          new DashboardRole({
            id: 'role-1',
            name: 'admin',
            display_name: 'Administrator',
          }),
        ],
      }

      const rolesByOrg2 = {
        'org-2': [
          new DashboardRole({
            id: 'role-2',
            name: 'member',
            display_name: 'Member',
          }),
        ],
      }

      const mock1 = createDashboardMock(orgs1, rolesByOrg1)
      const mock2 = createDashboardMock(orgs2, rolesByOrg2)
      const layer1 = Layer.succeed(Dashboard, mock1)
      const layer2 = Layer.succeed(Dashboard, mock2)

      // When: getting data from each mock
      const dashboard1 = await Effect.runPromise(
        Effect.gen(function* () {
          return yield* Dashboard
        }).pipe(Effect.provide(layer1)),
      )

      const dashboard2 = await Effect.runPromise(
        Effect.gen(function* () {
          return yield* Dashboard
        }).pipe(Effect.provide(layer2)),
      )

      const orgs1Result = await Effect.runPromise(
        dashboard1.getOrganizations().pipe(Effect.provide(layer1)),
      )

      const orgs2Result = await Effect.runPromise(
        dashboard2.getOrganizations().pipe(Effect.provide(layer2)),
      )

      const roles1Result = await Effect.runPromise(
        dashboard1.getRolesByOrg('org-1').pipe(Effect.provide(layer1)),
      )

      const roles2Result = await Effect.runPromise(
        dashboard2.getRolesByOrg('org-2').pipe(Effect.provide(layer2)),
      )

      // Then: each mock should have its own state
      expect(orgs1Result).toHaveLength(1)
      expect(orgs1Result[0].id).toBe('org-1')
      expect(orgs2Result).toHaveLength(1)
      expect(orgs2Result[0].id).toBe('org-2')
      expect(roles1Result).toHaveLength(1)
      expect(roles1Result[0].id).toBe('role-1')
      expect(roles2Result).toHaveLength(1)
      expect(roles2Result[0].id).toBe('role-2')
    })
  })
})
