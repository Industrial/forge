import { Navigate, useLocation, useNavigate } from 'react-router-dom'
import Box from '@mui/material/Box'
import Card from '@mui/material/Card'
import CardActionArea from '@mui/material/CardActionArea'
import CardContent from '@mui/material/CardContent'
import Typography from '@mui/material/Typography'
import { Effect, Option, pipe, Schema } from 'effect'
import { HttpClient, HttpClientRequest } from '@effect/platform'
import { useCallback, useEffect, useState } from 'react'

import { useComponentLogger } from '@/hooks'
import { getApplicationLayer } from '@/lib/appLayer'
import { useAuthStore } from '@/features/authentication/stores'
import { Authentication } from '@/features/authentication/services/Authentication'
import { ScopeError } from '../../errors'
import { getBaseUrl } from '@/lib/baseUrl'

const ScopesResponseSchema = Schema.Struct({
  scopes: Schema.optional(
    Schema.Array(
      Schema.Struct({
        org_id: Schema.String,
        org_name: Schema.String,
        role_id: Schema.optional(Schema.String),
        role: Schema.String,
      }),
    ),
  ),
})

type Scope = {
  org_id: string
  org_name: string
  role_id?: string
  role: string
}

export default function SelectScopePage() {
  useComponentLogger('SelectScopePage')
  const navigate = useNavigate()
  const location = useLocation()
  const authentication = useAuthStore()
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
  const isUserAuthenticated = Option.isSome(authentication.user)

  const [scopes, setScopes] = useState<Scope[]>([])
  const [loading, setLoading] = useState(false)

  // Fetch scopes when user is authenticated
  useEffect(() => {
    if (!isUserAuthenticated || scopes.length > 0 || loading) {
      return
    }

    setLoading(true)
    Effect.runPromise(
      Effect.gen(function* () {
        const client = yield* HttpClient.HttpClient
        const baseUrl = getBaseUrl()
        const token = Option.getOrElse(authentication.token, () => '')

        const req = HttpClientRequest.get(`${baseUrl}/api/auth/scopes`).pipe(
          HttpClientRequest.setHeader('Authorization', `Bearer ${token}`),
        )

        const response = yield* client.execute(req)

        if (response.status >= 200 && response.status < 300) {
          const rawBody = yield* response.json
          const body = yield* pipe(
            Schema.decodeUnknown(ScopesResponseSchema)(rawBody),
            Effect.catchAll(() =>
              Effect.succeed({ scopes: [] } as {
                scopes?: Scope[]
              }),
            ),
          )
          const scopesList = body.scopes ?? []

          setScopes(scopesList)
        } else {
          setScopes([])
        }
      }).pipe(Effect.provide(getApplicationLayer())),
    )
      .catch(() => {
        setScopes([])
      })
      .finally(() => {
        setLoading(false)
      })
  }, [isUserAuthenticated, scopes.length, loading, authentication.token])

  const handleSelectScope = useCallback(
    (orgId: string, roleId: string, _roleName: string) => {
      Effect.runPromise(
        Effect.gen(function* () {
          const auth = yield* Authentication
          setSubmitting(true)
          setErrorMessage(null)
          yield* auth.selectScope(orgId, roleId)
          // Navigate before resetting submitting to prevent race condition
          // with reactive redirect check
          navigate(from, { replace: true })
          setSubmitting(false)
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
  // Don't redirect while submitting (scope selection in progress) to avoid race condition
  // with manual navigation in handleSelectScope
  if (!loading && !submitting && user != null && !needs_scope_select) {
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
    <Box data-testid="select-scope-page">
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
      <Box
        sx={{ display: 'flex', flexDirection: 'column', gap: 1.5 }}
        data-testid="profile-list"
      >
        {scopes.map((p, index) => (
          <Card
            key={p.org_id + (p.role_id ?? p.role)}
            variant="outlined"
            data-testid={`profile-card-${index}`}
          >
            <CardActionArea
              onClick={() =>
                handleSelectScope(p.org_id, p.role_id ?? '', p.role ?? '')
              }
              disabled={submitting}
              data-testid={`profile-select-button-${index}`}
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
    </Box>
  )
}
