import { Data } from 'effect'

/**
 * Role entity (full). Used by the Roles service for list/create/update/delete.
 */
export class Role extends Data.TaggedClass('Role')<{
  readonly id: string
  readonly org_id: string
  /** Organization display name (when returned by list API). */
  readonly org_name?: string
  readonly name: string
  readonly display_name: string | null
  readonly created_at?: string
  readonly updated_at?: string
}> {}
