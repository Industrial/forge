import TableCell from '@mui/material/TableCell'
import TableRow from '@mui/material/TableRow'
import { formatDate } from '../utils/formatDate'

export type AuditLogEntryRow = {
  id: string
  occurred_at: string
  actor_id: string
  event_kind: string
  action: string
  resource_type: string
  outcome: string
  reason?: string | null
}

export type AuditLogTableRowProps = {
  entry: AuditLogEntryRow
}

export default function AuditLogTableRow({ entry }: AuditLogTableRowProps) {
  return (
    <TableRow>
      <TableCell sx={{ whiteSpace: 'nowrap' }}>
        {formatDate(entry.occurred_at)}
      </TableCell>
      <TableCell
        sx={{
          fontFamily: 'monospace',
          fontSize: '0.75rem',
        }}
      >
        {entry.actor_id.slice(0, 8)}…
      </TableCell>
      <TableCell>{entry.event_kind}</TableCell>
      <TableCell>{entry.action}</TableCell>
      <TableCell>{entry.resource_type}</TableCell>
      <TableCell>{entry.outcome}</TableCell>
      <TableCell
        sx={{
          maxWidth: 200,
          overflow: 'hidden',
          textOverflow: 'ellipsis',
        }}
      >
        {entry.reason ?? '—'}
      </TableCell>
    </TableRow>
  )
}
