import { useState } from 'react'
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
import TableEmptyRow from '../../../../components/TableEmptyRow'
import RoleTableRow from '../../components/RoleTableRow'
import FormControl from '@mui/material/FormControl'
import InputLabel from '@mui/material/InputLabel'
import Select from '@mui/material/Select'
import MenuItem from '@mui/material/MenuItem'
import PageHeader from '../../../../components/PageHeader'
import ErrorAlert from '../../../../components/ErrorAlert'
import LoadingSpinner from '../../../../components/LoadingSpinner'
import { useLiveRefreshTrigger } from '../../../../hooks/useLiveRefreshTrigger'
import type { Role } from '../../domain/Role'
import type { Organization } from '../../domain/Organization'
import { Roles as RolesService } from '../../services/Roles'
import { Dashboard } from '../../services/Dashboard'

type ListState = AsyncState<readonly Role[], Error>

export default function RolesPage() {
  const { runtime } = useEffectRuntime<AppServices>()
  const { trigger: liveRefreshTrigger, connected: wsConnected } =
    useLiveRefreshTrigger('roles')

  const [listState, , setListStateAsEffect] = useEffectState<ListState, never, never>(
    idle<Role[], Error>(),
  )
  const [addState, , setAddStateAsEffect] = useEffectState<ListState, never, never>(
    idle<Role[], Error>(),
  )
  const [updateState, , setUpdateStateAsEffect] =
    useEffectState<ListState, never, never>(idle<Role[], Error>())
  const [deleteState, , setDeleteStateAsEffect] =
    useEffectState<ListState, never, never>(idle<Role[], Error>())

  const [orgListState, , setOrgListStateAsEffect] = useEffectState<
    AsyncState<readonly Organization[], Error>,
    never,
    never
  >(idle<readonly Organization[], Error>())

  const roles = isSuccess(listState) ? [...listState.value] : []
  const organizations = isSuccess(orgListState) ? [...orgListState.value] : []
  const loading = isPending(listState)
  const error: Error | null = isFailure(listState)
    ? listState.error
    : isFailure(addState)
      ? addState.error
      : isFailure(updateState)
        ? updateState.error
        : isFailure(deleteState)
          ? deleteState.error
          : null
  const errorMessage =
    error instanceof Error
      ? error.message
      : error != null
        ? String(error)
        : null

  const [addOpen, setAddOpen] = useState(false)
  const [addName, setAddName] = useState('')
  const [addDisplayName, setAddDisplayName] = useState('')
  const [addOrgId, setAddOrgId] = useState('')
  const [editRole, setEditRole] = useState<Role | null>(null)
  const [editName, setEditName] = useState('')
  const [editDisplayName, setEditDisplayName] = useState('')
  const [deletingId, setDeletingId] = useState<string | null>(null)

  const adding = isPending(addState)
  const saving = isPending(updateState)
  const deleting = isPending(deleteState)

  const listRolesEffect = Effect.gen(function* () {
    const roles = yield* RolesService
    return yield* roles.list()
  })
  const refreshStream = streamWithPendingState(listRolesEffect)
  const refreshEffect = Effect.gen(function* () {
    yield* runStreamInto(refreshStream, setListStateAsEffect)
  })

  useRunEffect(refreshEffect, [liveRefreshTrigger, setListStateAsEffect])

  const listOrgsEffect = Effect.gen(function* () {
    const dashboard = yield* Dashboard
    return yield* dashboard.getOrganizations()
  })
  const orgsStream = streamWithPendingState(listOrgsEffect)
  const orgsEffect = Effect.gen(function* () {
    yield* runStreamInto(orgsStream, setOrgListStateAsEffect)
  })

  useRunEffect(orgsEffect, [setOrgListStateAsEffect])

  // On add success: copy to listState, close add dialog, reset form, reset add state
  useRunEffect(
    Effect.gen(function* () {
      if (!isSuccess(addState)) return
      yield* setListStateAsEffect(asyncSuccess(addState.value))
      yield* Effect.sync(() => {
        setAddOpen(false)
        setAddName('')
        setAddDisplayName('')
        setAddOrgId(organizations[0]?.id ?? '')
      })
      yield* setAddStateAsEffect(idle())
    }),
    [addState, setListStateAsEffect, setAddStateAsEffect, organizations],
  )

  // On update success: copy to listState, close edit dialog, reset update state
  useRunEffect(
    Effect.gen(function* () {
      if (!isSuccess(updateState)) return
      yield* setListStateAsEffect(asyncSuccess(updateState.value))
      yield* Effect.sync(() => setEditRole(null))
      yield* setUpdateStateAsEffect(idle())
    }),
    [updateState, setListStateAsEffect, setUpdateStateAsEffect],
  )

  // On delete success: copy to listState, clear deletingId, reset delete state
  useRunEffect(
    Effect.gen(function* () {
      if (!isSuccess(deleteState)) return
      yield* setListStateAsEffect(asyncSuccess(deleteState.value))
      yield* Effect.sync(() => setDeletingId(null))
      yield* setDeleteStateAsEffect(idle())
    }),
    [deleteState, setListStateAsEffect, setDeleteStateAsEffect],
  )

  const handleAdd = () => {
    const name = addName.trim()
    if (!name) return
    const orgId = addOrgId || (organizations[0]?.id ?? '')
    const addThenList = Effect.gen(function* () {
      const roles = yield* RolesService
      yield* roles.create({
        name,
        display_name: addDisplayName.trim() || undefined,
        org_id: orgId,
      })
      return yield* roles.list()
    })
    runWithAppRuntime(
      runtime,
      runStreamInto(streamWithPendingState(addThenList), setAddStateAsEffect),
    )
  }

  const openEdit = (role: Role) => {
    setEditRole(role)
    setEditName(role.name)
    setEditDisplayName(role.display_name ?? '')
  }

  const handleSaveEdit = () => {
    if (!editRole) return
    const updateThenList = Effect.gen(function* () {
      const roles = yield* RolesService
      yield* roles.update({
        id: editRole.id,
        name: editName.trim() || undefined,
        display_name: editDisplayName.trim() || undefined,
      })
      return yield* roles.list()
    })
    runWithAppRuntime(
      runtime,
      runStreamInto(
        streamWithPendingState(updateThenList),
        setUpdateStateAsEffect,
      ),
    )
  }

  const handleDelete = (id: string) => {
    setDeletingId(id)
    const deleteThenList = Effect.gen(function* () {
      const roles = yield* RolesService
      yield* roles.delete(id)
      return yield* roles.list()
    })
    runWithAppRuntime(
      runtime,
      runStreamInto(
        streamWithPendingState(deleteThenList),
        setDeleteStateAsEffect,
      ),
    )
  }

  return (
    <>
      <PageHeader
        title="Roles"
        description="Manage organization roles. Template roles (owner, admin, editor, viewer) are created when an org is created; you can add custom roles here."
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

      <Box sx={{ mb: 2 }}>
        <Button
          variant="contained"
          startIcon={<AddIcon />}
          onClick={() => {
            setAddOrgId(organizations[0]?.id ?? '')
            setAddOpen(true)
          }}
        >
          Add role
        </Button>
      </Box>

      {loading ? (
        <LoadingSpinner />
      ) : (
        <TableContainer component={Paper}>
          <Table size="small" aria-label="Roles">
            <TableHead>
              <TableRow>
                <TableCell>Organization</TableCell>
                <TableCell>Name</TableCell>
                <TableCell>Display name</TableCell>
                <TableCell align="right">Actions</TableCell>
              </TableRow>
            </TableHead>
            <TableBody>
              {roles.length === 0 ? (
                <TableEmptyRow colSpan={4}>
                  No roles. Add a role or ensure your organization has template
                  roles.
                </TableEmptyRow>
              ) : (
                roles.map((role) => (
                  <RoleTableRow
                    key={role.id}
                    role={role}
                    orgName={
                      organizations.find((o) => o.id === role.org_id)?.name ??
                      role.org_id
                    }
                    onEdit={openEdit}
                    onDelete={handleDelete}
                    isDeleting={deleting && deletingId === role.id}
                  />
                ))
              )}
            </TableBody>
          </Table>
        </TableContainer>
      )}

      <FormDialog
        open={addOpen}
        onClose={() => setAddOpen(false)}
        title="Add role"
        submitLabel="Add"
        submittingLabel="Adding…"
        onSubmit={handleAdd}
        submitDisabled={
          !addName.trim() || (organizations.length > 1 && !addOrgId)
        }
        submitting={adding}
      >
        <Box
          sx={{
            display: 'flex',
            flexDirection: 'column',
            gap: 2,
            pt: 1,
            minWidth: 320,
          }}
        >
          {organizations.length > 1 && (
            <FormControl fullWidth size="small" disabled={adding} required>
              <InputLabel>Organization</InputLabel>
              <Select
                label="Organization"
                value={addOrgId}
                onChange={(e) => setAddOrgId(e.target.value)}
              >
                {organizations.map((org) => (
                  <MenuItem key={org.id} value={org.id}>
                    {org.name}
                  </MenuItem>
                ))}
              </Select>
            </FormControl>
          )}
          <TextField
            label="Name"
            value={addName}
            onChange={(e) => setAddName(e.target.value)}
            placeholder="e.g. writer"
            required
            disabled={adding}
            fullWidth
          />
          <TextField
            label="Display name (optional)"
            value={addDisplayName}
            onChange={(e) => setAddDisplayName(e.target.value)}
            placeholder="e.g. Content writer"
            disabled={adding}
            fullWidth
          />
        </Box>
      </FormDialog>

      <FormDialog
        open={Boolean(editRole)}
        onClose={() => setEditRole(null)}
        title="Edit role"
        submitLabel="Save"
        submittingLabel="Saving…"
        onSubmit={handleSaveEdit}
        submitDisabled={!editName.trim()}
        submitting={saving}
      >
        <Box
          sx={{
            display: 'flex',
            flexDirection: 'column',
            gap: 2,
            pt: 1,
            minWidth: 320,
          }}
        >
          <TextField
            label="Name"
            value={editName}
            onChange={(e) => setEditName(e.target.value)}
            required
            disabled={saving}
            fullWidth
          />
          <TextField
            label="Display name (optional)"
            value={editDisplayName}
            onChange={(e) => setEditDisplayName(e.target.value)}
            disabled={saving}
            fullWidth
          />
        </Box>
      </FormDialog>
    </>
  )
}
