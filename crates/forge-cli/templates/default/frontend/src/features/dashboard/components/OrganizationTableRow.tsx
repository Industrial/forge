import TableCell from '@mui/material/TableCell'
import TableRow from '@mui/material/TableRow'
import IconButton from '@mui/material/IconButton'
import EditIcon from '@mui/icons-material/Edit'
import DeleteIcon from '@mui/icons-material/Delete'
import type { Organization } from '../domain'
import { formatDate } from '../utils/formatDate'

export type OrganizationTableRowProps = {
  org: Organization
  canWrite: boolean
  onEdit: (org: Organization) => void
  onDelete: (id: string) => void
  isDeleting: boolean
}

export default function OrganizationTableRow({
  org,
  canWrite,
  onEdit,
  onDelete,
  isDeleting,
}: OrganizationTableRowProps) {
  return (
    <TableRow>
      <TableCell sx={{ fontWeight: 500 }}>{org.name}</TableCell>
      <TableCell>{org.slug}</TableCell>
      <TableCell sx={{ whiteSpace: 'nowrap' }}>
        {formatDate(org.created_at)}
      </TableCell>
      <TableCell sx={{ whiteSpace: 'nowrap' }}>
        {formatDate(org.updated_at)}
      </TableCell>
      {canWrite && (
        <TableCell align="right">
          <IconButton
            size="small"
            aria-label="Edit"
            onClick={() => onEdit(org)}
          >
            <EditIcon />
          </IconButton>
          <IconButton
            size="small"
            aria-label="Delete"
            onClick={() => onDelete(org.id)}
            disabled={isDeleting}
          >
            <DeleteIcon />
          </IconButton>
        </TableCell>
      )}
    </TableRow>
  )
}
