import { useEffect, useState } from 'react'
import {
  useEffectRuntime,
  useEffectState,
  useRunEffect,
  streamWithPendingState,
  runStreamInto,
  type AsyncState,
  idle,
  success as asyncSuccess,
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
import TablePagination from '@mui/material/TablePagination'
import Paper from '@mui/material/Paper'
import PageHeader from '../../../../components/PageHeader'
import TableEmptyRow from '../../../../components/TableEmptyRow'
import PermissionsAddBar from '../../components/PermissionsAddBar'
import AssignmentTableRow from '../../components/AssignmentTableRow'
import ErrorAlert from '../../../../components/ErrorAlert'
import LoadingSpinner from '../../../../components/LoadingSpinner'
import { useLiveRefreshTrigger } from '../../../../hooks/useLiveRefreshTrigger'
import { useTablePaginationDefaults } from '../../../../hooks/useTablePaginationDefaults'
import type { Assignment } from '../../domain/Assignment'
import {
  Permissions as PermissionsService,
  type PermissionsData,
} from '../../services/Permissions'

const ORG_ROLES = ['owner', 'admin', 'editor', 'viewer'] as const
const GLOBAL_ROLES = ['platform_admin'] as const

type ListState = AsyncState<PermissionsData, Error>

export default function PermissionsPage() {
  const { runtime } = useEffectRuntime<AppServices>()
  const { trigger: liveRefreshTrigger, connected: wsConnected } =
    useLiveRefreshTrigger('role_permissions')

  const [listState, , setListStateAsEffect] = useEffectState<ListState, never, never>(
    idle<PermissionsData, Error>(),
  )
  const [addState, , setAddStateAsEffect] = useEffectState<ListState, never, never>(
    idle<PermissionsData, Error>(),
  )
  const [deleteState, , setDeleteStateAsEffect] =
    useEffectState<ListState, never, never>(idle<PermissionsData, Error>())

  const assignments = isSuccess(listState) ? listState.value.assignments : []
  const permissions = isSuccess(listState) ? listState.value.permissions : []
  const loading = isPending(listState)
  const error: Error | null = isFailure(listState)
    ? listState.error
    : isFailure(addState)
      ? addState.error
      : isFailure(deleteState)
        ? deleteState.error
        : null
  const errorMessage =
    error instanceof Error
      ? error.message
      : error != null
        ? String(error)
        : null

  const [addScope, setAddScope] = useState<string>('org')
  const [addRole, setAddRole] = useState<string>('owner')
  const [addPermission, setAddPermission] = useState<string>('')
  const [deletingKey, setDeletingKey] = useState<string | null>(null)
  const { defaultRowsPerPage, rowsPerPageOptions } =
    useTablePaginationDefaults()
  const [page, setPage] = useState(0)
  const [rowsPerPage, setRowsPerPage] = useState(defaultRowsPerPage)

  const adding = isPending(addState)
  const deleting = isPending(deleteState)

  const getDataEffect = Effect.gen(function* () {
    const permissions = yield* PermissionsService
    return yield* permissions.getData()
  })
  const refreshStream = streamWithPendingState(getDataEffect)
  const refreshEffect = Effect.gen(function* () {
    yield* runStreamInto(refreshStream, setListStateAsEffect)
  })

  useRunEffect(refreshEffect, [liveRefreshTrigger, setListStateAsEffect])

  // On add success: copy to listState and reset add state
  useRunEffect(
    Effect.gen(function* () {
      if (!isSuccess(addState)) return
      yield* setListStateAsEffect(asyncSuccess(addState.value))
      yield* setAddStateAsEffect(idle())
    }),
    [addState, setListStateAsEffect, setAddStateAsEffect],
  )

  // On delete success: copy to listState, clear deletingKey, reset delete state
  useRunEffect(
    Effect.gen(function* () {
      if (!isSuccess(deleteState)) return
      yield* setListStateAsEffect(asyncSuccess(deleteState.value))
      yield* Effect.sync(() => setDeletingKey(null))
      yield* setDeleteStateAsEffect(idle())
    }),
    [deleteState, setListStateAsEffect, setDeleteStateAsEffect],
  )

  // Sync addPermission when permissions list loads (keep selection valid)
  useEffect(() => {
    if (permissions.length > 0 && !permissions.includes(addPermission)) {
      setAddPermission(permissions[0])
    }
  }, [permissions, addPermission])

  // Sync rowsPerPage when breakpoint default changes (e.g. window resize)
  useEffect(() => {
    setRowsPerPage(defaultRowsPerPage)
    setPage(0)
  }, [defaultRowsPerPage])

  // Reset page if it goes out of range (e.g. after deleting items)
  useEffect(() => {
    const maxPage = Math.max(0, Math.ceil(assignments.length / rowsPerPage) - 1)
    if (page > maxPage) setPage(maxPage)
  }, [assignments.length, rowsPerPage, page])

  const handleAdd = () => {
    if (!addScope || !addRole || !addPermission) return
    const addThenList = Effect.gen(function* () {
      const permissions = yield* PermissionsService
      yield* permissions.add({
        scope: addScope,
        role_name: addRole,
        permission_key: addPermission,
      })
      return yield* permissions.getData()
    })
    runWithAppRuntime(
      runtime,
      runStreamInto(streamWithPendingState(addThenList), setAddStateAsEffect),
    )
  }

  const handleDelete = (a: Assignment) => {
    const key = [a.scope, a.role_name, a.permission_key, a.org_id ?? ''].join(
      ':',
    )
    setDeletingKey(key)
    const deleteThenList = Effect.gen(function* () {
      const permissions = yield* PermissionsService
      yield* permissions.delete({
        scope: a.scope,
        role_name: a.role_name,
        permission_key: a.permission_key,
        org_id: a.org_id,
      })
      return yield* permissions.getData()
    })
    runWithAppRuntime(
      runtime,
      runStreamInto(
        streamWithPendingState(deleteThenList),
        setDeleteStateAsEffect,
      ),
    )
  }

  const addBarRoles =
    addScope === 'global' ? [...GLOBAL_ROLES] : [...ORG_ROLES]

  const handleChangePage = (_: unknown, newPage: number) => {
    setPage(newPage)
  }

  const handleChangeRowsPerPage = (e: React.ChangeEvent<HTMLInputElement>) => {
    setRowsPerPage(parseInt(e.target.value, 10))
    setPage(0)
  }

  const paginatedAssignments = assignments.slice(
    page * rowsPerPage,
    page * rowsPerPage + rowsPerPage,
  )

  return (
    <>
      <PageHeader
        title="Permissions"
        description={
          <>
            View and manage role–permission assignments. List/view requires{' '}
            <code>dashboard.permissions.read</code>; add/delete requires{' '}
            <code>dashboard.permissions.write</code>.
          </>
        }
        liveConnected={wsConnected}
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
          <PermissionsAddBar
            scope={addScope}
            role={addRole}
            permission={addPermission}
            roles={addBarRoles}
            permissions={permissions}
            onScopeChange={(value) => {
              setAddScope(value)
              setAddRole(value === 'global' ? 'platform_admin' : 'owner')
            }}
            onRoleChange={setAddRole}
            onPermissionChange={setAddPermission}
            onAdd={handleAdd}
            adding={adding}
          />

          <TableContainer component={Paper}>
            <Table size="small" aria-label="Role–permission assignments">
              <TableHead>
                <TableRow>
                  <TableCell>Scope</TableCell>
                  <TableCell>Role</TableCell>
                  <TableCell>Permission</TableCell>
                  <TableCell align="right">Actions</TableCell>
                </TableRow>
              </TableHead>
              <TableBody>
                  {paginatedAssignments.length === 0 ? (
                    <TableEmptyRow colSpan={4}>
                      No assignments yet. Add one above.
                    </TableEmptyRow>
                  ) : (
                    paginatedAssignments.map((a) => {
                      const key = [
                        a.scope,
                        a.role_name,
                        a.permission_key,
                        a.org_id ?? '',
                      ].join(':')
                      return (
                        <AssignmentTableRow
                          key={key}
                          assignment={a}
                          onDelete={handleDelete}
                          isDeleting={deleting && deletingKey === key}
                        />
                      )
                    })
                  )}
                </TableBody>
            </Table>
          </TableContainer>
          <TablePagination
            component="div"
            count={assignments.length}
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
