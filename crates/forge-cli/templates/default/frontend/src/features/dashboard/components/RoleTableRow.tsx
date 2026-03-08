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
  onView?: (role: Role) => void
  canWrite: boolean
  isDeleting: boolean
}

export default function RoleTableRow({
  role,
  orgName,
  onEdit,
  onDelete,
  onView,
  canWrite,
  isDeleting,
}: RoleTableRowProps) {
  const handleRowClick = () => {
    if (onView) {
      onView(role)
    } else if (canWrite) {
      onEdit(role)
    }
  }

  return (
    <TableRow
      data-testid={`role-row-${role.name}`}
      onClick={handleRowClick}
      sx={{ cursor: 'pointer' }}
    >
      <TableCell>{orgName}</TableCell>
      <TableCell sx={{ fontWeight: 500 }}>{role.name}</TableCell>
      <TableCell>{role.display_name ?? '—'}</TableCell>
      {canWrite && (
        <TableCell align="right">
          <IconButton
            size="small"
            aria-label="Edit"
            onClick={(e) => {
              e.stopPropagation()
              onEdit(role)
            }}
            data-testid={`role-edit-button-${role.name}`}
          >
            <EditIcon />
          </IconButton>
          <IconButton
            size="small"
            aria-label="Delete"
            onClick={(e) => {
              e.stopPropagation()
              onDelete(role.id)
            }}
            disabled={isDeleting}
            data-testid={`role-delete-button-${role.name}`}
          >
            <DeleteIcon />
          </IconButton>
        </TableCell>
      )}
    </TableRow>
  )
}
