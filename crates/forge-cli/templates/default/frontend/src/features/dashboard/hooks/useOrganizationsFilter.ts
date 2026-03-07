import { useCallback, type Dispatch, type SetStateAction } from 'react'

import { useFilteredList } from '@/hooks/useFilteredList'
import type { Organization } from '@/features/dashboard/domain/Organization'

export type OrganizationsFilterState = {
  filterName: string
  filterSlug: string
}

const initialFilters: OrganizationsFilterState = {
  filterName: '',
  filterSlug: '',
}

function organizationMatches(
  org: Organization,
  filters: OrganizationsFilterState,
): boolean {
  const nameMatch =
    !filters.filterName.trim() ||
    org.name.toLowerCase().includes(filters.filterName.trim().toLowerCase())
  const slugMatch =
    !filters.filterSlug.trim() ||
    org.slug.toLowerCase().includes(filters.filterSlug.trim().toLowerCase())
  return nameMatch && slugMatch
}

/**
 * Filter state and filtered list for the organizations page. Uses the global
 * useFilteredList with organization-specific predicate (name/slug).
 */
export function useOrganizationsFilter(
  organizations: readonly Organization[],
): [
  OrganizationsFilterState,
  Dispatch<SetStateAction<OrganizationsFilterState>>,
  Organization[],
] {
  const predicate = useCallback(organizationMatches, [])
  return useFilteredList(organizations, initialFilters, predicate)
}
