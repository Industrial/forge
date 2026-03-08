import TableCell from '@mui/material/TableCell'
import TableRow from '@mui/material/TableRow'
import IconButton from '@mui/material/IconButton'
import EditIcon from '@mui/icons-material/Edit'
import DeleteIcon from '@mui/icons-material/Delete'

import type { Role } from '@/features/dashboard/domain/Role'

export type RoleTableRowProps = {
  role: Role
  orgName: string
  onEdit: (role: Role) => void
  onDelete: (id: string) => void
  isDeleting: boolean
}

export default function RoleTableRow({
  role,
  orgName,
  onEdit,
  onDelete,
  isDeleting,
}: RoleTableRowProps) {
  return (
    <TableRow data-testid={`role-row-${role.name}`}>
      <TableCell>{orgName}</TableCell>
      <TableCell sx={{ fontWeight: 500 }}>{role.name}</TableCell>
      <TableCell>{role.display_name ?? '—'}</TableCell>
      <TableCell align="right">
        <IconButton size="small" aria-label="Edit" onClick={() => onEdit(role)} data-testid={`role-edit-button-${role.name}`}>
          <EditIcon />
        </IconButton>
        <IconButton
          size="small"
          aria-label="Delete"
          onClick={() => onDelete(role.id)}
          disabled={isDeleting}
          data-testid={`role-delete-button-${role.name}`}
        >
          <DeleteIcon />
        </IconButton>
      </TableCell>
    </TableRow>
  )
}
