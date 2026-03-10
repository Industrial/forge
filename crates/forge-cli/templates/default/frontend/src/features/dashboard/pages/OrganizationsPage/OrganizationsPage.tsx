import Box from '@mui/material/Box'
import Button from '@mui/material/Button'
import TextField from '@mui/material/TextField'
import AddIcon from '@mui/icons-material/Add'
import Dialog from '@mui/material/Dialog'
import DialogActions from '@mui/material/DialogActions'
import DialogContent from '@mui/material/DialogContent'
import DialogTitle from '@mui/material/DialogTitle'
import IconButton from '@mui/material/IconButton'
import VisibilityIcon from '@mui/icons-material/Visibility'
import EditIcon from '@mui/icons-material/Edit'
import DeleteIcon from '@mui/icons-material/Delete'
import Typography from '@mui/material/Typography'
import { useEffect, useMemo, useState } from 'react'
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
import { Effect } from 'effect'

import FormDialog from '@/components/FormDialog'
import ActionBar from '@/components/ActionBar'
import DataTable from '@/components/DataTable'
import type { DataTableColumn } from '@/components/DataTable'
import DataListMobile from '@/components/DataListMobile'
import LoadingSpinner from '@/components/LoadingSpinner'
import EmptyState from '@/components/EmptyState'
import PageHeader from '@/components/PageHeader'
import ErrorAlert from '@/components/ErrorAlert'
import { ShowWithPermissions } from '@/components/ShowWithPermissions'
import { useEntitySubscription } from '@/hooks/useEntitySubscription'
import { useLiveRefreshTrigger } from '@/hooks/useLiveRefreshTrigger'
import { usePermission } from '@/hooks/usePermission'
import { useIsMobile } from '@/hooks/useIsMobile'
import { useTablePaginationDefaults } from '@/hooks/useTablePaginationDefaults'
import OrganizationsFilters from '@/features/dashboard/components/OrganizationsFilters'
import OrganizationCard from '@/features/dashboard/components/OrganizationCard'
import { useOrganizationsFilter } from '@/features/dashboard/hooks/useOrganizationsFilter'
import { formatDate } from '@/features/dashboard/utils/formatDate'
import { getApplicationLayer, type AppServices } from '@/lib/appLayer'
import { EntityApi } from '@/services/EntityApi'
import { Organization } from '@/features/dashboard/domain/Organization'

const ENTITY_ID = 'organization'
const ORG_WRITE = 'organization.create'

function toOrganization(r: Record<string, unknown>): Organization {
  return new Organization({
    id: String(r.id ?? ''),
    name: String(r.name ?? ''),
    slug: String(r.slug ?? ''),
    created_at: r.created_at != null ? String(r.created_at) : undefined,
    updated_at: r.updated_at != null ? String(r.updated_at) : undefined,
  })
}

const listEffect: Effect.Effect<readonly Organization[], Error, AppServices> =
  Effect.gen(function* () {
    const api = yield* EntityApi
    const res = yield* api.list(ENTITY_ID)
    return (res.data ?? []).map((r) =>
      toOrganization(r as Record<string, unknown>),
    )
  })

type ListState = AsyncState<readonly Organization[], Error>

export type OrganizationFormProps = {
  mode: 'add' | 'edit'
  name: string
  slug: string
  onNameChange: (value: string) => void
  onSlugChange: (value: string) => void
  disabled?: boolean
}

function OrganizationForm({
  mode,
  name,
  slug,
  onNameChange,
  onSlugChange,
  disabled = false,
}: OrganizationFormProps) {
  const slugLabel = mode === 'add' ? 'Slug (optional)' : 'Slug'
  const slugPlaceholder = mode === 'add' ? 'Auto from name if blank' : undefined
  return (
    <Box
      sx={{
        display: 'flex',
        flexDirection: 'column',
        gap: 2,
        pt: 1,
        minWidth: 0,
      }}
    >
      <TextField
        label="Name"
        size="small"
        fullWidth
        value={name}
        onChange={(e) => onNameChange(e.target.value)}
        required
        disabled={disabled}
      />
      <TextField
        label={slugLabel}
        size="small"
        fullWidth
        value={slug}
        onChange={(e) => onSlugChange(e.target.value)}
        placeholder={slugPlaceholder}
        disabled={disabled}
      />
    </Box>
  )
}

export default function OrganizationsPage() {
  const isMobile = useIsMobile()
  const canWrite = usePermission(ORG_WRITE)
  const { trigger: liveRefreshTrigger, connected: wsConnected } =
    useLiveRefreshTrigger('organizations')

  /** List state: only updated by refresh stream (and on mutation success we copy new list here). */
  const [listState, , setListStateAsEffect] = useEffectState<
    ListState,
    never,
    never
  >(idle<readonly Organization[], Error>())

  /** Create state: "create then list" stream; on success we copy to list and close add dialog. */
  const [createState, , setCreateStateAsEffect] = useEffectState<
    ListState,
    never,
    never
  >(idle<readonly Organization[], Error>())

  /** Update state: "update then list" stream; on success we copy to list and close edit dialog. */
  const [updateState, , setUpdateStateAsEffect] = useEffectState<
    ListState,
    never,
    never
  >(idle<readonly Organization[], Error>())

  /** Delete state: "delete then list" stream; on success we copy to list and clear deletingId. */
  const [deleteState, , setDeleteStateAsEffect] = useEffectState<
    ListState,
    never,
    never
  >(idle<readonly Organization[], Error>())

  const organizations = isSuccess(listState) ? listState.value : []
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

  const [filters, setFilters, filteredOrganizations] =
    useOrganizationsFilter(organizations)

  const { defaultRowsPerPage, rowsPerPageOptions } =
    useTablePaginationDefaults()
  const [page, setPage] = useState(0)
  const [rowsPerPage, setRowsPerPage] = useState(defaultRowsPerPage)
  useEffect(() => {
    setRowsPerPage((prev) =>
      rowsPerPageOptions.includes(prev) ? prev : defaultRowsPerPage,
    )
  }, [defaultRowsPerPage, rowsPerPageOptions])
  useEffect(() => setPage(0), [filters.filterName, filters.filterSlug])

  const paginatedOrganizations = useMemo(
    () =>
      filteredOrganizations.slice(
        page * rowsPerPage,
        page * rowsPerPage + rowsPerPage,
      ),
    [filteredOrganizations, page, rowsPerPage],
  )

  const orgColumns: DataTableColumn<Organization>[] = useMemo(
    () => [
      { id: 'name', label: 'Name', render: (o) => o.name },
      {
        id: 'created',
        label: 'Created',
        render: (o) => formatDate(o.created_at),
      },
    ],
    [],
  )

  const refreshStream = streamWithPendingState(listEffect)
  const refreshEffect = Effect.gen(function* () {
    yield* runStreamInto(refreshStream, setListStateAsEffect)
  })

  useEntitySubscription(ENTITY_ID, undefined, () => {
    Effect.runPromise(refreshEffect.pipe(Effect.provide(getApplicationLayer())))
  })

  const [addDialogOpen, setAddDialogOpen, setAddDialogOpenAsEffect] =
    useEffectState(false)
  const [addName, setAddName, setAddNameAsEffect] = useEffectState('')
  const [addSlug, setAddSlug, setAddSlugAsEffect] = useEffectState('')
  const [viewOrg, setViewOrg] = useState<Organization | null>(null)
  const [editOrg, setEditOrg, setEditOrgAsEffect] =
    useEffectState<Organization | null>(null)
  const [editName, setEditName] = useEffectState('')
  const [editSlug, setEditSlug] = useEffectState('')
  const [deletingId, setDeletingId, setDeletingIdAsEffect] = useEffectState<
    string | null
  >(null)

  const isAdding = isPending(createState)
  const isSaving = isPending(updateState)
  const isDeleting = (id: string) => isPending(deleteState) && deletingId === id

  const errorMessage =
    error instanceof Error
      ? error.message
      : error != null
        ? String(error)
        : null

  const handleAdd = () => {
    const name = addName.trim()
    if (!name) return
    Effect.runPromise(
      runStreamInto(
        streamWithPendingState(
          Effect.gen(function* () {
            const api = yield* EntityApi
            yield* api.create(ENTITY_ID, {
              name,
              slug: addSlug.trim() || undefined,
            })
            const res = yield* api.list(ENTITY_ID)
            return (res.data ?? []).map((r) =>
              toOrganization(r as Record<string, unknown>),
            )
          }),
        ),
        setCreateStateAsEffect,
      ).pipe(Effect.provide(getApplicationLayer())),
    )
  }

  const handleOpenEdit = (org: Organization) => {
    setEditOrg(org)
    setEditName(org.name)
    setEditSlug(org.slug)
  }

  const handleSaveEdit = () => {
    if (!editOrg) return
    Effect.runPromise(
      runStreamInto(
        streamWithPendingState(
          Effect.gen(function* () {
            const api = yield* EntityApi
            yield* api.update(ENTITY_ID, editOrg.id, {
              name: editName.trim() || undefined,
              slug: editSlug.trim() || undefined,
            })
            const res = yield* api.list(ENTITY_ID)
            return (res.data ?? []).map((r) =>
              toOrganization(r as Record<string, unknown>),
            )
          }),
        ),
        setUpdateStateAsEffect,
      ).pipe(Effect.provide(getApplicationLayer())),
    )
  }

  const handleDelete = (id: string) => {
    setDeletingId(id)
    Effect.runPromise(
      runStreamInto(
        streamWithPendingState(
          Effect.gen(function* () {
            const api = yield* EntityApi
            yield* api.delete(ENTITY_ID, id)
            const res = yield* api.list(ENTITY_ID)
            return (res.data ?? []).map((r) =>
              toOrganization(r as Record<string, unknown>),
            )
          }),
        ),
        setDeleteStateAsEffect,
      ).pipe(Effect.provide(getApplicationLayer())),
    )
  }

  const handleClearError = () => {
    Effect.runPromise(
      Effect.gen(function* () {
        yield* setCreateStateAsEffect(idle())
        yield* setUpdateStateAsEffect(idle())
        yield* setDeleteStateAsEffect(idle())
        yield* refreshEffect
      }).pipe(Effect.provide(getApplicationLayer())),
    )
  }

  // Initial load: run once when page mounts
  useEffect(() => {
    Effect.runPromise(refreshEffect.pipe(Effect.provide(getApplicationLayer())))
  }, [])

  // Live-refresh: re-run when liveRefreshTrigger fires
  useEffect(() => {
    Effect.runPromise(refreshEffect.pipe(Effect.provide(getApplicationLayer())))
  }, [liveRefreshTrigger, setListStateAsEffect])

  // On create success: copy list to listState, close add dialog, reset create state
  useEffect(() => {
    if (!isSuccess(createState)) return
    Effect.runPromise(
      Effect.gen(function* () {
        yield* setListStateAsEffect(asyncSuccess(createState.value))
        yield* setAddDialogOpenAsEffect(false)
        yield* setAddNameAsEffect('')
        yield* setAddSlugAsEffect('')
        yield* setCreateStateAsEffect(idle())
      }).pipe(Effect.provide(getApplicationLayer())),
    )
  }, [
    createState,
    setListStateAsEffect,
    setAddDialogOpenAsEffect,
    setAddNameAsEffect,
    setAddSlugAsEffect,
    setCreateStateAsEffect,
  ])

  // On update success: copy list to listState, close edit dialog, reset update state
  useEffect(() => {
    if (!isSuccess(updateState)) return
    Effect.runPromise(
      Effect.gen(function* () {
        yield* setListStateAsEffect(asyncSuccess(updateState.value))
        yield* setEditOrgAsEffect(null)
        yield* setUpdateStateAsEffect(idle())
      }).pipe(Effect.provide(getApplicationLayer())),
    )
  }, [
    updateState,
    setListStateAsEffect,
    setEditOrgAsEffect,
    setUpdateStateAsEffect,
  ])

  // On delete success: copy list to listState, clear deletingId, reset delete state
  useEffect(() => {
    if (!isSuccess(deleteState)) return
    Effect.runPromise(
      Effect.gen(function* () {
        yield* setListStateAsEffect(asyncSuccess(deleteState.value))
        yield* setDeletingIdAsEffect(null)
        yield* setDeleteStateAsEffect(idle())
      }).pipe(Effect.provide(getApplicationLayer())),
    )
  }, [
    deleteState,
    setListStateAsEffect,
    setDeletingIdAsEffect,
    setDeleteStateAsEffect,
  ])

  return (
    <Box data-testid="organizations-page">
      <PageHeader
        title="Organizations"
        description={
          <>
            View and manage organizations. Write actions require{' '}
            <code>organization.create</code>.
          </>
        }
        liveConnected={wsConnected}
      />

      {errorMessage != null && (
        <ErrorAlert message={errorMessage} onClose={handleClearError} />
      )}

      <OrganizationsFilters
        filterName={filters.filterName}
        filterSlug={filters.filterSlug}
        onFilterNameChange={(value) =>
          setFilters((prev) => ({ ...prev, filterName: value }))
        }
        onFilterSlugChange={(value) =>
          setFilters((prev) => ({ ...prev, filterSlug: value }))
        }
      />

      <ShowWithPermissions permissions={[ORG_WRITE]}>
        <ActionBar>
          <Button
            variant="contained"
            startIcon={<AddIcon />}
            onClick={() => setAddDialogOpen(true)}
          >
            Add organization
          </Button>
        </ActionBar>
      </ShowWithPermissions>

      {loading ? (
        <LoadingSpinner />
      ) : filteredOrganizations.length === 0 ? (
        <EmptyState
          message={
            organizations.length === 0
              ? 'No organizations.'
              : 'No organizations match the filters.'
          }
        />
      ) : isMobile ? (
        <DataListMobile<Organization>
          items={filteredOrganizations}
          getKey={(o) => o.id}
          renderItem={(org) => (
            <OrganizationCard
              org={org}
              canWrite={canWrite}
              onView={setViewOrg}
              onEdit={handleOpenEdit}
              onDelete={handleDelete}
              isDeleting={isDeleting(org.id)}
            />
          )}
          ariaLabel="Organizations"
          dataTestId="organizations-list-mobile"
        />
      ) : (
        <DataTable<Organization>
          columns={orgColumns}
          rows={paginatedOrganizations}
          loading={false}
          getRowId={(o) => o.id}
          emptyMessage={
            organizations.length === 0
              ? 'No organizations.'
              : 'No organizations match the filters.'
          }
          pagination={{
            page,
            rowsPerPage,
            totalCount: filteredOrganizations.length,
            onPageChange: (_ev, newPage) => setPage(newPage),
            onRowsPerPageChange: (ev) => {
              setRowsPerPage(parseInt(ev.target.value, 10))
              setPage(0)
            },
            rowsPerPageOptions,
          }}
          actionsColumn={{
            canShow: true,
            render: (org) => (
              <>
                <IconButton
                  size="small"
                  color="primary"
                  aria-label="View"
                  onClick={() => setViewOrg(org)}
                  data-testid={`row-view-${org.id}`}
                >
                  <VisibilityIcon />
                </IconButton>
                {canWrite && (
                  <>
                    <IconButton
                      size="small"
                      color="primary"
                      aria-label="Edit"
                      onClick={() => handleOpenEdit(org)}
                      data-testid={`row-edit-${org.id}`}
                    >
                      <EditIcon />
                    </IconButton>
                    <IconButton
                      size="small"
                      color="error"
                      aria-label="Delete"
                      onClick={() => handleDelete(org.id)}
                      disabled={isDeleting(org.id)}
                      data-testid={`row-delete-${org.id}`}
                    >
                      <DeleteIcon />
                    </IconButton>
                  </>
                )}
              </>
            ),
          }}
          ariaLabel="Organizations"
        />
      )}

      <FormDialog
        open={addDialogOpen}
        onClose={() => setAddDialogOpen(false)}
        title="Add organization"
        submitLabel="Add"
        submittingLabel="Adding…"
        onSubmit={handleAdd}
        submitDisabled={!addName.trim()}
        submitting={isAdding}
        contentSx={{ minWidth: 0 }}
      >
        <OrganizationForm
          mode="add"
          name={addName}
          slug={addSlug}
          onNameChange={setAddName}
          onSlugChange={setAddSlug}
          disabled={isAdding}
        />
      </FormDialog>

      <FormDialog
        open={Boolean(editOrg)}
        onClose={() => setEditOrg(null)}
        title="Edit organization"
        submitLabel="Save"
        submittingLabel="Saving…"
        onSubmit={handleSaveEdit}
        submitDisabled={false}
        submitting={isSaving}
        contentSx={{ minWidth: 0 }}
      >
        <OrganizationForm
          mode="edit"
          name={editName}
          slug={editSlug}
          onNameChange={setEditName}
          onSlugChange={setEditSlug}
          disabled={isSaving}
        />
      </FormDialog>

      <Dialog
        open={Boolean(viewOrg)}
        onClose={() => setViewOrg(null)}
        maxWidth="sm"
        fullWidth
        data-testid="organization-details-dialog"
      >
        <DialogTitle>Organization Details</DialogTitle>
        <DialogContent>
          {viewOrg && (
            <Box
              sx={{ display: 'flex', flexDirection: 'column', gap: 2, pt: 1 }}
            >
              <Box>
                <Typography variant="subtitle2" color="text.secondary">
                  Name
                </Typography>
                <Typography variant="body1">{viewOrg.name}</Typography>
              </Box>
              <Box>
                <Typography variant="subtitle2" color="text.secondary">
                  Slug
                </Typography>
                <Typography variant="body1">{viewOrg.slug}</Typography>
              </Box>
              <Box>
                <Typography variant="subtitle2" color="text.secondary">
                  Created
                </Typography>
                <Typography variant="body1">
                  {formatDate(viewOrg.created_at)}
                </Typography>
              </Box>
              <Box>
                <Typography variant="subtitle2" color="text.secondary">
                  Updated
                </Typography>
                <Typography variant="body1">
                  {formatDate(viewOrg.updated_at)}
                </Typography>
              </Box>
            </Box>
          )}
        </DialogContent>
        <DialogActions>
          <Button onClick={() => setViewOrg(null)}>Close</Button>
        </DialogActions>
      </Dialog>
    </Box>
  )
}
