import { Data } from 'effect'
import type { AuthenticationUser } from './AuthenticationUser'
import type { Flash } from './Flash'
import type { Scope } from './Scope'

/**
 * Read-only snapshot of auth state for consumers (legacy; prefer reactive store + useAuthenticationStateReactiveStore).
 *
 * @remarks
 * No setters; all mutations go through AuthenticationStore methods. Use
 * `getState()` to read the current snapshot. When `needs_scope_select` is
 * true, the app should redirect to scope selection before the dashboard.
 */
export class AuthenticationStateSnapshot extends Data.TaggedClass(
  'AuthenticationStateSnapshot',
)<{
  readonly user: AuthenticationUser | null
  readonly scopes: readonly Scope[]
  readonly permissions: readonly string[]
  readonly flash: Flash | null
  readonly token: string | null
  readonly currentOrgId: string | null
  readonly currentRoleId: string | null
  readonly currentRoleName: string | null
  /** When true, redirect to scope selection before dashboard. */
  readonly needs_scope_select: boolean
}> {}
