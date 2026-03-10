/**
 * Roles service – backend operations for the roles page.
 *
 * Provides list (all roles), listByOrg (for dropdowns), create, update, delete.
 * All methods return Effect<A, Error, never>; Live uses HttpClient.
 */

import { Context, type Effect } from 'effect'
import type { Role } from '../domain/Role'
import type { DashboardRole } from '../domain/DashboardRole'

export interface RolesService {
  /** List all roles. GET /api/auth/roles. */
  readonly list: () => Effect.Effect<readonly Role[], Error, never>
  /** List roles for an organization (e.g. dropdown). GET /api/auth/roles?org_id=... */
  readonly listByOrg: (
    orgId: string,
  ) => Effect.Effect<readonly DashboardRole[], Error, never>
  /** Create a role. POST /api/auth/roles. */
  readonly create: (body: {
    readonly org_id: string
    readonly name: string
    readonly display_name?: string
  }) => Effect.Effect<void, Error, never>
  /** Update a role. PATCH /api/auth/roles/{id}. */
  readonly update: (body: {
    readonly id: string
    readonly name?: string
    readonly display_name?: string
  }) => Effect.Effect<void, Error, never>
  /** Delete a role. DELETE /api/auth/roles/{id}. */
  readonly delete: (id: string) => Effect.Effect<void, Error, never>
}

export const Roles = Context.GenericTag<RolesService>('dashboard/Roles')
