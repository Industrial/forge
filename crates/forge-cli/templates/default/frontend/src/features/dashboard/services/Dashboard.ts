/**
 * Dashboard service – shared dropdown/list data for dashboard pages.
 *
 * Provides getOrganizations (for org dropdown) and getRolesByOrg (for role
 * dropdown when creating users). All methods return Effect<A, Error, never>;
 * Live uses HttpClient.
 *
 * For full CRUD use the feature services: Organizations, Roles, Users, etc.
 */

import { Context, Effect } from 'effect'
import type { Organization } from '../domain/Organization'
import type { DashboardRole } from '../domain/DashboardRole'

export interface DashboardService {
  /** List organizations (e.g. for dropdowns). GET /api/dashboard/organizations. */
  readonly getOrganizations: () => Effect.Effect<
    readonly Organization[],
    Error,
    never
  >
  /** List roles for an org (e.g. role dropdown). GET /api/dashboard/roles?org_id=... */
  readonly getRolesByOrg: (
    orgId: string,
  ) => Effect.Effect<readonly DashboardRole[], Error, never>
}

export const Dashboard = Context.GenericTag<DashboardService>(
  'dashboard/Dashboard',
)
