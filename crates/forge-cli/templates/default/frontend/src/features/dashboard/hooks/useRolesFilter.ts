import { useCallback, type Dispatch, type SetStateAction } from 'react'

import { useFilteredList } from '@/hooks/useFilteredList'
import type { Role } from '@/features/dashboard/domain/Role'
import type { Organization } from '@/features/dashboard/domain/Organization'

export type RolesFilterState = {
  filterName: string
  filterDisplayName: string
  filterOrg: string
}

const initialFilters: RolesFilterState = {
  filterName: '',
  filterDisplayName: '',
  filterOrg: '',
}

function getOrgName(
  role: Role,
  organizations: readonly Organization[],
): string {
  return (
    role.org_name ??
    organizations.find((o) => o.id === role.org_id)?.name ??
    role.org_id
  )
}

export function roleMatches(
  role: Role,
  filters: RolesFilterState,
  organizations: readonly Organization[],
): boolean {
  const nameMatch =
    !filters.filterName.trim() ||
    role.name.toLowerCase().includes(filters.filterName.trim().toLowerCase())
  const displayNameMatch =
    !filters.filterDisplayName.trim() ||
    (role.display_name ?? '')
      .toLowerCase()
      .includes(filters.filterDisplayName.trim().toLowerCase())
  const orgName = getOrgName(role, organizations)
  const orgMatch =
    !filters.filterOrg.trim() ||
    orgName.toLowerCase().includes(filters.filterOrg.trim().toLowerCase())
  return nameMatch && displayNameMatch && orgMatch
}

/**
 * Filter state and filtered list for the roles page. Uses the global
 * useFilteredList with role-specific predicate (name, display name, organization).
 */
export function useRolesFilter(
  roles: readonly Role[],
  organizations: readonly Organization[],
): [RolesFilterState, Dispatch<SetStateAction<RolesFilterState>>, Role[]] {
  const predicate = useCallback(
    (role: Role, filters: RolesFilterState) =>
      roleMatches(role, filters, organizations),
    [organizations],
  )
  return useFilteredList(roles, initialFilters, predicate)
}
