import { Navigate, useLocation, useNavigate } from 'react-router-dom'
import Box from '@mui/material/Box'
import Card from '@mui/material/Card'
import CardActionArea from '@mui/material/CardActionArea'
import CardContent from '@mui/material/CardContent'
import Typography from '@mui/material/Typography'
import { Effect, Option } from 'effect'
import { useCallback, useState } from 'react'

import { getApplicationLayer } from '@/lib/appLayer'
import { useAuthenticationStateReactiveStore } from '@/features/authentication/stores'
import { Authentication } from '@/features/authentication/services/Authentication'
import { ScopeError } from '../../errors'

export default function SelectScopePage() {
  const navigate = useNavigate()
  const location = useLocation()
  const authentication = useAuthenticationStateReactiveStore()
  const [submitting, setSubmitting] = useState(false)
  const [errorMessage, setErrorMessage] = useState<string | null>(null)

  const from =
    (location.state as { from?: { pathname: string } } | null)?.from
      ?.pathname ?? '/dashboard'

  const user = Option.getOrElse(authentication.user, () => null)
  const needs_scope_select = Option.getOrElse(
    authentication.needsScopeSelect,
    () => false,
  )
  // Scopes are not in the reactive store; extend AuthenticationState / me API if you need scope list.
  const scopes: {
    org_id: string
    role_id?: string
    role?: string
    org_name: string
  }[] = []
  const loading = false

  const handleSelectScope = useCallback(
    (orgId: string, roleId: string, _roleName: string) => {
      Effect.runPromise(
        Effect.gen(function* () {
          const auth = yield* Authentication
          setSubmitting(true)
          setErrorMessage(null)
          yield* auth.selectScope(orgId, roleId)
          setSubmitting(false)
          navigate(from, { replace: true })
        }).pipe(
          Effect.mapError((error) => {
            setSubmitting(false)
            setErrorMessage(
              error instanceof ScopeError ? error.message : String(error),
            )
          }),
          Effect.provide(getApplicationLayer()),
        ),
      )
    },
    [navigate, from],
  )

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
