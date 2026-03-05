import { Data } from 'effect'

/**
 * Role entity (full). Used by the Roles service for list/create/update/delete.
 */
export class Role extends Data.TaggedClass('Role')<{
  readonly id: string
  readonly org_id: string
  readonly name: string
  readonly display_name: string | null
  readonly created_at?: string
  readonly updated_at?: string
}> {}
