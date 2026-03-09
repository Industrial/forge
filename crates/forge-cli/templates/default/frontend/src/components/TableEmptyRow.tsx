import type { ReactNode } from 'react'
import TableCell from '@mui/material/TableCell'
import TableRow from '@mui/material/TableRow'
import { useComponentLogger } from '@/hooks'

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
  useComponentLogger('TableEmptyRow')
  return (
    <TableRow>
      <TableCell colSpan={colSpan} align="center">
        {children}
      </TableCell>
    </TableRow>
  )
}
