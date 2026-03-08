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

const ProfilesResponseSchema = Schema.Struct({
  profiles: Schema.optional(
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

type Profile = {
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

  const [scopes, setScopes] = useState<Profile[]>([])
  const [loading, setLoading] = useState(false)
  const [autoSelecting, setAutoSelecting] = useState(false)

  // Fetch profiles when user is authenticated
  // If needsScopeSelect is false, there's exactly one profile - fetch and auto-select it
  // If needsScopeSelect is true, there are multiple profiles - fetch to check count
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

        const req = HttpClientRequest.get(`${baseUrl}/api/auth/profiles`).pipe(
          HttpClientRequest.setHeader('Authorization', `Bearer ${token}`),
        )
        const response = yield* client.execute(req)

        if (response.status >= 200 && response.status < 300) {
          const rawBody = yield* response.json
          const body = yield* pipe(
            Schema.decodeUnknown(ProfilesResponseSchema)(rawBody),
            Effect.catchAll(() =>
              Effect.succeed({ profiles: [] } as {
                profiles?: Profile[]
              }),
            ),
          )
          const profiles = body.profiles ?? []
          setScopes(profiles)
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

  // Auto-select single scope (when there's exactly one profile)
  useEffect(() => {
    if (
      !loading &&
      isUserAuthenticated &&
      scopes.length === 1 &&
      !autoSelecting
    ) {
      setAutoSelecting(true)
      Effect.runPromise(
        Effect.gen(function* () {
          const p = scopes[0]
          const orgId = p.org_id
          const roleId = p.role_id ?? ''
          const auth = yield* Authentication
          yield* auth.selectScope(orgId, roleId)
        }).pipe(
          Effect.provide(getApplicationLayer()),
          Effect.ensuring(Effect.sync(() => setAutoSelecting(false))),
        ),
      ).finally(() => {
        setAutoSelecting(false)
      })
    }
  }, [loading, isUserAuthenticated, scopes.length, autoSelecting, scopes])

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
  if (!isUserAuthenticated) return null

  // If needsScopeSelect is false, user has exactly one profile and it should be auto-selected
  // If it's true and we have multiple scopes, redirect to selection
  // If it's true and we have one scope, we're auto-selecting (handled above)
  if (needsScopeSelect) {
    if (scopes.length === 1) {
      // Auto-selecting, show loading (handled above)
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
    return <Navigate to="/authentication/select-scope" replace />
  }
  return <>{children}</>
}

export default DashboardScopeGuard
