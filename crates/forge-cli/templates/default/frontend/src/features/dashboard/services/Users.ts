/**
 * Users service – backend operations for the users page.
 *
 * Provides list, create, update, and delete. All methods return
 * Effect<A, Error, never>; Live uses HttpClient.
 */

import { Context, Effect } from 'effect'
import type { User } from '../domain/User'

export interface UsersService {
  /** List all users. GET /api/auth/users. */
  readonly list: () => Effect.Effect<readonly User[], Error, never>
  /** Create a user. POST /api/auth/users. */
  readonly create: (body: {
    readonly email: string
    readonly password: string
    readonly org_id: string
    readonly role_ids: readonly string[]
  }) => Effect.Effect<void, Error, never>
  /** Update a user. PATCH /api/auth/users/{id}. */
  readonly update: (body: {
    readonly id: string
    readonly email?: string
    readonly is_active?: boolean
  }) => Effect.Effect<void, Error, never>
  /** Delete a user. DELETE /api/auth/users/{id}. */
  readonly delete: (id: string) => Effect.Effect<void, Error, never>
}

export const Users = Context.GenericTag<UsersService>('dashboard/Users')
