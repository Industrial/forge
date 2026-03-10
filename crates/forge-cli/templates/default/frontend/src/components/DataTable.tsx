import type { ReactNode } from 'react'
import Box from '@mui/material/Box'
import Table from '@mui/material/Table'
import TableBody from '@mui/material/TableBody'
import TableCell from '@mui/material/TableCell'
import TableContainer from '@mui/material/TableContainer'
import TableHead from '@mui/material/TableHead'
import TableRow from '@mui/material/TableRow'
import TablePagination from '@mui/material/TablePagination'
import Paper from '@mui/material/Paper'

import LoadingSpinner from '@/components/LoadingSpinner'
import TableEmptyRow from '@/components/TableEmptyRow'

export type DataTableColumn<T> = {
  id: string
  label: string
  align?: 'left' | 'right'
  render: (row: T) => ReactNode
}

export type DataTablePagination = {
  page: number
  rowsPerPage: number
  totalCount: number
  onPageChange: (event: unknown, newPage: number) => void
  onRowsPerPageChange: (event: React.ChangeEvent<HTMLInputElement>) => void
  rowsPerPageOptions: number[]
}

export type DataTableProps<T> = {
  /** Column definitions. */
  columns: readonly DataTableColumn<T>[]
  /** Data rows for the current page. */
  rows: readonly T[]
  /** When true, show loading spinner instead of table body. */
  loading?: boolean
  /** Row key for React. Default: row.id. */
  getRowId?: (row: T) => string
  /** Message when rows.length === 0 and not loading. */
  emptyMessage?: string
  /** Server-driven pagination. */
  pagination: DataTablePagination
  /** Optional actions column (e.g. edit/delete); only shown when canShow is true. */
  actionsColumn?: {
    canShow: boolean
    render: (row: T) => ReactNode
  }
  /** Accessible label for the table. */
  ariaLabel?: string
  /** Optional test id for the table container. */
  dataTestId?: string
}

function defaultGetRowId<T>(row: T): string {
  const r = row as { id?: unknown }
  return r?.id != null ? String(r.id) : ''
}

/**
 * Shared data table with column config, optional actions column, server
 * pagination, and empty state. Use with useServerList for data and pagination.
 */
export default function DataTable<T>({
  columns,
  rows,
  loading = false,
  getRowId = defaultGetRowId,
  emptyMessage = 'No items.',
  pagination,
  actionsColumn,
  ariaLabel = 'Data table',
  dataTestId,
}: DataTableProps<T>) {
  const hasActions = actionsColumn?.canShow === true
  const colSpan = columns.length + (hasActions ? 1 : 0)

  if (loading) {
    return <LoadingSpinner />
  }

  return (
    <>
      <TableContainer component={Paper} data-testid={dataTestId ?? 'data-table'}>
        <Table size="small" aria-label={ariaLabel}>
          <TableHead>
            <TableRow>
              {columns.map((col) => (
                <TableCell key={col.id} align={col.align}>
                  {col.label}
                </TableCell>
              ))}
              {hasActions && <TableCell align="right">Actions</TableCell>}
            </TableRow>
          </TableHead>
          <TableBody>
            {rows.length === 0 ? (
              <TableEmptyRow colSpan={colSpan}>
                {emptyMessage}
              </TableEmptyRow>
            ) : (
              rows.map((row) => {
                const id = getRowId(row)
                return (
                  <TableRow key={id}>
                    {columns.map((col) => (
                      <TableCell key={col.id} align={col.align}>
                        {col.render(row)}
                      </TableCell>
                    ))}
                    {hasActions && (
                      <TableCell align="right">
                        {actionsColumn!.render(row)}
                      </TableCell>
                    )}
                  </TableRow>
                )
              })
            )}
          </TableBody>
        </Table>
      </TableContainer>
      <Box component="div" sx={{ display: 'block' }}>
        <TablePagination
          component="div"
          count={pagination.totalCount}
          page={pagination.page}
          onPageChange={pagination.onPageChange}
          rowsPerPage={pagination.rowsPerPage}
          onRowsPerPageChange={pagination.onRowsPerPageChange}
          rowsPerPageOptions={pagination.rowsPerPageOptions}
          labelRowsPerPage="Rows per page:"
        />
      </Box>
    </>
  )
}
