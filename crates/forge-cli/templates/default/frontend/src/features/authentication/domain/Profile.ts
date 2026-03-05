import { Data } from 'effect'

/**
 * Profile: one org + role pair the user can switch to.
 *
 * @remarks
 * Used for profile-select and as the current scope. `role_id` is optional
 * for backwards compatibility; `role` is the role name (e.g. "owner", "viewer").
 */
export class Profile extends Data.TaggedClass('Profile')<{
  readonly org_id: string
  readonly org_name: string
  readonly role_id?: string
  readonly role: string
}> {}
