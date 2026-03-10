/**
 * Permissions service – backend operations for the permissions page.
 *
 * Provides getData (assignments + permissions list), add, and delete.
 * All methods return Effect<A, Error, never>; Live uses HttpClient.
 */

import { Context, type Effect } from 'effect'
import type { Assignment } from '../domain/Assignment'

export interface PermissionsData {
  readonly assignments: readonly Assignment[]
  readonly permissions: readonly string[]
}

export interface PermissionsService {
  /** Fetch assignments and available permissions. */
  readonly getData: () => Effect.Effect<PermissionsData, Error, never>
  /** Add a role–permission assignment. */
  readonly add: (body: {
    readonly scope: string
    readonly role_name: string
    readonly permission_key: string
  }) => Effect.Effect<void, Error, never>
  /** Remove a role–permission assignment. */
  readonly delete: (body: {
    readonly scope: string
    readonly role_name: string
    readonly permission_key: string
    readonly org_id?: string | null
  }) => Effect.Effect<void, Error, never>
}

export const Permissions = Context.GenericTag<PermissionsService>(
  'dashboard/Permissions',
)
