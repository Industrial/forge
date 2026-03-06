import { Navigate, useLocation, useNavigate } from 'react-router-dom'
import Box from '@mui/material/Box'
import Card from '@mui/material/Card'
import CardActionArea from '@mui/material/CardActionArea'
import CardContent from '@mui/material/CardContent'
import Typography from '@mui/material/Typography'
import { Effect } from 'effect'
import { useCallback, useEffect } from 'react'
import { AuthenticationStore } from '../../services/AuthenticationStore'
import { useAuthenticationState } from '../../hooks/useAuthentication'
import {
  useEffectState,
  streamWithPendingState,
  runStreamInto,
  type AsyncState,
  idle,
  isSuccess,
  isPending,
  isFailure,
} from 'react-effect-hooks'
import { runApp } from '../../../../lib/appRuntime'
import type { AppServices } from '../../../../lib/appLayer'

type SetScopeState = AsyncState<void, Error>

export default function SelectScopePage() {
  const navigate = useNavigate()
  const location = useLocation()
  const { state: authState, isPending: loading } = useAuthenticationState()
  const from =
    (location.state as { from?: { pathname: string } } | null)?.from
      ?.pathname ?? '/dashboard'

  const [submitState, , setSubmitStateAsEffect] = useEffectState<
    SetScopeState,
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
      runApp(effect)
    },
    [setScopeEffect, setSubmitStateAsEffect],
  )

  useEffect(() => {
    if (!isSuccess(submitState)) return
    runApp(
      Effect.gen(function* () {
        yield* Effect.sync(() => navigate(from, { replace: true }))
        yield* setSubmitStateAsEffect(idle<void, Error>())
      }),
    )
  }, [submitState, navigate, from, setSubmitStateAsEffect])

  const user = authState?.user ?? null
  const scopes = authState?.scopes ?? []
  const needs_scope_select = authState?.needs_scope_select ?? false

  if (!loading && user == null) {
    return (
      <Navigate
        to="/authentication/login"
        replace
        state={{ from: { pathname: '/dashboard' } }}
      />
    )
  }
  if (!loading && user != null && !needs_scope_select) {
    return <Navigate to={from} replace />
  }

  const submitting = isPending(submitState)
  const errorMessage =
    isFailure(submitState) && submitState.error
      ? submitState.error instanceof Error
        ? submitState.error.message
        : String(submitState.error)
      : null

  if (loading || scopes.length === 0) {
    return (
      <Box sx={{ p: 3 }}>
        <Typography variant="h5" gutterBottom>
          Select scope
        </Typography>
        <Typography color="text.secondary">
          {loading ? 'Loading…' : 'No scopes available.'}
        </Typography>
      </Box>
    )
  }

  return (
    <>
      <Typography variant="h4" component="h1" gutterBottom>
        Select scope
      </Typography>
      <Typography variant="body1" color="text.secondary" sx={{ mb: 2 }}>
        Choose the organization and role to use for this session.
      </Typography>
      {errorMessage != null && (
        <Typography
          color="error"
          sx={{ mb: 2 }}
          data-testid="scope-select-error"
        >
          {errorMessage}
        </Typography>
      )}
      <Box sx={{ display: 'flex', flexDirection: 'column', gap: 1.5 }}>
        {scopes.map((p) => (
          <Card key={p.org_id + (p.role_id ?? p.role)} variant="outlined">
            <CardActionArea
              onClick={() =>
                handleSelectScope(p.org_id, p.role_id ?? '', p.role ?? '')
              }
              disabled={submitting}
              data-testid={`scope-${p.org_name}-${p.role}`}
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
