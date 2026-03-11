import FiltersPanel from '../../../components/FiltersPanel'
import type { FilterField } from '../../../components/FiltersPanel'

export type RolesFiltersProps = {
  filterName: string
  filterDisplayName: string
  filterOrg: string
  onFilterNameChange: (value: string) => void
  onFilterDisplayNameChange: (value: string) => void
  onFilterOrgChange: (value: string) => void
}

const FIELDS: readonly FilterField[] = [
  {
    type: 'text',
    key: 'name',
    label: 'Name',
    placeholder: 'Search by role name',
    minWidth: 200,
    dataTestId: 'roles-filter-name',
  },
  {
    type: 'text',
    key: 'displayName',
    label: 'Display name',
    placeholder: 'Search by display name',
    minWidth: 200,
    dataTestId: 'roles-filter-display-name',
  },
  {
    type: 'text',
    key: 'org',
    label: 'Organization',
    placeholder: 'Search by organization',
    minWidth: 200,
    dataTestId: 'roles-filter-org',
  },
] as const

export default function RolesFilters({
  filterName,
  filterDisplayName,
  filterOrg,
  onFilterNameChange,
  onFilterDisplayNameChange,
  onFilterOrgChange,
}: RolesFiltersProps) {
  const values = {
    name: filterName,
    displayName: filterDisplayName,
    org: filterOrg,
  }
  const onChange = (key: string, value: string) => {
    if (key === 'name') {
      onFilterNameChange(value)
    } else if (key === 'displayName') {
      onFilterDisplayNameChange(value)
    } else if (key === 'org') {
      onFilterOrgChange(value)
    }
  }
  return <FiltersPanel fields={FIELDS} values={values} onChange={onChange} />
}
