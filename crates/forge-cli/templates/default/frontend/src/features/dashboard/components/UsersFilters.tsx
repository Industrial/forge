import TextField from '@mui/material/TextField'
import FormControl from '@mui/material/FormControl'
import InputLabel from '@mui/material/InputLabel'
import MenuItem from '@mui/material/MenuItem'
import Select from '@mui/material/Select'

import FiltersPanel from '@/components/FiltersPanel'

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
  return (
    <FiltersPanel>
      <TextField
        label="Email"
        type="search"
        size="small"
        value={filterEmail}
        onChange={(e) => onFilterEmailChange(e.target.value)}
        placeholder="Search by email"
        sx={{ minWidth: 220 }}
        data-testid="users-filter-input"
      />
      <FormControl size="small" sx={{ minWidth: 180 }}>
        <InputLabel>Organization</InputLabel>
        <Select
          label="Organization"
          value={filterOrgId}
          onChange={(e) => onFilterOrgIdChange(e.target.value)}
        >
          <MenuItem value="">All</MenuItem>
          {organizations.map((org) => (
            <MenuItem key={org.id} value={org.id}>
              {org.name}
            </MenuItem>
          ))}
        </Select>
      </FormControl>
      <FormControl size="small" sx={{ minWidth: 120 }}>
        <InputLabel>Role</InputLabel>
        <Select
          label="Role"
          value={filterRole}
          onChange={(e) => onFilterRoleChange(e.target.value)}
        >
          <MenuItem value="">All</MenuItem>
          {roleOptions.map((r) => (
            <MenuItem key={r} value={r}>
              {r}
            </MenuItem>
          ))}
        </Select>
      </FormControl>
      <FormControl size="small" sx={{ minWidth: 100 }}>
        <InputLabel>Active</InputLabel>
        <Select
          label="Active"
          value={filterActive}
          onChange={(e) =>
            onFilterActiveChange(e.target.value as '' | 'yes' | 'no')
          }
        >
          <MenuItem value="">All</MenuItem>
          <MenuItem value="yes">Yes</MenuItem>
          <MenuItem value="no">No</MenuItem>
        </Select>
      </FormControl>
      <FormControl size="small" sx={{ minWidth: 100 }}>
        <InputLabel>Admin</InputLabel>
        <Select
          label="Admin"
          value={filterAdmin}
          onChange={(e) =>
            onFilterAdminChange(e.target.value as '' | 'yes' | 'no')
          }
        >
          <MenuItem value="">All</MenuItem>
          <MenuItem value="yes">Yes</MenuItem>
          <MenuItem value="no">No</MenuItem>
        </Select>
      </FormControl>
    </FiltersPanel>
  )
}
