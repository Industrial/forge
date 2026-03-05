import Typography from '@mui/material/Typography'
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
import IconButton from '@mui/material/IconButton'
import AddIcon from '@mui/icons-material/Add'
import DeleteIcon from '@mui/icons-material/Delete'
import EditIcon from '@mui/icons-material/Edit'
import Alert from '@mui/material/Alert'
import CircularProgress from '@mui/material/CircularProgress'
import Chip from '@mui/material/Chip'
import useTheme from '@mui/material/styles/useTheme'
import FormDialog from '../../../../components/FormDialog'
import useMediaQuery from '@mui/material/useMediaQuery'
import { useAuthentication } from '../../../../context/AuthenticationContext'
import { useLiveUpdates } from '../../../../context/LiveWs'
import {
  useEffectState,
  useRunEffect,
  useEffectRuntime,
  streamWithPendingState,
  type AsyncState,
  idle,
  success as asyncSuccess,
  isSuccess,
  isFailure,
  isPending,
} from '../../../../lib/react-effect'
import { runWithAppRuntime, type AppServices } from '../../../../lib/appLayer'
import { Effect, Stream } from 'effect'
import { Organizations, type Organization } from '../../services/Organizations'

const ORG_WRITE = 'dashboard.organizations.write'

const listEffect: Effect.Effect<readonly Organization[], Error, AppServices> =
  Effect.gen(function* () {
    const orgs = yield* Organizations
    return yield* orgs.list()
  })

type ListState = AsyncState<readonly Organization[], Error>

type OrganizationFormProps = {
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

function formatDate(iso: string | undefined) {
  if (iso == null || iso === '') return '—'
  try {
    return new Date(iso).toLocaleString()
  } catch {
    return iso
  }
}

export default function OrganizationsPage() {
  const theme = useTheme()
  const isMobile = useMediaQuery(theme.breakpoints.down('md'))
  const { permissions } = useAuthentication()
  const canWrite = permissions.includes(ORG_WRITE)
  const [liveRefreshTrigger, setLiveRefreshTrigger] = useEffectState(0)
  const { connected: wsConnected } = useLiveUpdates('organizations', () => {
    setLiveRefreshTrigger((n) => n + 1)
  })

  const { runtime } = useEffectRuntime<AppServices>()

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

  /** Run a stream and push each emission into a state setter. */
  const runStreamInto = (
    stream: Stream.Stream<ListState, never, AppServices>,
    setState: (s: ListState) => Effect.Effect<void, never, never>,
  ) => Stream.runForEach(stream, setState)

  const refreshStream = streamWithPendingState(listEffect)
  const refreshEffect = Effect.gen(function* () {
    yield* runStreamInto(refreshStream, setListStateAsEffect)
  })

  const [addDialogOpen, setAddDialogOpen, setAddDialogOpenAsEffect] =
    useEffectState(false)
  const [addName, setAddName, setAddNameAsEffect] = useEffectState('')
  const [addSlug, setAddSlug, setAddSlugAsEffect] = useEffectState('')
  const [filterName, setFilterName] = useEffectState('')
  const [filterSlug, setFilterSlug] = useEffectState('')
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

  const filteredOrganizations = organizations.filter((org) => {
    const nameMatch =
      !filterName.trim() ||
      org.name.toLowerCase().includes(filterName.trim().toLowerCase())
    const slugMatch =
      !filterSlug.trim() ||
      org.slug.toLowerCase().includes(filterSlug.trim().toLowerCase())
    return nameMatch && slugMatch
  })

  const errorMessage =
    error instanceof Error
      ? error.message
      : error != null
        ? String(error)
        : null

  const handleAdd = () => {
    const name = addName.trim()
    if (!name) return
    runWithAppRuntime(
      runtime,
      runStreamInto(
        streamWithPendingState(
          Effect.gen(function* () {
            const orgs = yield* Organizations
            yield* orgs.create({ name, slug: addSlug.trim() || undefined })
            return yield* orgs.list()
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
    runWithAppRuntime(
      runtime,
      runStreamInto(
        streamWithPendingState(
          Effect.gen(function* () {
            const orgs = yield* Organizations
            yield* orgs.update({
              id: editOrg.id,
              name: editName.trim() || undefined,
              slug: editSlug.trim() || undefined,
            })
            return yield* orgs.list()
          }),
        ),
        setUpdateStateAsEffect,
      ),
    )
  }

  const handleDelete = (id: string) => {
    setDeletingId(id)
    runWithAppRuntime(
      runtime,
      runStreamInto(
        streamWithPendingState(
          Effect.gen(function* () {
            const orgs = yield* Organizations
            yield* orgs.delete(id)
            return yield* orgs.list()
          }),
        ),
        setDeleteStateAsEffect,
      ),
    )
  }

  const handleClearError = () => {
    runWithAppRuntime(
      runtime,
      Effect.gen(function* () {
        yield* setCreateStateAsEffect(idle())
        yield* setUpdateStateAsEffect(idle())
        yield* setDeleteStateAsEffect(idle())
        yield* refreshEffect
      }),
    )
  }

  // Initial load and live-refresh: run stream into listState only
  useRunEffect(refreshEffect, [liveRefreshTrigger, setListStateAsEffect])

  // On create success: copy list to listState, close add dialog, reset create state
  useRunEffect(
    Effect.gen(function* () {
      if (!isSuccess(createState)) return
      yield* setListStateAsEffect(asyncSuccess(createState.value))
      yield* setAddDialogOpenAsEffect(false)
      yield* setAddNameAsEffect('')
      yield* setAddSlugAsEffect('')
      yield* setCreateStateAsEffect(idle())
    }),
    [
      createState,
      setListStateAsEffect,
      setAddDialogOpenAsEffect,
      setAddNameAsEffect,
      setAddSlugAsEffect,
      setCreateStateAsEffect,
    ],
  )

  // On update success: copy list to listState, close edit dialog, reset update state
  useRunEffect(
    Effect.gen(function* () {
      if (!isSuccess(updateState)) return
      yield* setListStateAsEffect(asyncSuccess(updateState.value))
      yield* setEditOrgAsEffect(null)
      yield* setUpdateStateAsEffect(idle())
    }),
    [
      updateState,
      setListStateAsEffect,
      setEditOrgAsEffect,
      setUpdateStateAsEffect,
    ],
  )

  // On delete success: copy list to listState, clear deletingId, reset delete state
  useRunEffect(
    Effect.gen(function* () {
      if (!isSuccess(deleteState)) return
      yield* setListStateAsEffect(asyncSuccess(deleteState.value))
      yield* setDeletingIdAsEffect(null)
      yield* setDeleteStateAsEffect(idle())
    }),
    [
      deleteState,
      setListStateAsEffect,
      setDeletingIdAsEffect,
      setDeleteStateAsEffect,
    ],
  )

  return (
    <>
      <Box sx={{ display: 'flex', alignItems: 'center', gap: 1, mb: 0 }}>
        <Typography variant="h4" component="h1" gutterBottom sx={{ mb: 0 }}>
          Organizations
        </Typography>
        {wsConnected && <Chip label="Live" color="success" size="small" />}
      </Box>
      <Typography variant="body2" color="text.secondary" sx={{ mb: 2 }}>
        View and manage organizations. Write actions require{' '}
        <code>dashboard.organizations.write</code>.
      </Typography>

      {errorMessage != null && (
        <Alert severity="error" sx={{ mb: 2 }} onClose={handleClearError}>
          {errorMessage}
        </Alert>
      )}

      <Paper sx={{ p: 2, mb: 2 }}>
        <Typography variant="subtitle2" gutterBottom>
          Filters
        </Typography>
        <Box
          sx={{
            display: 'flex',
            flexWrap: 'wrap',
            gap: 2,
            alignItems: 'flex-end',
          }}
        >
          <TextField
            label="Name"
            size="small"
            value={filterName}
            onChange={(e) => setFilterName(e.target.value)}
            placeholder="Search by name"
            sx={{ minWidth: 200 }}
          />
          <TextField
            label="Slug"
            size="small"
            value={filterSlug}
            onChange={(e) => setFilterSlug(e.target.value)}
            placeholder="Search by slug"
            sx={{ minWidth: 160 }}
          />
        </Box>
      </Paper>

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
        <Box sx={{ display: 'flex', justifyContent: 'center', py: 4 }}>
          <CircularProgress />
        </Box>
      ) : filteredOrganizations.length === 0 ? (
        <Paper sx={{ p: 3, textAlign: 'center' }}>
          <Typography color="text.secondary">
            {organizations.length === 0
              ? 'No organizations.'
              : 'No organizations match the filters.'}
          </Typography>
        </Paper>
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
              <Paper sx={{ p: 2 }}>
                <Typography variant="subtitle1" fontWeight={600}>
                  {org.name}
                </Typography>
                <Typography variant="body2" color="text.secondary">
                  {org.slug}
                </Typography>
                <Typography
                  variant="caption"
                  color="text.secondary"
                  display="block"
                  sx={{ mt: 0.5 }}
                >
                  Created {formatDate(org.created_at)} · Updated{' '}
                  {formatDate(org.updated_at)}
                </Typography>
                {canWrite && (
                  <Box
                    sx={{
                      mt: 2,
                      display: 'flex',
                      gap: 0.5,
                      justifyContent: 'flex-end',
                    }}
                  >
                    <Button
                      size="small"
                      startIcon={<EditIcon />}
                      onClick={() => handleOpenEdit(org)}
                    >
                      Edit
                    </Button>
                    <Button
                      size="small"
                      color="error"
                      startIcon={<DeleteIcon />}
                      onClick={() => handleDelete(org.id)}
                      disabled={isDeleting(org.id)}
                    >
                      Delete
                    </Button>
                  </Box>
                )}
              </Paper>
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
                <TableRow key={org.id}>
                  <TableCell sx={{ fontWeight: 500 }}>{org.name}</TableCell>
                  <TableCell>{org.slug}</TableCell>
                  <TableCell sx={{ whiteSpace: 'nowrap' }}>
                    {formatDate(org.created_at)}
                  </TableCell>
                  <TableCell sx={{ whiteSpace: 'nowrap' }}>
                    {formatDate(org.updated_at)}
                  </TableCell>
                  {canWrite && (
                    <TableCell align="right">
                      <IconButton
                        size="small"
                        aria-label="Edit"
                        onClick={() => handleOpenEdit(org)}
                      >
                        <EditIcon />
                      </IconButton>
                      <IconButton
                        size="small"
                        aria-label="Delete"
                        onClick={() => handleDelete(org.id)}
                        disabled={isDeleting(org.id)}
                      >
                        <DeleteIcon />
                      </IconButton>
                    </TableCell>
                  )}
                </TableRow>
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
