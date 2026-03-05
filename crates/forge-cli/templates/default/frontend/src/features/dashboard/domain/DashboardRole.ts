import { Data } from 'effect'

/**
 * Role summary for dropdowns (e.g. org-scoped role list).
 * Used by Dashboard.getRolesByOrg and Roles.listByOrg.
 */
export class DashboardRole extends Data.TaggedClass('DashboardRole')<{
  readonly id: string
  readonly name: string
  readonly display_name: string | null
}> {}
