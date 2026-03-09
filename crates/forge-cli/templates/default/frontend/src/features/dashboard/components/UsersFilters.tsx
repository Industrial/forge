import FilterPanelFromConfig from '@/components/FilterPanelFromConfig'
import type { FilterField } from '@/components/FilterPanelFromConfig'

export type OrgOption = { id: string; name: string }

export type UsersFiltersProps = {
  filterEmail: string
  filterOrgId: string
  filterRole: string
  filterActive: '' | 'yes' | 'no'
  filterAdmin: '' | 'yes' | 'no'
  organizations: readonly OrgOption[]
  roleOptions: readonly string[]
  onFilterEmailChange: (value: string) => void
  onFilterOrgIdChange: (value: string) => void
  onFilterRoleChange: (value: string) => void
  onFilterActiveChange: (value: '' | 'yes' | 'no') => void
  onFilterAdminChange: (value: '' | 'yes' | 'no') => void
}

const YES_NO_OPTIONS = [
  { value: '', label: 'All' },
  { value: 'yes', label: 'Yes' },
  { value: 'no', label: 'No' },
] as const

export default function UsersFilters({
  filterEmail,
  filterOrgId,
  filterRole,
  filterActive,
  filterAdmin,
  organizations,
  roleOptions,
  onFilterEmailChange,
  onFilterOrgIdChange,
  onFilterRoleChange,
  onFilterActiveChange,
  onFilterAdminChange,
}: UsersFiltersProps) {
  const values = {
    email: filterEmail,
    orgId: filterOrgId,
    role: filterRole,
    active: filterActive,
    admin: filterAdmin,
  }
  const onChange = (key: string, value: string) => {
    if (key === 'email') onFilterEmailChange(value)
    else if (key === 'orgId') onFilterOrgIdChange(value)
    else if (key === 'role') onFilterRoleChange(value)
    else if (key === 'active') onFilterActiveChange(value as '' | 'yes' | 'no')
    else if (key === 'admin') onFilterAdminChange(value as '' | 'yes' | 'no')
  }
  const fields: FilterField[] = [
    {
      type: 'text',
      key: 'email',
      label: 'Email',
      placeholder: 'Search by email',
      minWidth: 220,
      inputType: 'search',
    },
    {
      type: 'select',
      key: 'orgId',
      label: 'Organization',
      minWidth: 180,
      options: [
        { value: '', label: 'All' },
        ...organizations.map((org) => ({ value: org.id, label: org.name })),
      ],
    },
    {
      type: 'select',
      key: 'role',
      label: 'Role',
      minWidth: 120,
      options: [
        { value: '', label: 'All' },
        ...roleOptions.map((r) => ({ value: r, label: r })),
      ],
    },
    {
      type: 'select',
      key: 'active',
      label: 'Active',
      minWidth: 100,
      options: [...YES_NO_OPTIONS],
    },
    {
      type: 'select',
      key: 'admin',
      label: 'Admin',
      minWidth: 100,
      options: [...YES_NO_OPTIONS],
    },
  ]
  return (
    <FilterPanelFromConfig
      fields={fields}
      values={values}
      onChange={onChange}
    />
  )
}
