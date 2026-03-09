import type { ReactNode } from 'react'
import Box from '@mui/material/Box'
import Paper from '@mui/material/Paper'
import Typography from '@mui/material/Typography'
import { useComponentLogger } from '@/hooks'

export type FiltersPanelProps = {
  /** Panel title (e.g. "Filters"). Default "Filters". */
  title?: string
  children: ReactNode
}

/**
 * Shared panel for filter controls: Paper with optional title and a flex wrap box.
 * Use for Organizations, AuditLog, Users filter sections.
 */
export default function FiltersPanel({
  title = 'Filters',
  children,
}: FiltersPanelProps) {
  useComponentLogger('FiltersPanel')
  return (
    <Paper sx={{ p: 2, mb: 2 }}>
      <Typography variant="subtitle2" gutterBottom>
        {title}
      </Typography>
      <Box
        sx={{
          display: 'flex',
          flexWrap: 'wrap',
          gap: 2,
          alignItems: 'flex-end',
        }}
      >
        {children}
      </Box>
    </Paper>
  )
}
