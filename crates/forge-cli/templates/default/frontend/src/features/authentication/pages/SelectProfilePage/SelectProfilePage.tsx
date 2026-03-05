import { Navigate, useLocation, useNavigate } from 'react-router-dom'
import Box from '@mui/material/Box'
import Card from '@mui/material/Card'
import CardActionArea from '@mui/material/CardActionArea'
import CardContent from '@mui/material/CardContent'
import Typography from '@mui/material/Typography'
import { Effect } from 'effect'
import { useCallback } from 'react'
import { AuthenticationStore } from '../../services/AuthenticationStore'
import { useAuthenticationState } from '../../hooks/useAuthentication'
import {
  useEffectState,
  useRunEffect,
  useEffectRuntime,
  streamWithPendingState,
  runStreamInto,
  type AsyncState,
  idle,
  isSuccess,
  isPending,
  isFailure,
} from 'react-effect-hooks'
import { runWithAppRuntime, type AppServices } from '../../../../lib/appLayer'

type SetProfileState = AsyncState<void, Error>

export default function SelectProfilePage() {
  const navigate = useNavigate()
  const location = useLocation()
  const { runtime } = useEffectRuntime<AppServices>()
  const { state: authState, isPending: loading } = useAuthenticationState()
  const from =
    (location.state as { from?: { pathname: string } } | null)?.from
      ?.pathname ?? '/dashboard'

  const [submitState, , setSubmitStateAsEffect] = useEffectState<
    SetProfileState,
    never,
    never
  >(idle())

  const setScopeEffect = useCallback(
    (
      orgId: string,
      roleId: string,
      roleName: string,
    ): Effect.Effect<void, Error, AppServices> =>
      Effect.gen(function* () {
        const store = yield* AuthenticationStore
        const prev = yield* store.getState()
        yield* store.setScope(orgId, roleId, roleName)
        yield* store.fetchMe().pipe(
          Effect.catchAll((e) =>
            Effect.gen(function* () {
              yield* store.setScope(
                prev.currentOrgId ?? '',
                prev.currentRoleId ?? '',
                prev.currentRoleName ?? '',
              )
              return yield* Effect.fail(e)
            }),
          ),
        )
      }),
    [],
  )

  const handleSelectScope = useCallback(
    (orgId: string, roleId: string, roleName: string) => {
      const stream = streamWithPendingState(
        setScopeEffect(orgId, roleId, roleName),
      )
      const effect = runStreamInto(stream, setSubmitStateAsEffect)
      runWithAppRuntime(runtime, effect).catch(() => {})
    },
    [runtime, setScopeEffect, setSubmitStateAsEffect],
  )

  const successEffect: Effect.Effect<void, never, AppServices> = Effect.gen(
    function* () {
      if (!isSuccess(submitState)) return
      yield* Effect.sync(() => navigate(from, { replace: true }))
      yield* setSubmitStateAsEffect(idle<void, Error>())
    },
  )
  useRunEffect(successEffect, [submitState, navigate, from, setSubmitStateAsEffect])

  const user = authState?.user ?? null
  const profiles = authState?.profiles ?? []
  const needs_profile_select = authState?.needs_profile_select ?? false

  if (!loading && user == null) {
    return (
      <Navigate
        to="/authentication/login"
        replace
        state={{ from: { pathname: '/dashboard' } }}
      />
    )
  }
  if (!loading && user != null && !needs_profile_select) {
    return <Navigate to={from} replace />
  }

  const submitting = isPending(submitState)
  const errorMessage =
    isFailure(submitState) && submitState.error
      ? submitState.error instanceof Error
        ? submitState.error.message
        : String(submitState.error)
      : null

  if (loading || profiles.length === 0) {
    return (
      <Box sx={{ p: 3 }}>
        <Typography variant="h5" gutterBottom>
          Select profile
        </Typography>
        <Typography color="text.secondary">
          {loading ? 'Loading…' : 'No profiles available.'}
        </Typography>
      </Box>
    )
  }

  return (
    <>
      <Typography variant="h4" component="h1" gutterBottom>
        Select profile
      </Typography>
      <Typography variant="body1" color="text.secondary" sx={{ mb: 2 }}>
        Choose the organization and role to use for this session.
      </Typography>
      {errorMessage != null && (
        <Typography
          color="error"
          sx={{ mb: 2 }}
          data-testid="profile-select-error"
        >
          {errorMessage}
        </Typography>
      )}
      <Box sx={{ display: 'flex', flexDirection: 'column', gap: 1.5 }}>
        {profiles.map((p) => (
          <Card key={p.org_id + (p.role_id ?? p.role)} variant="outlined">
            <CardActionArea
              onClick={() =>
                handleSelectScope(
                  p.org_id,
                  p.role_id ?? '',
                  p.role ?? '',
                )
              }
              disabled={submitting}
              data-testid={`profile-${p.org_name}-${p.role}`}
            >
              <CardContent>
                <Typography variant="subtitle1">
                  {p.org_name} · {(p.role ?? '').charAt(0).toUpperCase()}
                  {(p.role ?? '').slice(1)}
                </Typography>
              </CardContent>
            </CardActionArea>
          </Card>
        ))}
      </Box>
    </>
  )
}
