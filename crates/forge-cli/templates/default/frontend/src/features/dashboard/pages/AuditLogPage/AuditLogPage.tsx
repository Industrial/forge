import Box from '@mui/material/Box'
import { Effect } from 'effect'

import PageHeader from '@/components/PageHeader'
import FiltersBar from '@/components/FiltersBar'
import DataTable from '@/components/DataTable'
import type { DataTableColumn } from '@/components/DataTable'
import AuditLogFilters from '@/features/dashboard/components/AuditLogFilters'
import ErrorAlert from '@/components/ErrorAlert'
import { useServerList } from '@/hooks/useServerList'
import { useLiveRefreshTrigger } from '@/hooks/useLiveRefreshTrigger'
import { formatDate } from '@/features/dashboard/utils/formatDate'
import type { AuditLogEntry } from '@/features/dashboard/domain/AuditLogEntry'
import { AuditLog as AuditLogService } from '@/features/dashboard/services/AuditLog'

type AuditLogFilterState = {
  from: string
  to: string
  outcome: string
  eventKind: string
  action: string
  reason: string
}

const initialFilters: AuditLogFilterState = {
  from: '',
  to: '',
  outcome: '',
  eventKind: '',
  action: '',
  reason: '',
}

const auditLogColumns: readonly DataTableColumn<AuditLogEntry>[] = [
  {
    id: 'occurred_at',
    label: 'Time',
    render: (e) => formatDate(e.occurred_at),
  },
  {
    id: 'actor_id',
    label: 'Actor ID',
    render: (e) => `${e.actor_id.slice(0, 8)}…`,
  },
  { id: 'event_kind', label: 'Event', render: (e) => e.event_kind },
  { id: 'action', label: 'Action', render: (e) => e.action },
  { id: 'resource_type', label: 'Resource', render: (e) => e.resource_type },
  { id: 'outcome', label: 'Outcome', render: (e) => e.outcome },
  {
    id: 'reason',
    label: 'Reason',
    render: (e) => e.reason ?? '—',
  },
]

export default function AuditLogPage() {
  const { trigger: liveRefreshTrigger, connected: wsConnected } =
    useLiveRefreshTrigger('audit-log')

  const list = useServerList<AuditLogEntry, AuditLogFilterState>({
    fetch: (params) =>
      Effect.gen(function* () {
        const auditLog = yield* AuditLogService
        const result = yield* auditLog.list({
          limit: params.limit,
          offset: params.offset,
          from: params.filters.from || undefined,
          to: params.filters.to || undefined,
          outcome: params.filters.outcome || undefined,
          event_kind: params.filters.eventKind || undefined,
          action: params.filters.action || undefined,
          reason: params.filters.reason.trim() || undefined,
        })
        return { items: result.entries, total: result.total }
      }),
    initialFilters,
    deps: [liveRefreshTrigger],
  })

  const handleApplyFilters = () => {
    list.setPage(0)
  }

  const handleResetFilters = () => {
    list.setFilters(initialFilters)
  }

  return (
    <Box data-testid="audit-log-page">
      <PageHeader
        title="Audit log"
        description="Read-only list of audit events. Use filters to narrow results."
        liveConnected={wsConnected}
        data-testid="audit-log-page-title"
      />

      <FiltersBar>
        <AuditLogFilters
          from={list.filters.from}
          to={list.filters.to}
          outcome={list.filters.outcome}
          eventKind={list.filters.eventKind}
          action={list.filters.action}
          reason={list.filters.reason}
          onFromChange={(v) =>
            list.setFilters((prev) => ({ ...prev, from: v }))
          }
          onToChange={(v) =>
            list.setFilters((prev) => ({ ...prev, to: v }))
          }
          onOutcomeChange={(v) =>
            list.setFilters((prev) => ({ ...prev, outcome: v }))
          }
          onEventKindChange={(v) =>
            list.setFilters((prev) => ({ ...prev, eventKind: v }))
          }
          onActionChange={(v) =>
            list.setFilters((prev) => ({ ...prev, action: v }))
          }
          onReasonChange={(v) =>
            list.setFilters((prev) => ({ ...prev, reason: v }))
          }
          onApply={handleApplyFilters}
          onReset={handleResetFilters}
        />
      </FiltersBar>

      {list.errorMessage != null && (
        <ErrorAlert
          message={list.errorMessage}
          onClose={() => list.refresh()}
        />
      )}

      <DataTable<AuditLogEntry>
        columns={auditLogColumns}
        rows={list.items}
        loading={list.loading}
        getRowId={(e) => e.id}
        emptyMessage="No entries"
        ariaLabel="Audit log"
        dataTestId="audit-log-list"
        pagination={{
          page: list.page,
          rowsPerPage: list.rowsPerPage,
          totalCount: list.total,
          onPageChange: (_, newPage) => list.setPage(newPage),
          onRowsPerPageChange: (e) => {
            list.setRowsPerPage(parseInt(e.target.value, 10))
          },
          rowsPerPageOptions: list.rowsPerPageOptions,
        }}
      />
    </Box>
  )
}
