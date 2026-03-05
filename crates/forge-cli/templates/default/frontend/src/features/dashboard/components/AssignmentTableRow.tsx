import TableCell from '@mui/material/TableCell'
import TableRow from '@mui/material/TableRow'
import IconButton from '@mui/material/IconButton'
import DeleteIcon from '@mui/icons-material/Delete'
import type { Assignment } from '../domain/Assignment'

export type AssignmentTableRowProps = {
  assignment: Assignment
  onDelete: (assignment: Assignment) => void
  isDeleting: boolean
}

export default function AssignmentTableRow({
  assignment,
  onDelete,
  isDeleting,
}: AssignmentTableRowProps) {
  return (
    <TableRow>
      <TableCell>{assignment.scope}</TableCell>
      <TableCell>{assignment.role_name}</TableCell>
      <TableCell>{assignment.permission_key}</TableCell>
      <TableCell align="right">
        <IconButton
          size="small"
          aria-label="Remove"
          onClick={() => onDelete(assignment)}
          disabled={isDeleting}
        >
          <DeleteIcon />
        </IconButton>
      </TableCell>
    </TableRow>
  )
}
