import type { ReactNode } from 'react'
import Box from '@mui/material/Box'

export type FiltersBarProps = {
  /** Filter fields or controls (e.g. page-specific filter components). */
  children: ReactNode
  /** Optional content after the fields (e.g. Apply / Reset buttons). */
  extra?: ReactNode
}

/**
 * Layout wrapper for filter content. Renders children and optional extra in a
 * flex row with wrap and spacing. Presentational only; filter state lives in
 * the page or useServerList.
 */
export default function FiltersBar({ children, extra }: FiltersBarProps) {
  return (
    <Box
      data-testid="filters-bar"
      sx={{
        display: 'flex',
        flexWrap: 'wrap',
        gap: 2,
        alignItems: 'flex-end',
        mb: 2,
      }}
    >
      {children}
      {extra != null ? extra : null}
    </Box>
  )
}
