/**
 * Generic CRUD UI driven by entity_id and EntityApi.
 * List/get/create/update/delete with configurable columns and optional create/edit form slots.
 * Use as-is or extend per screen (custom columns, custom forms).
 *
 * @see EntityApi, docs/frontend-implementation-and-choices.md (Epic 5)
 */

import { useState, useCallback, useEffect } from 'react'
import Box from '@mui/material/Box'
import Button from '@mui/material/Button'
import Table from '@mui/material/Table'
import TableBody from '@mui/material/TableBody'
import TableCell from '@mui/material/TableCell'
import TableContainer from '@mui/material/TableContainer'
import TableHead from '@mui/material/TableHead'
import TableRow from '@mui/material/TableRow'
import Paper from '@mui/material/Paper'
import IconButton from '@mui/material/IconButton'
import AddIcon from '@mui/icons-material/Add'
import EditIcon from '@mui/icons-material/Edit'
import DeleteIcon from '@mui/icons-material/Delete'
import Dialog from '@mui/material/Dialog'
import DialogTitle from '@mui/material/DialogTitle'
import DialogContent from '@mui/material/DialogContent'
import DialogActions from '@mui/material/DialogActions'
import PageHeader from './PageHeader'
import LoadingSpinner from './LoadingSpinner'
import EmptyState from './EmptyState'
import ErrorAlert from './ErrorAlert'
import {
  useEffectState,
  streamWithPendingState,
  runStreamInto,
  type AsyncState,
  idle,
  success as asyncSuccess,
  isSuccess,
  isFailure,
  isPending,
} from 'react-effect-hooks'
import { runApp } from '../lib/appRuntime'
import type { AppServices } from '../lib/appLayer'
import { Effect } from 'effect'
import { EntityApi } from '../services/EntityApi'
import type { ListQueryParams } from '../services/EntityApi'
import { usePermission } from '../hooks/usePermission'

export type GenericEntityCrudColumn = {
  key: string
  label: string
  render?: (item: Record<string, unknown>) => React.ReactNode
}

export type GenericEntityCrudProps = {
  entityId: string
  title: string
  listQueryParams?: ListQueryParams
  /** Column definitions. If not provided, columns are derived from first item keys (id, then others). */
  columns?: GenericEntityCrudColumn[]
  /** Row id for get/update/delete. Default: item.id */
  getRowId?: (item: Record<string, unknown>) => string
  /** When provided, Add button and create dialog are shown. onSubmit receives the form body. */
  renderCreateForm?: (
    onSubmit: (body: Record<string, unknown>) => void,
    onClose: () => void,
  ) => React.ReactNode
  /** When provided, Edit action and edit dialog are shown. onSubmit receives the patch body. */
  renderEditForm?: (
    item: Record<string, unknown>,
    onSubmit: (body: Record<string, unknown>) => void,
    onClose: () => void,
  ) => React.ReactNode
  /** Optional empty message when list is empty. */
  emptyMessage?: string
}

function defaultGetRowId(item: Record<string, unknown>): string {
  const id = item?.id
  return id != null ? String(id) : ''
}

function defaultColumns(
  items: readonly Record<string, unknown>[],
): GenericEntityCrudColumn[] {
  const first = items[0]
  if (!first || typeof first !== 'object') return [{ key: 'id', label: 'ID' }]
  const keys = Object.keys(first)
  const idFirst = keys.includes('id')
    ? ['id', ...keys.filter((k) => k !== 'id')]
    : keys
  return idFirst.slice(0, 8).map((key) => ({
    key,
    label: key.replace(/_/g, ' '),
    render: (item) => {
      const v = item[key]
      if (v == null) return '—'
      if (typeof v === 'object') return JSON.stringify(v)
      return String(v)
    },
  }))
}

const listEffect = (
  entityId: string,
  params: ListQueryParams | undefined,
): Effect.Effect<readonly Record<string, unknown>[], Error, AppServices> =>
  Effect.gen(function* () {
    const api = yield* EntityApi
    const res = yield* api.list(entityId, params)
    return (res.data ?? []) as readonly Record<string, unknown>[]
  })

type ListState = AsyncState<readonly Record<string, unknown>[], Error>

export default function GenericEntityCrud({
  entityId,
  title,
  listQueryParams,
  columns: columnsProp,
  getRowId = defaultGetRowId,
  renderCreateForm,
  renderEditForm,
  emptyMessage = 'No items.',
}: GenericEntityCrudProps) {
  const canRead = usePermission(`${entityId}.read`)
  const canCreate = usePermission(`${entityId}.create`)
  const canUpdate = usePermission(`${entityId}.update`)
  const canDelete = usePermission(`${entityId}.delete`)

  const [listState, , setListStateAsEffect] = useEffectState<
    ListState,
    never,
    never
  >(idle<readonly Record<string, unknown>[], Error>())
  const [createState, , setCreateStateAsEffect] = useEffectState<
    ListState,
    never,
    never
  >(idle<readonly Record<string, unknown>[], Error>())
  const [updateState, , setUpdateStateAsEffect] = useEffectState<
    ListState,
    never,
    never
  >(idle<readonly Record<string, unknown>[], Error>())
  const [deleteState, , setDeleteStateAsEffect] = useEffectState<
    ListState,
    never,
    never
  >(idle<readonly Record<string, unknown>[], Error>())

  const [addDialogOpen, setAddDialogOpen] = useState(false)
  const [editItem, setEditItem] = useState<Record<string, unknown> | null>(null)
  const [deletingId, setDeletingId] = useState<string | null>(null)
  const [deleteConfirmId, setDeleteConfirmId] = useState<string | null>(null)

  const items = isSuccess(listState) ? listState.value : []
  const loading = isPending(listState)
  const error: Error | null = isFailure(listState)
    ? listState.error
    : isFailure(createState)
      ? createState.error
      : isFailure(updateState)
        ? updateState.error
        : isFailure(deleteState)
          ? deleteState.error
          : null

  const columns =
    columnsProp ?? defaultColumns(items as Record<string, unknown>[])
  const hasActions = canUpdate || canDelete

  const refreshEffect = Effect.gen(function* () {
    yield* runStreamInto(
      streamWithPendingState(listEffect(entityId, listQueryParams)),
      setListStateAsEffect,
    )
  })

  useEffect(() => {
    runApp(refreshEffect)
  }, [entityId, JSON.stringify(listQueryParams ?? {}), setListStateAsEffect])

  const handleCreateSubmit = useCallback(
    (body: Record<string, unknown>) => {
      runApp(
        runStreamInto(
          streamWithPendingState(
            Effect.gen(function* () {
              const api = yield* EntityApi
              yield* api.create(entityId, body)
              const res = yield* api.list(entityId, listQueryParams)
              return (res.data ?? []) as readonly Record<string, unknown>[]
            }),
          ),
          setCreateStateAsEffect,
        ),
      )
      setAddDialogOpen(false)
    },
    [entityId, listQueryParams, setCreateStateAsEffect],
  )

  const handleUpdateSubmit = useCallback(
    (body: Record<string, unknown>) => {
      if (!editItem) return
      const id = getRowId(editItem)
      runApp(
        runStreamInto(
          streamWithPendingState(
            Effect.gen(function* () {
              const api = yield* EntityApi
              yield* api.update(entityId, id, body)
              const res = yield* api.list(entityId, listQueryParams)
              return (res.data ?? []) as readonly Record<string, unknown>[]
            }),
          ),
          setUpdateStateAsEffect,
        ),
      )
      setEditItem(null)
    },
    [entityId, editItem, getRowId, listQueryParams, setUpdateStateAsEffect],
  )

  const handleDeleteConfirm = useCallback(
    (id: string) => {
      setDeletingId(id)
      setDeleteConfirmId(null)
      runApp(
        runStreamInto(
          streamWithPendingState(
            Effect.gen(function* () {
              const api = yield* EntityApi
              yield* api.delete(entityId, id)
              const res = yield* api.list(entityId, listQueryParams)
              return (res.data ?? []) as readonly Record<string, unknown>[]
            }),
          ),
          setDeleteStateAsEffect,
        ),
      )
    },
    [entityId, listQueryParams, setDeleteStateAsEffect],
  )

  // On create/update/delete success: refresh list state
  useEffect(() => {
    if (!isSuccess(createState)) return
    runApp(
      Effect.gen(function* () {
        yield* setListStateAsEffect(asyncSuccess(createState.value))
        yield* setCreateStateAsEffect(idle())
      }),
    )
  }, [createState, setListStateAsEffect, setCreateStateAsEffect])
  useEffect(() => {
    if (!isSuccess(updateState)) return
    runApp(
      Effect.gen(function* () {
        yield* setListStateAsEffect(asyncSuccess(updateState.value))
        yield* setUpdateStateAsEffect(idle())
      }),
    )
  }, [updateState, setListStateAsEffect, setUpdateStateAsEffect])
  useEffect(() => {
    if (!isSuccess(deleteState)) return
    runApp(
      Effect.gen(function* () {
        yield* setListStateAsEffect(asyncSuccess(deleteState.value))
        yield* setDeleteStateAsEffect(idle())
      }),
    )
  }, [deleteState, setListStateAsEffect, setDeleteStateAsEffect])

  useEffect(() => {
    if (!isPending(deleteState)) setDeletingId(null)
  }, [deleteState])

  if (!canRead) {
    return (
      <>
        <PageHeader title={title} />
        <ErrorAlert
          message="You do not have permission to view this resource."
          onClose={() => {}}
        />
      </>
    )
  }

  const errorMessage =
    error instanceof Error
      ? error.message
      : error != null
        ? String(error)
        : null

  return (
    <Box>
      <Box
        sx={{
          display: 'flex',
          alignItems: 'center',
          justifyContent: 'space-between',
          flexWrap: 'wrap',
          gap: 1,
          mb: 2,
        }}
      >
        <PageHeader title={title} />
        {canCreate && renderCreateForm != null && (
          <Button
            variant="contained"
            startIcon={<AddIcon />}
            onClick={() => setAddDialogOpen(true)}
            disabled={isPending(createState)}
          >
            Add
          </Button>
        )}
      </Box>

      {errorMessage != null && (
        <ErrorAlert
          message={errorMessage}
          onClose={() => {
            runApp(refreshEffect)
          }}
        />
      )}

      {loading && <LoadingSpinner />}

      {!loading && items.length === 0 && <EmptyState message={emptyMessage} />}

      {!loading && items.length > 0 && (
        <TableContainer component={Paper}>
          <Table size="small">
            <TableHead>
              <TableRow>
                {columns.map((col) => (
                  <TableCell key={col.key}>{col.label}</TableCell>
                ))}
                {hasActions && <TableCell align="right">Actions</TableCell>}
              </TableRow>
            </TableHead>
            <TableBody>
              {(items as Record<string, unknown>[]).map((item) => {
                const id = getRowId(item)
                const isDeleting = deletingId === id
                return (
                  <TableRow key={id}>
                    {columns.map((col) => (
                      <TableCell key={col.key}>
                        {col.render
                          ? col.render(item)
                          : String(item[col.key] ?? '—')}
                      </TableCell>
                    ))}
                    {hasActions && (
                      <TableCell align="right">
                        {canUpdate && renderEditForm != null && (
                          <IconButton
                            size="small"
                            aria-label="Edit"
                            onClick={() => setEditItem(item)}
                          >
                            <EditIcon />
                          </IconButton>
                        )}
                        {canDelete && (
                          <IconButton
                            size="small"
                            aria-label="Delete"
                            onClick={() => setDeleteConfirmId(id)}
                            disabled={isDeleting}
                          >
                            <DeleteIcon />
                          </IconButton>
                        )}
                      </TableCell>
                    )}
                  </TableRow>
                )
              })}
            </TableBody>
          </Table>
        </TableContainer>
      )}

      {addDialogOpen && renderCreateForm != null && (
        <Dialog
          open={true}
          onClose={() => setAddDialogOpen(false)}
          maxWidth="sm"
          fullWidth
        >
          <DialogTitle>Create</DialogTitle>
          <DialogContent>
            {renderCreateForm(handleCreateSubmit, () =>
              setAddDialogOpen(false),
            )}
          </DialogContent>
        </Dialog>
      )}

      {editItem != null && renderEditForm != null && (
        <Dialog
          open={true}
          onClose={() => setEditItem(null)}
          maxWidth="sm"
          fullWidth
        >
          <DialogTitle>Edit</DialogTitle>
          <DialogContent>
            {renderEditForm(editItem, handleUpdateSubmit, () =>
              setEditItem(null),
            )}
          </DialogContent>
        </Dialog>
      )}

      {deleteConfirmId != null && (
        <Dialog open={true} onClose={() => setDeleteConfirmId(null)}>
          <DialogTitle>Delete?</DialogTitle>
          <DialogContent>This action cannot be undone.</DialogContent>
          <DialogActions>
            <Button onClick={() => setDeleteConfirmId(null)}>Cancel</Button>
            <Button
              color="error"
              variant="contained"
              onClick={() => handleDeleteConfirm(deleteConfirmId)}
              disabled={deletingId === deleteConfirmId}
            >
              Delete
            </Button>
          </DialogActions>
        </Dialog>
      )}
    </Box>
  )
}
