import { useState } from 'react'
import Box from '@mui/material/Box'
import Button from '@mui/material/Button'
import Dialog from '@mui/material/Dialog'
import DialogActions from '@mui/material/DialogActions'
import DialogContent from '@mui/material/DialogContent'
import DialogTitle from '@mui/material/DialogTitle'
import IconButton from '@mui/material/IconButton'
import Typography from '@mui/material/Typography'
import VisibilityIcon from '@mui/icons-material/Visibility'
import { Effect } from 'effect'

import PageHeader from '@/components/PageHeader'
import DataTable from '@/components/DataTable'
import type { DataTableColumn } from '@/components/DataTable'
import DataListMobile from '@/components/DataListMobile'
import EmptyState from '@/components/EmptyState'
import LoadingSpinner from '@/components/LoadingSpinner'
import AuditLogFilters from '@/features/dashboard/components/AuditLogFilters'
import AuditLogEntryCard from '@/features/dashboard/components/AuditLogEntryCard'
import ErrorAlert from '@/components/ErrorAlert'
import { useIsMobile } from '@/hooks/useIsMobile'
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
  const isMobile = useIsMobile()
  const [viewEntry, setViewEntry] = useState<AuditLogEntry | null>(null)
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

  return (
    <Box data-testid="audit-log-page">
      <PageHeader
        title="Audit log"
        description="Read-only list of audit events. Use filters to narrow results."
        liveConnected={wsConnected}
        data-testid="audit-log-page-title"
      />

      <AuditLogFilters
        from={list.filters.from}
        to={list.filters.to}
        outcome={list.filters.outcome}
        eventKind={list.filters.eventKind}
        action={list.filters.action}
        reason={list.filters.reason}
        onFromChange={(v) => list.setFilters((prev) => ({ ...prev, from: v }))}
        onToChange={(v) => list.setFilters((prev) => ({ ...prev, to: v }))}
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
      />

      {list.errorMessage != null && (
        <ErrorAlert
          message={list.errorMessage}
          onClose={() => list.refresh()}
        />
      )}

      {list.loading ? (
        <LoadingSpinner />
      ) : list.items.length === 0 ? (
        <EmptyState message="No entries" />
      ) : isMobile ? (
        <DataListMobile<AuditLogEntry>
          items={list.items}
          getKey={(e) => e.id}
          renderItem={(entry) => (
            <AuditLogEntryCard
              entry={entry}
              onView={() => setViewEntry(entry)}
            />
          )}
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
          ariaLabel="Audit log"
          dataTestId="audit-log-list-mobile"
        />
      ) : (
        <DataTable<AuditLogEntry>
          columns={auditLogColumns}
          rows={list.items}
          loading={false}
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
          actionsColumn={{
            canShow: true,
            render: (entry) => (
              <IconButton
                size="small"
                color="primary"
                aria-label="View"
                onClick={() => setViewEntry(entry)}
                data-testid={`row-view-${entry.id}`}
              >
                <VisibilityIcon />
              </IconButton>
            ),
          }}
        />
      )}

      <Dialog
        open={viewEntry != null}
        onClose={() => setViewEntry(null)}
        maxWidth="sm"
        fullWidth
        data-testid="audit-log-entry-details-dialog"
      >
        <DialogTitle>Entry Details</DialogTitle>
        <DialogContent>
          {viewEntry && (
            <Box
              sx={{
                display: 'flex',
                flexDirection: 'column',
                gap: 2,
                pt: 1,
              }}
            >
              <Box>
                <Typography variant="subtitle2" color="text.secondary">
                  Time
                </Typography>
                <Typography variant="body1">
                  {formatDate(viewEntry.occurred_at)}
                </Typography>
              </Box>
              <Box>
                <Typography variant="subtitle2" color="text.secondary">
                  Actor ID
                </Typography>
                <Typography variant="body1">{viewEntry.actor_id}</Typography>
              </Box>
              {viewEntry.subject_id != null && viewEntry.subject_id !== '' && (
                <Box>
                  <Typography variant="subtitle2" color="text.secondary">
                    Subject ID
                  </Typography>
                  <Typography variant="body1">
                    {viewEntry.subject_id}
                  </Typography>
                </Box>
              )}
              {viewEntry.organization_id != null &&
                viewEntry.organization_id !== '' && (
                  <Box>
                    <Typography variant="subtitle2" color="text.secondary">
                      Organization ID
                    </Typography>
                    <Typography variant="body1">
                      {viewEntry.organization_id}
                    </Typography>
                  </Box>
                )}
              <Box>
                <Typography variant="subtitle2" color="text.secondary">
                  Event
                </Typography>
                <Typography variant="body1">{viewEntry.event_kind}</Typography>
              </Box>
              <Box>
                <Typography variant="subtitle2" color="text.secondary">
                  Action
                </Typography>
                <Typography variant="body1">{viewEntry.action}</Typography>
              </Box>
              <Box>
                <Typography variant="subtitle2" color="text.secondary">
                  Resource type
                </Typography>
                <Typography variant="body1">
                  {viewEntry.resource_type}
                </Typography>
              </Box>
              {viewEntry.resource_id != null &&
                viewEntry.resource_id !== '' && (
                  <Box>
                    <Typography variant="subtitle2" color="text.secondary">
                      Resource ID
                    </Typography>
                    <Typography variant="body1">
                      {viewEntry.resource_id}
                    </Typography>
                  </Box>
                )}
              <Box>
                <Typography variant="subtitle2" color="text.secondary">
                  Outcome
                </Typography>
                <Typography variant="body1">{viewEntry.outcome}</Typography>
              </Box>
              {(viewEntry.reason ?? '').length > 0 && (
                <Box>
                  <Typography variant="subtitle2" color="text.secondary">
                    Reason
                  </Typography>
                  <Typography variant="body1">
                    {viewEntry.reason ?? '—'}
                  </Typography>
                </Box>
              )}
            </Box>
          )}
        </DialogContent>
        <DialogActions>
          <Button onClick={() => setViewEntry(null)}>Close</Button>
        </DialogActions>
      </Dialog>
    </Box>
  )
}
