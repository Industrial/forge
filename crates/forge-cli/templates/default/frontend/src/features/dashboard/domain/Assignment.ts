import { Data } from 'effect'

/**
 * Role–permission assignment. Used by the Permissions service.
 */
export class Assignment extends Data.TaggedClass('Assignment')<{
  readonly scope: string
  readonly role_name: string
  readonly permission_key: string
  readonly org_id?: string | null
}> {}
