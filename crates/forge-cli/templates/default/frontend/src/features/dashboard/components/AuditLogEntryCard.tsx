import Typography from '@mui/material/Typography'
import Box from '@mui/material/Box'
import Button from '@mui/material/Button'
import Paper from '@mui/material/Paper'
import VisibilityIcon from '@mui/icons-material/Visibility'

import type { AuditLogEntry } from '@/features/dashboard/domain/AuditLogEntry'
import { formatDate } from '@/features/dashboard/utils/formatDate'

export type AuditLogEntryCardProps = {
  entry: AuditLogEntry
  onView: () => void
}

export default function AuditLogEntryCard({
  entry,
  onView,
}: AuditLogEntryCardProps) {
  return (
    <Paper sx={{ p: 2 }}>
      <Typography variant="subtitle2" color="text.secondary">
        {formatDate(entry.occurred_at)}
      </Typography>
      <Typography variant="subtitle1" fontWeight={600}>
        {entry.event_kind} · {entry.action} · {entry.resource_type}
      </Typography>
      <Typography variant="body2" color="text.secondary">
        Actor: {entry.actor_id.slice(0, 8)}… · Outcome: {entry.outcome}
      </Typography>
      {entry.reason != null && entry.reason !== '' && (
        <Typography
          variant="caption"
          color="text.secondary"
          display="block"
          sx={{ mt: 0.5 }}
        >
          {entry.reason}
        </Typography>
      )}
      <Box
        sx={{
          mt: 2,
          display: 'flex',
          gap: 0.5,
          justifyContent: 'flex-end',
        }}
      >
        <Button size="medium" startIcon={<VisibilityIcon />} onClick={onView}>
          View
        </Button>
      </Box>
    </Paper>
  )
}
