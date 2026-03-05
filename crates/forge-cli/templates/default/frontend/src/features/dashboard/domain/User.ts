import { Data } from 'effect'

/** User membership in an organization with roles. */
export class UserMembership extends Data.TaggedClass('UserMembership')<{
  readonly org_id: string
  readonly org_name: string
  readonly roles: readonly string[]
}> {}

/**
 * User entity. Used by the Users service and dashboard pages.
 */
export class User extends Data.TaggedClass('User')<{
  readonly id: string
  readonly email: string
  readonly is_active: boolean
  readonly is_admin: boolean
  readonly created_at: string
  readonly memberships: readonly UserMembership[]
}> {}
