import { Data } from 'effect'
import type { AuthenticationUser } from './AuthenticationUser'
import type { Flash } from './Flash'
import type { Scope } from './Scope'

/**
 * Read-only snapshot of auth state for consumers (legacy; prefer reactive store + useAuthStore / useAuthStoreWithInit).
 *
 * @remarks
 * No setters; all mutations go through AuthenticationStore methods. Use
 * `getState()` to read the current snapshot. Scope selection need is
 * derived on the client from currentOrgId/currentRoleId (none => need selection).
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
}> {}
