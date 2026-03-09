import type { ReactNode } from 'react'
import TableCell from '@mui/material/TableCell'
import TableRow from '@mui/material/TableRow'

export type TableEmptyRowProps = {
  colSpan: number
  children: ReactNode
}

/**
 * Single table row for empty state: one cell spanning all columns, centered.
 * Use in TableBody when the list is empty.
 */
export default function TableEmptyRow({
  colSpan,
  children,
}: TableEmptyRowProps) {
  return (
    <TableRow>
      <TableCell colSpan={colSpan} align="center">
        {children}
      </TableCell>
    </TableRow>
  )
}
