import EntityTableRow from '@/components/EntityTableRow'
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
    <EntityTableRow<AuditLogEntryRow>
      item={entry}
      getRowId={(e) => e.id}
      testIdPrefix="audit-log-row"
      columns={[
        {
          key: 'occurred_at',
          render: (e) => formatDate(e.occurred_at),
          cellSx: { whiteSpace: 'nowrap' },
        },
        {
          key: 'actor_id',
          render: (e) => `${e.actor_id.slice(0, 8)}…`,
          cellSx: { fontFamily: 'monospace', fontSize: '0.75rem' },
        },
        { key: 'event_kind', render: (e) => e.event_kind },
        { key: 'action', render: (e) => e.action },
        { key: 'resource_type', render: (e) => e.resource_type },
        { key: 'outcome', render: (e) => e.outcome },
        {
          key: 'reason',
          render: (e) => e.reason ?? '—',
          cellSx: {
            maxWidth: 200,
            overflow: 'hidden',
            textOverflow: 'ellipsis',
          },
        },
      ]}
    />
  )
}
