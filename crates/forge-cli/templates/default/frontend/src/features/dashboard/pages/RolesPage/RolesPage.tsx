import { useCallback, useEffect, useMemo, useState } from 'react'
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
import Box from '@mui/material/Box'
import Button from '@mui/material/Button'
import TextField from '@mui/material/TextField'
import AddIcon from '@mui/icons-material/Add'
import IconButton from '@mui/material/IconButton'
import VisibilityIcon from '@mui/icons-material/Visibility'
import EditIcon from '@mui/icons-material/Edit'
import DeleteIcon from '@mui/icons-material/Delete'
import FormControl from '@mui/material/FormControl'
import InputLabel from '@mui/material/InputLabel'
import Select from '@mui/material/Select'
import MenuItem from '@mui/material/MenuItem'
import Dialog from '@mui/material/Dialog'
import DialogActions from '@mui/material/DialogActions'
import DialogContent from '@mui/material/DialogContent'
import DialogTitle from '@mui/material/DialogTitle'
import Typography from '@mui/material/Typography'

import { getApplicationLayer } from '../../../../lib/appLayer'
import FormDialog from '../../../../components/FormDialog'
import ActionBar from '../../../../components/ActionBar'
import DataTable from '../../../../components/DataTable'
import type { DataTableColumn } from '../../../../components/DataTable'
import DataListMobile from '../../../../components/DataListMobile'
import EmptyState from '../../../../components/EmptyState'
import LoadingSpinner from '../../../../components/LoadingSpinner'
import PageHeader from '../../../../components/PageHeader'
import RoleCard from '../../../../features/dashboard/components/RoleCard'
import RolesFilters from '../../../../features/dashboard/components/RolesFilters'
import ErrorAlert from '../../../../components/ErrorAlert'
import { ShowWithPermissions } from '../../../../components/ShowWithPermissions'
import { useIsMobile } from '../../../../hooks/useIsMobile'
import { useLiveRefreshTrigger } from '../../../../hooks/useLiveRefreshTrigger'
import { usePermission } from '../../../../hooks/usePermission'
import { useTablePaginationDefaults } from '../../../../hooks/useTablePaginationDefaults'
import { useRolesFilter } from '../../../../features/dashboard/hooks/useRolesFilter'
import type { Role } from '../../../../features/dashboard/domain/Role'
import type { Organization } from '../../../../features/dashboard/domain/Organization'
import { Roles as RolesService } from '../../../../features/dashboard/services/Roles'
import { Dashboard } from '../../../../features/dashboard/services/Dashboard'

type ListState = AsyncState<readonly Role[], Error>

const ROLES_WRITE = 'role.create'

export default function RolesPage() {
  const isMobile = useIsMobile()
  const canWrite = usePermission(ROLES_WRITE)
  const { trigger: _liveRefreshTrigger, connected: wsConnected } =
    useLiveRefreshTrigger('roles')

  const [listState, , setListStateAsEffect] = useEffectState<
    ListState,
    never,
    never
  >(idle<Role[], Error>())
  const [addState, , setAddStateAsEffect] = useEffectState<
    ListState,
    never,
    never
  >(idle<Role[], Error>())
  const [updateState, , setUpdateStateAsEffect] = useEffectState<
    ListState,
    never,
    never
  >(idle<Role[], Error>())
  const [deleteState, , setDeleteStateAsEffect] = useEffectState<
    ListState,
    never,
    never
  >(idle<Role[], Error>())

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
  const [viewRole, setViewRole] = useState<Role | null>(null)
  const [editName, setEditName] = useState('')
  const [editDisplayName, setEditDisplayName] = useState('')
  const [deletingId, setDeletingId] = useState<string | null>(null)

  const [filters, setFilters, filteredRoles] = useRolesFilter(
    roles,
    organizations,
  )

  const { defaultRowsPerPage, rowsPerPageOptions } =
    useTablePaginationDefaults()
  const [page, setPage] = useState(0)
  const [rowsPerPage, setRowsPerPage] = useState(defaultRowsPerPage)
  useEffect(() => {
    setRowsPerPage((prev) =>
      rowsPerPageOptions.includes(prev) ? prev : defaultRowsPerPage,
    )
  }, [defaultRowsPerPage, rowsPerPageOptions])
  useEffect(() => setPage(0), [])

  const paginatedRoles = useMemo(
    () =>
      filteredRoles.slice(page * rowsPerPage, page * rowsPerPage + rowsPerPage),
    [filteredRoles, page, rowsPerPage],
  )

  const getOrgName = useCallback(
    (role: Role) =>
      role.org_name ??
      organizations.find((o) => o.id === role.org_id)?.name ??
      role.org_id,
    [organizations],
  )

  const roleColumns: DataTableColumn<Role>[] = useMemo(
    () => [
      {
        id: 'organization',
        label: 'Organization',
        render: (r) => getOrgName(r),
      },
      {
        id: 'name',
        label: 'Name',
        render: (r) => r.display_name ?? r.name ?? '—',
      },
    ],
    [getOrgName],
  )

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

  useEffect(() => {
    Effect.runPromise(refreshEffect.pipe(Effect.provide(getApplicationLayer())))
  }, [refreshEffect.pipe])

  const listOrgsEffect = Effect.gen(function* () {
    const dashboard = yield* Dashboard
    return yield* dashboard.getOrganizations()
  })
  const orgsStream = streamWithPendingState(listOrgsEffect)
  const orgsEffect = Effect.gen(function* () {
    yield* runStreamInto(orgsStream, setOrgListStateAsEffect)
  })

  useEffect(() => {
    Effect.runPromise(orgsEffect.pipe(Effect.provide(getApplicationLayer())))
  }, [orgsEffect.pipe])

  // On add success: copy to listState, close add dialog, reset form, reset add state
  useEffect(() => {
    if (!isSuccess(addState)) {
      return
    }
    Effect.runPromise(
      Effect.gen(function* () {
        yield* setListStateAsEffect(asyncSuccess(addState.value))
        yield* Effect.sync(() => {
          setAddOpen(false)
          setAddName('')
          setAddDisplayName('')
          setAddOrgId(organizations[0]?.id ?? '')
        })
        yield* setAddStateAsEffect(idle())
      }).pipe(Effect.provide(getApplicationLayer())),
    )
  }, [addState, setListStateAsEffect, setAddStateAsEffect, organizations])

  // On update success: copy to listState, close edit dialog, reset update state
  useEffect(() => {
    if (!isSuccess(updateState)) {
      return
    }
    Effect.runPromise(
      Effect.gen(function* () {
        yield* setListStateAsEffect(asyncSuccess(updateState.value))
        yield* Effect.sync(() => setEditRole(null))
        yield* setUpdateStateAsEffect(idle())
      }).pipe(Effect.provide(getApplicationLayer())),
    )
  }, [updateState, setListStateAsEffect, setUpdateStateAsEffect])

  // On delete success: copy to listState, clear deletingId, reset delete state
  useEffect(() => {
    if (!isSuccess(deleteState)) {
      return
    }
    Effect.runPromise(
      Effect.gen(function* () {
        yield* setListStateAsEffect(asyncSuccess(deleteState.value))
        yield* Effect.sync(() => setDeletingId(null))
        yield* setDeleteStateAsEffect(idle())
      }).pipe(Effect.provide(getApplicationLayer())),
    )
  }, [deleteState, setListStateAsEffect, setDeleteStateAsEffect])

  const handleAdd = () => {
    const name = addName.trim()
    if (!name) {
      return
    }
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
    Effect.runPromise(
      runStreamInto(
        streamWithPendingState(addThenList),
        setAddStateAsEffect,
      ).pipe(Effect.provide(getApplicationLayer())),
    )
  }

  const openEdit = (role: Role) => {
    setEditRole(role)
    setEditName(role.name)
    setEditDisplayName(role.display_name ?? '')
  }

  const handleSaveEdit = () => {
    if (!editRole) {
      return
    }
    const updateThenList = Effect.gen(function* () {
      const roles = yield* RolesService
      yield* roles.update({
        id: editRole.id,
        name: editName.trim() || undefined,
        display_name: editDisplayName.trim() || undefined,
      })
      return yield* roles.list()
    })
    Effect.runPromise(
      runStreamInto(
        streamWithPendingState(updateThenList),
        setUpdateStateAsEffect,
      ).pipe(Effect.provide(getApplicationLayer())),
    )
  }

  const handleDelete = (id: string) => {
    setDeletingId(id)
    const deleteThenList = Effect.gen(function* () {
      const roles = yield* RolesService
      yield* roles.delete(id)
      return yield* roles.list()
    })
    Effect.runPromise(
      runStreamInto(
        streamWithPendingState(deleteThenList),
        setDeleteStateAsEffect,
      ).pipe(Effect.provide(getApplicationLayer())),
    )
  }

  return (
    <Box data-testid="roles-page">
      <PageHeader
        title="Roles"
        description="Manage organization roles. Template roles (owner, admin, editor, viewer) are created when an org is created; you can add custom roles here."
        liveConnected={wsConnected}
        data-testid="roles-page-title"
      />

      {errorMessage != null && (
        <ErrorAlert
          message={errorMessage}
          onClose={() => {
            Effect.runPromise(
              refreshEffect.pipe(Effect.provide(getApplicationLayer())),
            )
          }}
        />
      )}

      <RolesFilters
        filterName={filters.filterName}
        filterDisplayName={filters.filterDisplayName}
        filterOrg={filters.filterOrg}
        onFilterNameChange={(value) =>
          setFilters((prev) => ({ ...prev, filterName: value }))
        }
        onFilterDisplayNameChange={(value) =>
          setFilters((prev) => ({ ...prev, filterDisplayName: value }))
        }
        onFilterOrgChange={(value) =>
          setFilters((prev) => ({ ...prev, filterOrg: value }))
        }
      />

      <ShowWithPermissions permissions={[ROLES_WRITE]}>
        <ActionBar>
          <Button
            variant="contained"
            startIcon={<AddIcon />}
            onClick={() => {
              setAddOrgId(organizations[0]?.id ?? '')
              setAddOpen(true)
            }}
            data-testid="roles-create-button"
          >
            Add role
          </Button>
        </ActionBar>
      </ShowWithPermissions>

      {loading ? (
        <LoadingSpinner />
      ) : filteredRoles.length === 0 ? (
        <EmptyState
          message={
            roles.length === 0
              ? 'No roles. Add a role or ensure your organization has template roles.'
              : 'No roles match the filters.'
          }
        />
      ) : isMobile ? (
        <DataListMobile<Role>
          items={paginatedRoles}
          getKey={(r) => r.id}
          renderItem={(role) => (
            <RoleCard
              role={role}
              organizations={organizations}
              canWrite={canWrite}
              onView={setViewRole}
              onEdit={openEdit}
              onDelete={handleDelete}
              isDeleting={deleting && deletingId === role.id}
            />
          )}
          pagination={{
            page,
            rowsPerPage,
            totalCount: filteredRoles.length,
            onPageChange: (_ev, newPage) => setPage(newPage),
            onRowsPerPageChange: (ev) => {
              setRowsPerPage(parseInt(ev.target.value, 10))
              setPage(0)
            },
            rowsPerPageOptions,
          }}
          ariaLabel="Roles"
          dataTestId="roles-list-mobile"
        />
      ) : (
        <DataTable<Role>
          columns={roleColumns}
          rows={paginatedRoles}
          loading={false}
          getRowId={(r) => r.id}
          emptyMessage={
            roles.length === 0
              ? 'No roles. Add a role or ensure your organization has template roles.'
              : 'No roles match the filters.'
          }
          pagination={{
            page,
            rowsPerPage,
            totalCount: filteredRoles.length,
            onPageChange: (_ev, newPage) => setPage(newPage),
            onRowsPerPageChange: (ev) => {
              setRowsPerPage(parseInt(ev.target.value, 10))
              setPage(0)
            },
            rowsPerPageOptions,
          }}
          actionsColumn={{
            canShow: true,
            render: (role) => (
              <>
                <IconButton
                  size="small"
                  color="primary"
                  aria-label="View"
                  onClick={() => setViewRole(role)}
                  data-testid={`role-row-view-${role.id}`}
                >
                  <VisibilityIcon />
                </IconButton>
                {canWrite && (
                  <>
                    <IconButton
                      size="small"
                      color="primary"
                      aria-label="Edit"
                      onClick={() => openEdit(role)}
                      data-testid={`role-row-edit-${role.id}`}
                    >
                      <EditIcon />
                    </IconButton>
                    <IconButton
                      size="small"
                      color="error"
                      aria-label="Delete"
                      onClick={() => handleDelete(role.id)}
                      disabled={deleting && deletingId === role.id}
                      data-testid={`role-row-delete-${role.id}`}
                    >
                      <DeleteIcon />
                    </IconButton>
                  </>
                )}
              </>
            ),
          }}
          ariaLabel="Roles"
          dataTestId="roles-list"
        />
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
        data-testid="role-create-dialog"
        submitButtonTestId="role-create-submit-button"
      >
        <Box
          sx={{
            display: 'flex',
            flexDirection: 'column',
            gap: 2,
            pt: 1,
            minWidth: 320,
          }}
          component="form"
          data-testid="role-create-form"
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
            data-testid="role-create-name-input"
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
        data-testid="role-edit-dialog"
        submitButtonTestId="role-edit-submit-button"
      >
        <Box
          sx={{
            display: 'flex',
            flexDirection: 'column',
            gap: 2,
            pt: 1,
            minWidth: 320,
          }}
          component="form"
          data-testid="role-edit-form"
        >
          <TextField
            label="Name"
            value={editName}
            onChange={(e) => setEditName(e.target.value)}
            required
            disabled={saving}
            fullWidth
            data-testid="role-edit-name-input"
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

      <Dialog
        open={Boolean(viewRole)}
        onClose={() => setViewRole(null)}
        maxWidth="sm"
        fullWidth
        data-testid="role-details-dialog"
      >
        <DialogTitle>Role Details</DialogTitle>
        <DialogContent>
          {viewRole && (
            <Box
              sx={{ display: 'flex', flexDirection: 'column', gap: 2, pt: 1 }}
            >
              <Box>
                <Typography variant="subtitle2" color="text.secondary">
                  Organization
                </Typography>
                <Typography variant="body1">
                  {viewRole.org_name ??
                    organizations.find((o) => o.id === viewRole.org_id)?.name ??
                    viewRole.org_id}
                </Typography>
              </Box>
              <Box>
                <Typography variant="subtitle2" color="text.secondary">
                  Name
                </Typography>
                <Typography variant="body1">{viewRole.name}</Typography>
              </Box>
              <Box>
                <Typography variant="subtitle2" color="text.secondary">
                  Display Name
                </Typography>
                <Typography variant="body1">
                  {viewRole.display_name ?? viewRole.name ?? '—'}
                </Typography>
              </Box>
            </Box>
          )}
        </DialogContent>
        <DialogActions>
          <Button onClick={() => setViewRole(null)}>Close</Button>
        </DialogActions>
      </Dialog>
    </Box>
  )
}
