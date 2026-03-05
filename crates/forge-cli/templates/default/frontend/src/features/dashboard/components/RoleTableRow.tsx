import TableCell from '@mui/material/TableCell'
import TableRow from '@mui/material/TableRow'
import IconButton from '@mui/material/IconButton'
import EditIcon from '@mui/icons-material/Edit'
import DeleteIcon from '@mui/icons-material/Delete'

export type RoleRow = {
  id: string
  org_id: string
  name: string
  display_name: string | null
}

export type RoleTableRowProps = {
  role: RoleRow
  orgName: string
  onEdit: (role: RoleRow) => void
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
    <TableRow>
      <TableCell>{orgName}</TableCell>
      <TableCell sx={{ fontWeight: 500 }}>{role.name}</TableCell>
      <TableCell>{role.display_name ?? '—'}</TableCell>
      <TableCell align="right">
        <IconButton
          size="small"
          aria-label="Edit"
          onClick={() => onEdit(role)}
        >
          <EditIcon />
        </IconButton>
        <IconButton
          size="small"
          aria-label="Delete"
          onClick={() => onDelete(role.id)}
          disabled={isDeleting}
        >
          <DeleteIcon />
        </IconButton>
      </TableCell>
    </TableRow>
  )
}
