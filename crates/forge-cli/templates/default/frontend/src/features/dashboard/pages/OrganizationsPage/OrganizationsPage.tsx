import Box from '@mui/material/Box'
import Button from '@mui/material/Button'
import TextField from '@mui/material/TextField'
import Table from '@mui/material/Table'
import TableBody from '@mui/material/TableBody'
import TableCell from '@mui/material/TableCell'
import TableContainer from '@mui/material/TableContainer'
import TableHead from '@mui/material/TableHead'
import TableRow from '@mui/material/TableRow'
import Paper from '@mui/material/Paper'
import AddIcon from '@mui/icons-material/Add'
import FormDialog from '../../../../components/FormDialog'
import LoadingSpinner from '../../../../components/LoadingSpinner'
import EmptyState from '../../../../components/EmptyState'
import PageHeader from '../../../../components/PageHeader'
import ErrorAlert from '../../../../components/ErrorAlert'
import { useEntitySubscription } from '../../../../hooks/useEntitySubscription'
import { useLiveRefreshTrigger } from '../../../../hooks/useLiveRefreshTrigger'
import { usePermission } from '../../../../hooks/usePermission'
import { useIsMobile } from '../../../../hooks/useIsMobile'
import OrganizationsFilters from '../../components/OrganizationsFilters'
import OrganizationCard from '../../components/OrganizationCard'
import OrganizationTableRow from '../../components/OrganizationTableRow'
import { useOrganizationsFilter } from '../../hooks/useOrganizationsFilter'
import { useEffect } from 'react'
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
import { runApp } from '../../../../lib/appRuntime'
import type { AppServices } from '../../../../lib/appLayer'
import { Effect } from 'effect'
import { EntityApi } from '../../../../services/EntityApi'
import { Organization } from '../../domain/Organization'

const ENTITY_ID = 'organization'
const ORG_WRITE = 'dashboard.organizations.write'

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

  const refreshStream = streamWithPendingState(listEffect)
  const refreshEffect = Effect.gen(function* () {
    yield* runStreamInto(refreshStream, setListStateAsEffect)
  })

  useEntitySubscription(ENTITY_ID, undefined, () => {
    runApp(refreshEffect)
  })

  const [addDialogOpen, setAddDialogOpen, setAddDialogOpenAsEffect] =
    useEffectState(false)
  const [addName, setAddName, setAddNameAsEffect] = useEffectState('')
  const [addSlug, setAddSlug, setAddSlugAsEffect] = useEffectState('')
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
    runApp(
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
      ),
    )
  }

  const handleOpenEdit = (org: Organization) => {
    setEditOrg(org)
    setEditName(org.name)
    setEditSlug(org.slug)
  }

  const handleSaveEdit = () => {
    if (!editOrg) return
    runApp(
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
      ),
    )
  }

  const handleDelete = (id: string) => {
    setDeletingId(id)
    runApp(
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
      ),
    )
  }

  const handleClearError = () => {
    runApp(
      Effect.gen(function* () {
        yield* setCreateStateAsEffect(idle())
        yield* setUpdateStateAsEffect(idle())
        yield* setDeleteStateAsEffect(idle())
        yield* refreshEffect
      }),
    )
  }

  // Initial load: run once when page mounts
  useEffect(() => {
    runApp(refreshEffect)
  }, [])

  // Live-refresh: re-run when liveRefreshTrigger fires
  useEffect(() => {
    runApp(refreshEffect)
  }, [liveRefreshTrigger, setListStateAsEffect])

  // On create success: copy list to listState, close add dialog, reset create state
  useEffect(() => {
    if (!isSuccess(createState)) return
    runApp(
      Effect.gen(function* () {
        yield* setListStateAsEffect(asyncSuccess(createState.value))
        yield* setAddDialogOpenAsEffect(false)
        yield* setAddNameAsEffect('')
        yield* setAddSlugAsEffect('')
        yield* setCreateStateAsEffect(idle())
      }),
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
    runApp(
      Effect.gen(function* () {
        yield* setListStateAsEffect(asyncSuccess(updateState.value))
        yield* setEditOrgAsEffect(null)
        yield* setUpdateStateAsEffect(idle())
      }),
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
    runApp(
      Effect.gen(function* () {
        yield* setListStateAsEffect(asyncSuccess(deleteState.value))
        yield* setDeletingIdAsEffect(null)
        yield* setDeleteStateAsEffect(idle())
      }),
    )
  }, [
    deleteState,
    setListStateAsEffect,
    setDeletingIdAsEffect,
    setDeleteStateAsEffect,
  ])

  return (
    <>
      <PageHeader
        title="Organizations"
        description={
          <>
            View and manage organizations. Write actions require{' '}
            <code>dashboard.organizations.write</code>.
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

      {canWrite && (
        <Box sx={{ mb: 2 }}>
          <Button
            variant="contained"
            startIcon={<AddIcon />}
            onClick={() => setAddDialogOpen(true)}
          >
            Add organization
          </Button>
        </Box>
      )}

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
        <Box
          component="ul"
          sx={{
            listStyle: 'none',
            m: 0,
            p: 0,
            display: 'flex',
            flexDirection: 'column',
            gap: 1.5,
          }}
        >
          {filteredOrganizations.map((org) => (
            <Box key={org.id} component="li">
              <OrganizationCard
                org={org}
                canWrite={canWrite}
                onEdit={handleOpenEdit}
                onDelete={handleDelete}
                isDeleting={isDeleting(org.id)}
              />
            </Box>
          ))}
        </Box>
      ) : (
        <TableContainer component={Paper}>
          <Table size="small" aria-label="Organizations">
            <TableHead>
              <TableRow>
                <TableCell>Name</TableCell>
                <TableCell>Slug</TableCell>
                <TableCell>Created</TableCell>
                <TableCell>Updated</TableCell>
                {canWrite && <TableCell align="right">Actions</TableCell>}
              </TableRow>
            </TableHead>
            <TableBody>
              {filteredOrganizations.map((org) => (
                <OrganizationTableRow
                  key={org.id}
                  org={org}
                  canWrite={canWrite}
                  onEdit={handleOpenEdit}
                  onDelete={handleDelete}
                  isDeleting={isDeleting(org.id)}
                />
              ))}
            </TableBody>
          </Table>
        </TableContainer>
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
    </>
  )
}
