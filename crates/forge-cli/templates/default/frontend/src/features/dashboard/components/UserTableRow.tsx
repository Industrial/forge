import TableCell from '@mui/material/TableCell'
import TableRow from '@mui/material/TableRow'
import IconButton from '@mui/material/IconButton'
import EditIcon from '@mui/icons-material/Edit'
import DeleteIcon from '@mui/icons-material/Delete'
import { formatDate } from '../utils/formatDate'
import { membershipsSummary } from '../utils/membershipsSummary'

export type UserRow = {
  id: string
  email: string
  is_active: boolean
  is_admin: boolean
  created_at: string
  memberships: readonly { org_name: string; roles?: readonly string[] | null }[]
}

export type UserTableRowProps = {
  user: UserRow
  canWrite: boolean
  onEdit: (user: UserRow) => void
  onDelete: (id: string) => void
  isDeleting: boolean
}

export default function UserTableRow({
  user,
  canWrite,
  onEdit,
  onDelete,
  isDeleting,
}: UserTableRowProps) {
  return (
    <TableRow>
      <TableCell sx={{ fontWeight: 500 }}>{user.email}</TableCell>
      <TableCell sx={{ maxWidth: 280 }}>
        {membershipsSummary(user.memberships)}
      </TableCell>
      <TableCell>{user.is_active ? 'Yes' : 'No'}</TableCell>
      <TableCell>{user.is_admin ? 'Yes' : 'No'}</TableCell>
      <TableCell sx={{ whiteSpace: 'nowrap' }}>
        {formatDate(user.created_at)}
      </TableCell>
      {canWrite && (
        <TableCell align="right">
          <IconButton
            size="small"
            aria-label="Edit"
            onClick={() => onEdit(user)}
          >
            <EditIcon />
          </IconButton>
          <IconButton
            size="small"
            aria-label="Delete"
            onClick={() => onDelete(user.id)}
            disabled={isDeleting}
          >
            <DeleteIcon />
          </IconButton>
        </TableCell>
      )}
    </TableRow>
  )
}
