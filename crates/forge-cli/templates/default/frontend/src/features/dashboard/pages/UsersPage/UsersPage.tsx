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
import { useEffect, useState } from 'react'
import { useForm, Controller } from 'react-hook-form'
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
import FormControl from '@mui/material/FormControl'
import InputLabel from '@mui/material/InputLabel'
import Select from '@mui/material/Select'
import MenuItem from '@mui/material/MenuItem'
import Alert from '@mui/material/Alert'
import FormControlLabel from '@mui/material/FormControlLabel'
import Checkbox from '@mui/material/Checkbox'
import { Effect, Schema } from 'effect'

import FormDialog from '@/components/FormDialog'
import PageHeader from '@/components/PageHeader'
import TableEmptyRow from '@/components/TableEmptyRow'
import UsersFilters from '@/features/dashboard/components/UsersFilters'
import UserTableRow from '@/features/dashboard/components/UserTableRow'
import ErrorAlert from '@/components/ErrorAlert'
import LoadingSpinner from '@/components/LoadingSpinner'
import { ShowWithPermissions } from '@/components/ShowWithPermissions'
import { useLiveRefreshTrigger } from '@/hooks/useLiveRefreshTrigger'
import { usePermission } from '@/hooks/usePermission'
import { getApplicationLayer } from '@/lib/appLayer'
import { effectSchemaResolver } from '@/lib/effectSchemaResolver'
import {
  userAddFormSchemaStrict,
  userEditFormSchema,
  type UserAddFormValuesStrict,
  type UserEditFormValues,
} from '@/schemas/userFormSchemas'
import type { User } from '@/features/dashboard/domain/User'
import type { Organization } from '@/features/dashboard/domain/Organization'
import type { DashboardRole } from '@/features/dashboard/domain/DashboardRole'
import { Users } from '@/features/dashboard/services/Users'
import { Dashboard } from '@/features/dashboard/services/Dashboard'

const USERS_READ = 'dashboard.users.read'
const USERS_WRITE = 'dashboard.users.write'
const FILTER_ROLES = ['owner', 'admin', 'editor', 'viewer']

export default function UsersPage() {
  const canRead = usePermission(USERS_READ)
  const canWrite = usePermission(USERS_WRITE)
  const { trigger: liveRefreshTrigger, connected: wsConnected } =
    useLiveRefreshTrigger('users')

  const [listState, setListState, setListStateAsEffect] = useEffectState<
    AsyncState<readonly User[], Error>
  >(idle())
  const [addState, _setAddState, setAddStateAsEffect] = useEffectState<
    AsyncState<readonly User[], Error>
  >(idle())
  const [updateState, _setUpdateState, setUpdateStateAsEffect] = useEffectState<
    AsyncState<readonly User[], Error>
  >(idle())
  const [deleteState, _setDeleteState, setDeleteStateAsEffect] = useEffectState<
    AsyncState<readonly User[], Error>
  >(idle())
  const [orgListState, setOrgListState, setOrgListStateAsEffect] =
    useEffectState<AsyncState<readonly Organization[], Error>>(idle())
  const [orgRoles, _setOrgRoles, setOrgRolesAsEffect] = useEffectState<
    readonly DashboardRole[]
  >([])

  const [addDialogOpen, setAddDialogOpen] = useState(false)
  const [filterEmail, setFilterEmail] = useState('')
  const [filterOrgId, setFilterOrgId] = useState('')
  const [filterRole, setFilterRole] = useState('')
  const [filterActive, setFilterActive] = useState<'' | 'yes' | 'no'>('')
  const [filterAdmin, setFilterAdmin] = useState<'' | 'yes' | 'no'>('')
  const [editUser, setEditUser] = useState<User | null>(null)
  const [deletingId, setDeletingId] = useState<string | null>(null)

  const users = listState._tag === 'success' ? [...listState.value] : []
  const organizations =
    orgListState._tag === 'success' ? [...orgListState.value] : []
  const loading = isPending(listState)
  const adding = isPending(addState)
  const saving = isPending(updateState)
  const deleting = isPending(deleteState)
  const errorMessage =
    [listState, addState, updateState, deleteState].filter(
      (s): s is AsyncState<readonly User[], Error> & { _tag: 'failure' } =>
        isFailure(s),
    )[0]?.error?.message ?? null
  const error = errorMessage != null ? errorMessage : null

  const addForm = useForm<UserAddFormValuesStrict>({
    resolver: effectSchemaResolver(
      userAddFormSchemaStrict as Schema.Schema<
        UserAddFormValuesStrict,
        unknown,
        never
      >,
    ),
    defaultValues: { email: '', password: '', orgId: '', roleIds: [] },
    mode: 'onChange',
  })

  const editForm = useForm<UserEditFormValues>({
    resolver: effectSchemaResolver(
      userEditFormSchema as Schema.Schema<UserEditFormValues, unknown, never>,
    ),
    defaultValues: { email: '', active: true },
    mode: 'onChange',
  })

  const addFormOrgId = addForm.watch('orgId')

  const listUsersEffect = Effect.gen(function* () {
    const users = yield* Users
    return yield* users.list()
  })
  const refreshEffect = streamWithPendingState(
    canRead ? listUsersEffect : Effect.succeed([] as User[]),
  )
  useEffect(() => {
    const effect = canRead
      ? runStreamInto(refreshEffect, setListStateAsEffect)
      : Effect.sync(() => setListState(idle()))
    Effect.runPromise(effect.pipe(Effect.provide(getApplicationLayer())))
  }, [liveRefreshTrigger, setListStateAsEffect, canRead])

  const listOrgsEffect = Effect.gen(function* () {
    const dashboard = yield* Dashboard
    return yield* dashboard.getOrganizations()
  })
  useEffect(() => {
    const effect = canRead
      ? runStreamInto(
          streamWithPendingState(listOrgsEffect),
          setOrgListStateAsEffect,
        )
      : Effect.sync(() => setOrgListState(idle()))
    Effect.runPromise(effect.pipe(Effect.provide(getApplicationLayer())))
  }, [setOrgListStateAsEffect, canRead])

  useEffect(() => {
    const effect = addFormOrgId
      ? Effect.gen(function* () {
          const dashboard = yield* Dashboard
          const roles = yield* dashboard.getRolesByOrg(addFormOrgId)
          yield* setOrgRolesAsEffect(roles)
        })
      : setOrgRolesAsEffect([])
    Effect.runPromise(effect.pipe(Effect.provide(getApplicationLayer())))
  }, [addFormOrgId, setOrgRolesAsEffect])

  useEffect(() => {
    const current = addForm.getValues('roleIds')
    const valid = current.filter((id) => orgRoles.some((r) => r.id === id))
    if (valid.length !== current.length) {
      addForm.setValue('roleIds', valid)
    }
  }, [orgRoles, addForm])

  const createThenList = (data: UserAddFormValuesStrict) =>
    Effect.gen(function* () {
      const users = yield* Users
      yield* users.create({
        email: data.email,
        password: data.password,
        org_id: data.orgId,
        role_ids: [...data.roleIds],
      })
      return yield* users.list()
    })

  const handleAdd = (data: UserAddFormValuesStrict) => {
    Effect.runPromise(
      runStreamInto(
        streamWithPendingState(createThenList(data)),
        setAddStateAsEffect,
      ).pipe(Effect.provide(getApplicationLayer())),
    )
  }

  const openEdit = (user: User) => {
    setEditUser(user)
    editForm.reset({ email: user.email, active: user.is_active })
  }

  const updateThenList = (data: UserEditFormValues) =>
    Effect.gen(function* () {
      const users = yield* Users
      if (!editUser) return yield* users.list()
      yield* users.update({
        id: editUser.id,
        email: data.email.trim() || undefined,
        is_active: data.active,
      })
      return yield* users.list()
    })

  const handleSaveEdit = (data: UserEditFormValues) => {
    if (!editUser) return
    Effect.runPromise(
      runStreamInto(
        streamWithPendingState(updateThenList(data)),
        setUpdateStateAsEffect,
      ).pipe(Effect.provide(getApplicationLayer())),
    )
  }

  const deleteThenList = (id: string) =>
    Effect.gen(function* () {
      const users = yield* Users
      yield* users.delete(id)
      return yield* users.list()
    })

  const handleDelete = (id: string) => {
    setDeletingId(id)
    Effect.runPromise(
      runStreamInto(
        streamWithPendingState(deleteThenList(id)),
        setDeleteStateAsEffect,
      ).pipe(Effect.provide(getApplicationLayer())),
    )
  }

  useEffect(() => {
    if (!isSuccess(addState)) return
    Effect.runPromise(
      Effect.gen(function* () {
        yield* setListStateAsEffect(asyncSuccess(addState.value))
        yield* Effect.sync(() => {
          addForm.reset({
            email: '',
            password: '',
            orgId: organizations[0]?.id ?? '',
            roleIds: [],
          })
          setAddDialogOpen(false)
        })
        yield* setAddStateAsEffect(idle())
      }).pipe(Effect.provide(getApplicationLayer())),
    )
  }, [addState])

  useEffect(() => {
    if (!isSuccess(updateState)) return
    Effect.runPromise(
      Effect.gen(function* () {
        yield* setListStateAsEffect(asyncSuccess(updateState.value))
        yield* Effect.sync(() => setEditUser(null))
        yield* setUpdateStateAsEffect(idle())
      }).pipe(Effect.provide(getApplicationLayer())),
    )
  }, [updateState])

  useEffect(() => {
    if (!isSuccess(deleteState)) return
    Effect.runPromise(
      Effect.gen(function* () {
        yield* setListStateAsEffect(asyncSuccess(deleteState.value))
        yield* Effect.sync(() => setDeletingId(null))
        yield* setDeleteStateAsEffect(idle())
      }).pipe(Effect.provide(getApplicationLayer())),
    )
  }, [deleteState])

  const filteredUsers = users.filter((user) => {
    const emailMatch =
      !filterEmail.trim() ||
      user.email.toLowerCase().includes(filterEmail.trim().toLowerCase())
    const orgMatch =
      !filterOrgId || user.memberships.some((m) => m.org_id === filterOrgId)
    const roleMatch =
      !filterRole ||
      user.memberships.some((m) =>
        (m.roles ?? []).some(
          (r) => r.toLowerCase() === filterRole.toLowerCase(),
        ),
      )
    const activeMatch =
      filterActive === '' ||
      (filterActive === 'yes' && user.is_active) ||
      (filterActive === 'no' && !user.is_active)
    const adminMatch =
      filterAdmin === '' ||
      (filterAdmin === 'yes' && user.is_admin) ||
      (filterAdmin === 'no' && !user.is_admin)
    return emailMatch && orgMatch && roleMatch && activeMatch && adminMatch
  })

  if (!canRead) {
    return (
      <>
        <PageHeader title="Users" />
        <Alert severity="info">You do not have permission to view users.</Alert>
      </>
    )
  }

  return (
    <Box data-testid="users-page">
      <PageHeader
        title="Users"
        description={
          <>
            View and manage users. Write actions require{' '}
            <code>dashboard.users.write</code>.
          </>
        }
        liveConnected={wsConnected}
        data-testid="users-page-title"
      />

      {error != null && (
        <ErrorAlert
          message={error}
          onClose={() => {
            Effect.runPromise(
              Effect.gen(function* () {
                yield* runStreamInto(refreshEffect, setListStateAsEffect)
                yield* setAddStateAsEffect(idle())
                yield* setUpdateStateAsEffect(idle())
                yield* setDeleteStateAsEffect(idle())
              }).pipe(Effect.provide(getApplicationLayer())),
            )
          }}
        />
      )}

      <UsersFilters
        filterEmail={filterEmail}
        filterOrgId={filterOrgId}
        filterRole={filterRole}
        filterActive={filterActive}
        filterAdmin={filterAdmin}
        organizations={organizations}
        roleOptions={FILTER_ROLES}
        onFilterEmailChange={setFilterEmail}
        onFilterOrgIdChange={setFilterOrgId}
        onFilterRoleChange={setFilterRole}
        onFilterActiveChange={setFilterActive}
        onFilterAdminChange={setFilterAdmin}
      />

      <ShowWithPermissions permissions={[USERS_WRITE]}>
        <Box sx={{ mb: 2 }}>
          <Button
            variant="contained"
            startIcon={<AddIcon />}
            onClick={() => {
              addForm.reset({
                email: '',
                password: '',
                orgId: organizations[0]?.id ?? '',
                roleIds: [],
              })
              setAddDialogOpen(true)
            }}
            data-testid="users-create-button"
          >
            Add user
          </Button>
        </Box>
      </ShowWithPermissions>

      {loading ? (
        <LoadingSpinner />
      ) : (
        <TableContainer component={Paper} data-testid="users-list">
          <Table size="small" aria-label="Users" data-testid="users-table">
            <TableHead>
              <TableRow>
                <TableCell>Email</TableCell>
                <TableCell>Organizations / Role</TableCell>
                <TableCell>Active</TableCell>
                <TableCell>Admin</TableCell>
                <TableCell>Created</TableCell>
                {canWrite && <TableCell align="right">Actions</TableCell>}
              </TableRow>
            </TableHead>
            <TableBody>
              {filteredUsers.length === 0 ? (
                <TableEmptyRow colSpan={canWrite ? 6 : 5}>
                  {users.length === 0
                    ? 'No users.'
                    : 'No users match the filters.'}
                </TableEmptyRow>
              ) : (
                filteredUsers.map((user) => (
                  <UserTableRow
                    key={user.id}
                    user={user}
                    canWrite={canWrite}
                    onEdit={(u) => openEdit(u as User)}
                    onDelete={handleDelete}
                    isDeleting={deleting && deletingId === user.id}
                  />
                ))
              )}
            </TableBody>
          </Table>
        </TableContainer>
      )}

      <FormDialog
        open={addDialogOpen}
        onClose={() => setAddDialogOpen(false)}
        title="Add user"
        submitLabel="Add"
        submittingLabel="Adding…"
        onSubmit={() => addForm.handleSubmit(handleAdd)()}
        submitDisabled={!addForm.formState.isValid}
        submitting={adding}
        data-testid="user-create-dialog"
        submitButtonTestId="user-create-submit-button"
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
          data-testid="user-create-form"
        >
          <Controller
            control={addForm.control}
            name="email"
            render={({ field, fieldState }) => (
              <TextField
                {...field}
                label="Email"
                type="email"
                required
                disabled={adding}
                error={Boolean(fieldState.error)}
                helperText={fieldState.error?.message}
                data-testid="user-create-email-input"
              />
            )}
          />
          <Controller
            control={addForm.control}
            name="password"
            render={({ field, fieldState }) => (
              <TextField
                {...field}
                label="Password"
                type="password"
                placeholder="Min 8 characters"
                disabled={adding}
                error={Boolean(fieldState.error)}
                helperText={fieldState.error?.message}
                data-testid="user-create-password-input"
              />
            )}
          />
          <Controller
            control={addForm.control}
            name="orgId"
            render={({ field, fieldState }) => (
              <FormControl
                fullWidth
                size="small"
                disabled={adding}
                error={Boolean(fieldState.error)}
              >
                <InputLabel>Organization</InputLabel>
                <Select
                  {...field}
                  label="Organization"
                  onChange={(e) => field.onChange(e.target.value)}
                >
                  {organizations.map((org) => (
                    <MenuItem key={org.id} value={org.id}>
                      {org.name}
                    </MenuItem>
                  ))}
                </Select>
                {fieldState.error?.message && (
                  <Box
                    component="span"
                    sx={{
                      color: 'error.main',
                      fontSize: '0.75rem',
                      mt: 0.5,
                      display: 'block',
                    }}
                  >
                    {fieldState.error.message}
                  </Box>
                )}
              </FormControl>
            )}
          />
          <Controller
            control={addForm.control}
            name="roleIds"
            render={({ field, fieldState }) => (
              <FormControl
                fullWidth
                size="small"
                disabled={adding}
                error={Boolean(fieldState.error)}
              >
                <InputLabel>Roles</InputLabel>
                <Select
                  {...field}
                  label="Roles"
                  multiple
                  onChange={(e) =>
                    field.onChange([].slice.call(e.target.value))
                  }
                  renderValue={(selected) =>
                    (selected as string[])
                      .map(
                        (id) => orgRoles.find((r) => r.id === id)?.name ?? id,
                      )
                      .join(', ')
                  }
                >
                  {orgRoles.map((r) => (
                    <MenuItem key={r.id} value={r.id}>
                      {r.display_name || r.name}
                    </MenuItem>
                  ))}
                </Select>
                {fieldState.error?.message && (
                  <Box
                    component="span"
                    sx={{
                      color: 'error.main',
                      fontSize: '0.75rem',
                      mt: 0.5,
                      display: 'block',
                    }}
                  >
                    {fieldState.error.message}
                  </Box>
                )}
              </FormControl>
            )}
          />
        </Box>
      </FormDialog>

      <FormDialog
        open={Boolean(editUser)}
        onClose={() => setEditUser(null)}
        title="Edit user"
        submitLabel="Save"
        submittingLabel="Saving…"
        onSubmit={() => editForm.handleSubmit(handleSaveEdit)()}
        submitDisabled={!editForm.formState.isValid}
        submitting={saving}
        data-testid="user-edit-dialog"
        submitButtonTestId="user-edit-submit-button"
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
          data-testid="user-edit-form"
        >
          <Controller
            control={editForm.control}
            name="email"
            render={({ field, fieldState }) => (
              <TextField
                {...field}
                label="Email"
                type="email"
                disabled={saving}
                error={Boolean(fieldState.error)}
                helperText={fieldState.error?.message}
                data-testid="user-edit-email-input"
              />
            )}
          />
          <Controller
            control={editForm.control}
            name="active"
            render={({ field }) => (
              <FormControlLabel
                control={
                  <Checkbox
                    checked={field.value}
                    onChange={(e) => field.onChange(e.target.checked)}
                    disabled={saving}
                  />
                }
                label="Active"
              />
            )}
          />
        </Box>
      </FormDialog>
    </Box>
  )
}
