import FiltersPanel from '@/components/FiltersPanel'
import type { FilterField } from '@/components/FiltersPanel'

const SCOPE_OPTIONS = [
  { value: '', label: 'All' },
  { value: 'org', label: 'Org' },
  { value: 'global', label: 'Global' },
] as const

const ROLE_OPTIONS = [
  { value: '', label: 'All' },
  { value: 'owner', label: 'owner' },
  { value: 'admin', label: 'admin' },
  { value: 'editor', label: 'editor' },
  { value: 'viewer', label: 'viewer' },
  { value: 'platform_admin', label: 'platform_admin' },
] as const

export type PermissionsFiltersProps = {
  filterScope: string
  filterRole: string
  filterPermission: string
  onFilterScopeChange: (value: string) => void
  onFilterRoleChange: (value: string) => void
  onFilterPermissionChange: (value: string) => void
}

const FIELDS: readonly FilterField[] = [
  {
    type: 'select',
    key: 'scope',
    label: 'Scope',
    minWidth: 120,
    options: [...SCOPE_OPTIONS],
  },
  {
    type: 'select',
    key: 'role',
    label: 'Role',
    minWidth: 140,
    options: [...ROLE_OPTIONS],
  },
  {
    type: 'text',
    key: 'permission',
    label: 'Permission',
    placeholder: 'Search by permission key',
    minWidth: 200,
    dataTestId: 'permissions-filter-input',
  },
]

export default function PermissionsFilters({
  filterScope,
  filterRole,
  filterPermission,
  onFilterScopeChange,
  onFilterRoleChange,
  onFilterPermissionChange,
}: PermissionsFiltersProps) {
  const values = {
    scope: filterScope,
    role: filterRole,
    permission: filterPermission,
  }
  const onChange = (key: string, value: string) => {
    if (key === 'scope') onFilterScopeChange(value)
    else if (key === 'role') onFilterRoleChange(value)
    else if (key === 'permission') onFilterPermissionChange(value)
  }
  return <FiltersPanel fields={FIELDS} values={values} onChange={onChange} />
}
