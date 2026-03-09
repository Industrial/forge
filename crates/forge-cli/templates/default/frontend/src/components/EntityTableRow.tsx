import type { SxProps, Theme } from '@mui/material/styles'
import TableCell from '@mui/material/TableCell'
import TableRow from '@mui/material/TableRow'
import IconButton from '@mui/material/IconButton'
import EditIcon from '@mui/icons-material/Edit'
import DeleteIcon from '@mui/icons-material/Delete'

export type EntityTableRowColumn<T> = {
  key: string
  render: (item: T) => React.ReactNode
  cellSx?: SxProps<Theme>
}

export type EntityTableRowProps<T> = {
  item: T
  columns: readonly EntityTableRowColumn<T>[]
  getRowId: (item: T) => string
  testIdPrefix?: string
  onRowClick?: (item: T) => void
  /** Show Edit + Delete buttons. */
  canEditDelete?: boolean
  /** Show only Delete button (e.g. "Remove"). */
  canDeleteOnly?: boolean
  onEdit?: (item: T) => void
  onDelete?: (item: T) => void
  isDeleting?: boolean
  deleteAriaLabel?: string
}

/**
 * Generic table row: renders a row from column config and optional edit/delete actions.
 */
export default function EntityTableRow<T>({
  item,
  columns,
  getRowId,
  testIdPrefix = 'row',
  onRowClick,
  canEditDelete,
  canDeleteOnly,
  onEdit,
  onDelete,
  isDeleting = false,
  deleteAriaLabel = 'Delete',
}: EntityTableRowProps<T>): React.JSX.Element {
  const rowId = getRowId(item)
  const hasClick = Boolean(onRowClick || (canEditDelete && onEdit))
  const handleRowClick = () => {
    if (onRowClick) onRowClick(item)
    else if (canEditDelete && onEdit) onEdit(item)
  }
  const handleEdit = (e: React.MouseEvent) => {
    e?.stopPropagation?.()
    onEdit?.(item)
  }
  const handleDelete = (e: React.MouseEvent) => {
    e?.stopPropagation?.()
    onDelete?.(item)
  }

  return (
    <TableRow
      data-testid={`${testIdPrefix}-${rowId}`}
      onClick={hasClick ? handleRowClick : undefined}
      sx={hasClick ? { cursor: 'pointer' } : undefined}
    >
      {columns.map((col) => (
        <TableCell key={col.key} sx={col.cellSx}>
          {col.render(item)}
        </TableCell>
      ))}
      {(canEditDelete || canDeleteOnly) && (onEdit || onDelete) && (
        <TableCell align="right">
          {canEditDelete && onEdit && (
            <IconButton
              size="small"
              aria-label="Edit"
              onClick={handleEdit}
              data-testid={`${testIdPrefix}-edit-${rowId}`}
            >
              <EditIcon />
            </IconButton>
          )}
          {onDelete && (canEditDelete || canDeleteOnly) && (
            <IconButton
              size="small"
              aria-label={deleteAriaLabel}
              onClick={handleDelete}
              disabled={isDeleting}
              data-testid={`${testIdPrefix}-delete-${rowId}`}
            >
              <DeleteIcon />
            </IconButton>
          )}
        </TableCell>
      )}
    </TableRow>
  )
}
