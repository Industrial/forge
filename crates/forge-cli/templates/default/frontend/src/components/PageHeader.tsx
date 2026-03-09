import type { ReactNode } from 'react'
import Box from '@mui/material/Box'
import Chip from '@mui/material/Chip'
import Typography from '@mui/material/Typography'

export type PageHeaderProps = {
  title: string
  description?: ReactNode
  /** When true, shows a "Live" chip next to the title. */
  liveConnected?: boolean
  /** Optional data-testid for the heading (e.g. for E2E tests). */
  'data-testid'?: string
}

export default function PageHeader({
  title,
  description,
  liveConnected = false,
  'data-testid': dataTestId,
}: PageHeaderProps) {
  return (
    <>
      <Box sx={{ display: 'flex', alignItems: 'center', gap: 1, mb: 0 }}>
        <Typography
          variant="h4"
          component="h1"
          gutterBottom
          sx={{ mb: 0 }}
          {...(dataTestId != null ? { 'data-testid': dataTestId } : {})}
        >
          {title}
        </Typography>
        {liveConnected && <Chip label="Live" color="success" size="small" />}
      </Box>
      {description != null && (
        <Typography variant="body2" color="text.secondary" sx={{ mb: 2 }}>
          {description}
        </Typography>
      )}
    </>
  )
}
