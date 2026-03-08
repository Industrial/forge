import React, { useEffect, useState } from 'react'
import { Navigate } from 'react-router-dom'
import { Effect, Option, pipe, Schema } from 'effect'
import { HttpClient, HttpClientRequest } from '@effect/platform'
import Box from '@mui/material/Box'
import LoadingSpinner from '../../../components/LoadingSpinner'
import { getApplicationLayer } from '@/lib/appLayer'
import { useAuthStore } from '@/features/authentication/stores'
import { Authentication } from '@/features/authentication/services/Authentication'
import { getBaseUrl } from '@/lib/baseUrl'

export type DashboardScopeGuardProps = {
  children: React.ReactNode
}

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

/**
 * When the user is logged in but needs scope: redirects to scope selection, or
 * auto-selects the single scope so single-org users do not see the selection screen (§7.5).
 * Use inside ProtectedRoute so it only runs for authenticated users.
 */
function DashboardScopeGuard({ children }: DashboardScopeGuardProps) {
  const authentication = useAuthStore()
  const isUserAuthenticated = Option.isSome(authentication.user)
  const needsScopeSelectValue = authentication.needsScopeSelect
  const needsScopeSelect = Option.getOrElse(needsScopeSelectValue, () => false)

  const [scopes, setScopes] = useState<Scope[]>([])
  const [loading, setLoading] = useState(false)
  const [autoSelecting, setAutoSelecting] = useState(false)
  const [autoSelectionAttempted, setAutoSelectionAttempted] = useState(false)

  // Fetch scopes when user is authenticated
  // If needsScopeSelect is false, user has exactly one scope - fetch and auto-select it
  // If needsScopeSelect is true, there are multiple scopes - fetch to check count and auto-select if only one
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

  // Auto-select single scope (when there's exactly one scope)
  // When needsScopeSelect is false, user has exactly one scope - fetch and auto-select it
  // When needsScopeSelect is true and we have one scope, also auto-select it
  useEffect(() => {
    if (
      !loading &&
      isUserAuthenticated &&
      scopes.length === 1 &&
      !autoSelecting &&
      !autoSelectionAttempted
    ) {
      setAutoSelecting(true)
      setAutoSelectionAttempted(true)
      Effect.runPromise(
        Effect.gen(function* () {
          const p = scopes[0]
          const orgId = p.org_id
          const roleId = p.role_id ?? ''
          const auth = yield* Authentication
          yield* auth.selectScope(orgId, roleId)
        }).pipe(Effect.provide(getApplicationLayer())),
      )
        .then(() => {
          setAutoSelecting(false)
        })
        .catch(() => {
          setAutoSelecting(false)
        })
    }
  }, [loading, isUserAuthenticated, scopes.length, autoSelecting, scopes, autoSelectionAttempted])

  if (!isUserAuthenticated) {
    return null
  }

  // If needsScopeSelect is false, user has exactly one scope - wait for auto-selection
  if (!needsScopeSelect) {
    // Wait for scopes to be fetched
    if (loading || scopes.length === 0) {
      return (
        <Box
          sx={{
            display: 'flex',
            justifyContent: 'center',
            alignItems: 'center',
            flex: 1,
            minHeight: '40vh',
          }}
        >
          <LoadingSpinner />
        </Box>
      )
    }
    // If we have one scope, wait for auto-selection to complete
    if (scopes.length === 1 && (autoSelecting || !autoSelectionAttempted)) {
      return (
        <Box
          sx={{
            display: 'flex',
            justifyContent: 'center',
            alignItems: 'center',
            flex: 1,
            minHeight: '40vh',
          }}
        >
          <LoadingSpinner />
        </Box>
      )
    }
    // Auto-selection completed (or no scope needed), render children
    return <>{children}</>
  }

  // If needsScopeSelect is true, we need to fetch scopes and either redirect or auto-select
  if (loading || autoSelecting) {
    return (
      <Box
        sx={{
          display: 'flex',
          justifyContent: 'center',
          alignItems: 'center',
          flex: 1,
          minHeight: '40vh',
        }}
      >
        <LoadingSpinner />
      </Box>
    )
  }

  // If needsScopeSelect is true and we have multiple scopes, redirect to selection
  if (scopes.length > 1) {
    return <Navigate to="/authentication/select-scope" replace />
  }

  // If needsScopeSelect is true and we have one scope, we're auto-selecting (handled above)
  if (scopes.length === 1) {
    return (
      <Box
        sx={{
          display: 'flex',
          justifyContent: 'center',
          alignItems: 'center',
          flex: 1,
          minHeight: '40vh',
        }}
      >
        <LoadingSpinner />
      </Box>
    )
  }

  // Still loading or no scopes yet
  return (
    <Box
      sx={{
        display: 'flex',
        justifyContent: 'center',
        alignItems: 'center',
        flex: 1,
        minHeight: '40vh',
      }}
    >
      <LoadingSpinner />
    </Box>
  )
}

export default DashboardScopeGuard
