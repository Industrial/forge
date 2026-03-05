import { Data } from 'effect'
import type { AuthUser } from './AuthUser'
import type { Flash } from './Flash'
import type { Profile } from './Profile'

/**
 * Read-only snapshot of auth state for consumers (e.g. React useAuthenticationState hook).
 *
 * @remarks
 * No setters; all mutations go through AuthenticationStore methods. Use
 * `getState()` to read the current snapshot. When `needs_profile_select` is
 * true, the app should redirect to profile-select before the dashboard.
 */
export class AuthStateSnapshot extends Data.TaggedClass('AuthStateSnapshot')<{
  readonly user: AuthUser | null
  readonly profiles: readonly Profile[]
  readonly permissions: readonly string[]
  readonly flash: Flash | null
  readonly token: string | null
  readonly currentOrgId: string | null
  readonly currentRoleId: string | null
  readonly currentRoleName: string | null
  /** When true, redirect to /select-profile before dashboard. */
  readonly needs_profile_select: boolean
}> {}
