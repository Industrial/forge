import Box from '@mui/material/Box'
import Button from '@mui/material/Button'
import FormControl from '@mui/material/FormControl'
import InputLabel from '@mui/material/InputLabel'
import MenuItem from '@mui/material/MenuItem'
import Select from '@mui/material/Select'
import { useComponentLogger } from '@/hooks'

export type PermissionsAddBarProps = {
  scope: string
  role: string
  permission: string
  roles: readonly string[]
  permissions: readonly string[]
  onScopeChange: (value: string) => void
  onRoleChange: (value: string) => void
  onPermissionChange: (value: string) => void
  onAdd: () => void
  adding: boolean
}

export default function PermissionsAddBar({
  scope,
  role,
  permission,
  roles,
  permissions,
  onScopeChange,
  onRoleChange,
  onPermissionChange,
  onAdd,
  adding,
}: PermissionsAddBarProps) {
  useComponentLogger('PermissionsAddBar')
  return (
    <Box
      sx={{
        display: 'flex',
        flexWrap: 'wrap',
        gap: 2,
        alignItems: 'center',
        mb: 3,
      }}
    >
      <FormControl size="small" sx={{ minWidth: 100 }}>
        <InputLabel>Scope</InputLabel>
        <Select
          value={scope}
          label="Scope"
          onChange={(e) => onScopeChange(e.target.value)}
        >
          {['org', 'global'].map((s) => (
            <MenuItem key={s} value={s}>
              {s}
            </MenuItem>
          ))}
        </Select>
      </FormControl>
      <FormControl size="small" sx={{ minWidth: 140 }}>
        <InputLabel>Role</InputLabel>
        <Select
          value={role}
          label="Role"
          onChange={(e) => onRoleChange(e.target.value)}
        >
          {roles.map((r) => (
            <MenuItem key={r} value={r}>
              {r}
            </MenuItem>
          ))}
        </Select>
      </FormControl>
      <FormControl size="small" sx={{ minWidth: 200 }}>
        <InputLabel>Permission</InputLabel>
        <Select
          value={permission}
          label="Permission"
          onChange={(e) => onPermissionChange(e.target.value)}
        >
          {permissions.map((p) => (
            <MenuItem key={p} value={p}>
              {p}
            </MenuItem>
          ))}
        </Select>
      </FormControl>
      <Button
        variant="contained"
        onClick={onAdd}
        disabled={adding || !permission}
      >
        {adding ? 'Adding…' : 'Add'}
      </Button>
    </Box>
  )
}
