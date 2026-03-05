/**
 * Mock Dashboard service for tests.
 */

import { Effect, Layer } from 'effect'
import { Dashboard } from './Dashboard'
import type { DashboardService } from './Dashboard'
import { Organization } from '../domain/Organization'
import { DashboardRole } from '../domain/DashboardRole'

/**
 * Creates a mock Dashboard service. Optionally pass initial organizations
 * and a map of orgId -> roles for getRolesByOrg.
 */
export function createDashboardMock(
  initialOrgs: readonly Organization[] = [],
  rolesByOrg: Record<string, readonly DashboardRole[]> = {},
): DashboardService {
  const organizations: Organization[] = initialOrgs.map((o) =>
    o instanceof Organization ? o : new Organization(o),
  )

  return {
    getOrganizations: () =>
      Effect.succeed([...organizations] as readonly Organization[]),

    getRolesByOrg: (orgId: string) =>
      Effect.sync(() => {
        const roles = rolesByOrg[orgId] ?? []
        return roles.map((r) =>
          r instanceof DashboardRole ? r : new DashboardRole(r),
        ) as readonly DashboardRole[]
      }),
  }
}

export const DashboardMockLayer = (
  initialOrgs: readonly Organization[] = [],
  rolesByOrg: Record<string, readonly DashboardRole[]> = {},
) => Layer.succeed(Dashboard, createDashboardMock(initialOrgs, rolesByOrg))
