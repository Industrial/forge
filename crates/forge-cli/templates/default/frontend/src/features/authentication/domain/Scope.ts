import { Data } from 'effect'

/**
 * Scope: one org + role pair the user can switch to (session scope).
 *
 * @remarks
 * Used for scope selection and as the current scope. `role_id` is optional
 * for backwards compatibility; `role` is the role name (e.g. "owner", "viewer").
 * API still returns these as "profiles"; we map to Scope in the frontend.
 */
export class Scope extends Data.TaggedClass('Scope')<{
  readonly org_id: string
  readonly org_name: string
  readonly role_id?: string
  readonly role: string
}> {}
