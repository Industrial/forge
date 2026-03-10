import React, { useEffect, useState } from 'react'
import { Navigate } from 'react-router-dom'
import { Effect, Option, pipe, Schema } from 'effect'
import { HttpClient, HttpClientRequest } from '@effect/platform'
import CenteredLoader from '@/components/CenteredLoader'
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
  const permissions = authentication.permissions

  const [scopes, setScopes] = useState<Scope[]>([])
  const [loading, setLoading] = useState(false)
  const [autoSelecting, setAutoSelecting] = useState(false)
  const [autoSelectionAttempted, setAutoSelectionAttempted] = useState(false)

  // Fetch scopes only when needsScopeSelect is true
  // When needsScopeSelect is false, user already has a scope selected (from login), so we don't need to fetch
  useEffect(() => {
    if (
      !isUserAuthenticated ||
      !needsScopeSelect ||
      scopes.length > 0 ||
      loading
    ) {
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

          setScopes(scopesList.map((s) => ({ ...s })))
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
  }, [
    isUserAuthenticated,
    needsScopeSelect,
    scopes.length,
    loading,
    authentication.token,
  ])

  // Auto-select single scope when needsScopeSelect is true and we have exactly one scope
  useEffect(() => {
    if (
      !needsScopeSelect ||
      loading ||
      !isUserAuthenticated ||
      scopes.length !== 1 ||
      autoSelecting ||
      autoSelectionAttempted
    ) {
      return
    }

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
  }, [
    needsScopeSelect,
    loading,
    isUserAuthenticated,
    scopes.length,
    autoSelecting,
    scopes,
    autoSelectionAttempted,
  ])

  if (!isUserAuthenticated) {
    return null
  }

  // If needsScopeSelect is false, user already has a scope selected (from login).
  // Render children so the dashboard shell appears; permission-gated UI (e.g. sidebar
  // items) uses permissions and will show/hide correctly. If permissions are still
  // loading (empty), we still render to avoid an infinite spinner when /me with
  // scope is slow or fails; the UI degrades gracefully (fewer nav items until loaded).
  if (!needsScopeSelect) {
    return <>{children}</>
  }

  // If needsScopeSelect is true, we need to fetch scopes and either redirect or auto-select
  if (loading || autoSelecting) {
    return <CenteredLoader />
  }

  // If needsScopeSelect is true and we have multiple scopes, redirect to selection
  if (scopes.length > 1) {
    return <Navigate to="/authentication/select-scope" replace />
  }

  // If needsScopeSelect is true and we have one scope, we're auto-selecting (handled above)
  if (scopes.length === 1) {
    return <CenteredLoader />
  }

  return <CenteredLoader />
}

export default DashboardScopeGuard
