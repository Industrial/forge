import { useEffect, useState } from 'react'
import {
  useEffectRuntime,
  useEffectState,
  useRunEffect,
  streamWithPendingState,
  runStreamInto,
  type AsyncState,
  idle,
  isSuccess,
  isFailure,
  isPending,
} from 'react-effect-hooks'
import { runWithAppRuntime, type AppServices } from '../../../../lib/appLayer'
import { Effect } from 'effect'
import Table from '@mui/material/Table'
import TableBody from '@mui/material/TableBody'
import TableCell from '@mui/material/TableCell'
import TableContainer from '@mui/material/TableContainer'
import TableHead from '@mui/material/TableHead'
import TableRow from '@mui/material/TableRow'
import Paper from '@mui/material/Paper'
import TablePagination from '@mui/material/TablePagination'
import PageHeader from '../../../../components/PageHeader'
import TableEmptyRow from '../../../../components/TableEmptyRow'
import AuditLogFilters from '../../components/AuditLogFilters'
import AuditLogTableRow from '../../components/AuditLogTableRow'
import ErrorAlert from '../../../../components/ErrorAlert'
import LoadingSpinner from '../../../../components/LoadingSpinner'
import { useLiveRefreshTrigger } from '../../../../hooks/useLiveRefreshTrigger'
import { useTablePaginationDefaults } from '../../../../hooks/useTablePaginationDefaults'
import type { AuditLogResult } from '../../services/AuditLog'
import { AuditLog as AuditLogService } from '../../services/AuditLog'

type ListState = AsyncState<AuditLogResult, Error>

export default function AuditLogPage() {
  const { runtime } = useEffectRuntime<AppServices>()
  const { trigger: liveRefreshTrigger, connected: wsConnected } =
    useLiveRefreshTrigger('audit-log')

  const [listState, , setListStateAsEffect] = useEffectState<ListState, never, never>(
    idle<AuditLogResult, Error>(),
  )

  const entries = isSuccess(listState) ? listState.value.entries : []
  const total = isSuccess(listState) ? listState.value.total : 0
  const loading = isPending(listState)
  const error: Error | null = isFailure(listState) ? listState.error : null
  const errorMessage =
    error instanceof Error
      ? error.message
      : error != null
        ? String(error)
        : null

  const { defaultRowsPerPage, rowsPerPageOptions } =
    useTablePaginationDefaults()
  const [page, setPage] = useState(0)
  const [rowsPerPage, setRowsPerPage] = useState(defaultRowsPerPage)
  const [from, setFrom] = useState('')
  const [to, setTo] = useState('')
  const [outcome, setOutcome] = useState('')
  const [eventKind, setEventKind] = useState('')
  const [action, setAction] = useState('')
  const [reason, setReason] = useState('')

  const listEffect: Effect.Effect<AuditLogResult, Error, AppServices> =
    Effect.gen(function* () {
      const auditLog = yield* AuditLogService
      return yield* auditLog.list({
        limit: rowsPerPage,
        offset: page * rowsPerPage,
        from: from || undefined,
        to: to || undefined,
        outcome: outcome || undefined,
        event_kind: eventKind || undefined,
        action: action || undefined,
        reason: reason.trim() || undefined,
      })
    })

  const refreshStream = streamWithPendingState(listEffect)
  const refreshEffect = Effect.gen(function* () {
    yield* runStreamInto(refreshStream, setListStateAsEffect)
  })

  useRunEffect(refreshEffect, [
    liveRefreshTrigger,
    page,
    rowsPerPage,
    from,
    to,
    outcome,
    eventKind,
    action,
    reason,
    setListStateAsEffect,
  ])

  // Sync rowsPerPage when breakpoint default changes (e.g. window resize)
  useEffect(() => {
    setRowsPerPage(defaultRowsPerPage)
    setPage(0)
  }, [defaultRowsPerPage])

  const handleApplyFilters = () => {
    setPage(0)
  }

  const handleResetFilters = () => {
    setFrom('')
    setTo('')
    setOutcome('')
    setEventKind('')
    setAction('')
    setReason('')
    setPage(0)
  }

  const handleChangePage = (_: unknown, newPage: number) => {
    setPage(newPage)
  }

  const handleChangeRowsPerPage = (e: React.ChangeEvent<HTMLInputElement>) => {
    setRowsPerPage(parseInt(e.target.value, 10))
    setPage(0)
  }

  return (
    <>
      <PageHeader
        title="Audit log"
        description="Read-only list of audit events. Use filters to narrow results."
        liveConnected={wsConnected}
      />

      <AuditLogFilters
        from={from}
        to={to}
        outcome={outcome}
        eventKind={eventKind}
        action={action}
        reason={reason}
        onFromChange={setFrom}
        onToChange={setTo}
        onOutcomeChange={setOutcome}
        onEventKindChange={setEventKind}
        onActionChange={setAction}
        onReasonChange={setReason}
        onApply={handleApplyFilters}
        onReset={handleResetFilters}
      />

      {errorMessage != null && (
        <ErrorAlert
          message={errorMessage}
          onClose={() => {
            runWithAppRuntime(runtime, refreshEffect)
          }}
        />
      )}

      {loading ? (
        <LoadingSpinner />
      ) : (
        <>
          <TableContainer component={Paper}>
            <Table size="small" aria-label="Audit log">
              <TableHead>
                <TableRow>
                  <TableCell>Time</TableCell>
                  <TableCell>Actor ID</TableCell>
                  <TableCell>Event</TableCell>
                  <TableCell>Action</TableCell>
                  <TableCell>Resource</TableCell>
                  <TableCell>Outcome</TableCell>
                  <TableCell>Reason</TableCell>
                </TableRow>
              </TableHead>
              <TableBody>
                {entries.length === 0 ? (
                  <TableEmptyRow colSpan={7}>No entries</TableEmptyRow>
                ) : (
                  entries.map((row) => (
                    <AuditLogTableRow key={row.id} entry={row} />
                  ))
                )}
              </TableBody>
            </Table>
          </TableContainer>
          <TablePagination
            component="div"
            count={total}
            page={page}
            onPageChange={handleChangePage}
            rowsPerPage={rowsPerPage}
            onRowsPerPageChange={handleChangeRowsPerPage}
            rowsPerPageOptions={rowsPerPageOptions}
            labelRowsPerPage="Rows per page:"
          />
        </>
      )}
    </>
  )
}
