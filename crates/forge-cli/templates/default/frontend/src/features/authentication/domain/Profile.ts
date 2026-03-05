import { Data } from 'effect'

/**
 * Profile: one org + role pair the user can switch to.
 *
 * @remarks
 * **Deprecated:** Prefer {@link Scope} and scope terminology. This type is kept for
 * backwards compatibility; new code should use Scope from './Scope'.
 */
export class Profile extends Data.TaggedClass('Profile')<{
  readonly org_id: string
  readonly org_name: string
  readonly role_id?: string
  readonly role: string
}> {}
