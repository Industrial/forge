import { useEffect, useState } from 'react'
import useTheme from '@mui/material/styles/useTheme'
import useMediaQuery from '@mui/material/useMediaQuery'

/**
 * Breakpoint-based defaults for table pagination so that rows-per-page
 * reflects available space (window width): smaller viewport → fewer rows.
 * Shared by Audit log and Permissions (and any other paginated tables).
 *
 * - xs/sm (narrow): default 10, options [10, 25]
 * - md: default 25, options [10, 25, 50]
 * - lg+: default 50, options [10, 25, 50, 100]
 */
export function useTablePaginationDefaults(): {
  defaultRowsPerPage: number
  rowsPerPageOptions: number[]
} {
  const theme = useTheme()
  const isMdUp = useMediaQuery(theme.breakpoints.up('md'))
  const isLgUp = useMediaQuery(theme.breakpoints.up('lg'))

  const [defaults, setDefaults] = useState(() => ({
    defaultRowsPerPage: 25,
    rowsPerPageOptions: [10, 25, 50, 100] as number[],
  }))

  useEffect(() => {
    if (isLgUp) {
      setDefaults({
        defaultRowsPerPage: 50,
        rowsPerPageOptions: [10, 25, 50, 100],
      })
    } else if (isMdUp) {
      setDefaults({ defaultRowsPerPage: 25, rowsPerPageOptions: [10, 25, 50] })
    } else {
      setDefaults({ defaultRowsPerPage: 10, rowsPerPageOptions: [10, 25] })
    }
  }, [isMdUp, isLgUp])

  return defaults
}
