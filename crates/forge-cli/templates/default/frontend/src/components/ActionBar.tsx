import type { ReactNode } from 'react'
import Box from '@mui/material/Box'

export type ActionBarProps = {
  /** Primary actions (e.g. Add button, or PermissionsAddBar content). */
  children: ReactNode
}

/**
 * Layout wrapper for primary actions above a list or table. Same flex layout
 * as GenericEntityCrud header row (flex, space-between, wrap, gap). Layout
 * only; permission and behavior stay in the page.
 */
export default function ActionBar({ children }: ActionBarProps) {
  return (
    <Box
      data-testid="action-bar"
      sx={{
        display: 'flex',
        alignItems: 'center',
        justifyContent: 'flex-end',
        flexWrap: 'wrap',
        gap: 1,
        mb: 2,
      }}
    >
      {children}
    </Box>
  )
}
