import React, { useEffect, useState } from 'react'
import { Navigate } from 'react-router-dom'
import { Effect, Option } from 'effect'
import Box from '@mui/material/Box'
import LoadingSpinner from '../../../components/LoadingSpinner'
import { getApplicationLayer } from '@/lib/appLayer'
import { useAuthStore } from '@/features/authentication/stores'
import { Authentication } from '@/features/authentication/services/Authentication'

export type DashboardScopeGuardProps = {
  children: React.ReactNode
}

/**
 * When the user is logged in but needs scope: redirects to scope selection, or
 * auto-selects the single scope so single-org users do not see the selection screen (§7.5).
 * Use inside ProtectedRoute so it only runs for authenticated users.
 */
function DashboardScopeGuard({ children }: DashboardScopeGuardProps) {
  const authentication = useAuthStore()
  const isUserAuthenticated = Option.isSome(authentication.user)
  const needsScopeSelect = Option.getOrElse(
    authentication.needsScopeSelect,
    () => false,
  )

  // Scopes not in reactive store; extend store/me API to enable single-scope auto-select.
  const scopes: { org_id: string; role_id?: string; role?: string }[] = []
  const loading = false

  const [autoSelecting, setAutoSelecting] = useState(false)

  useEffect(() => {
    if (
      !loading &&
      isUserAuthenticated &&
      needsScopeSelect &&
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
      ).finally(() => setAutoSelecting(false))
    }
  }, [
    loading,
    isUserAuthenticated,
    needsScopeSelect,
    scopes.length,
    autoSelecting,
  ])

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
  if (needsScopeSelect) {
    return <Navigate to="/authentication/select-scope" replace />
  }
  return <>{children}</>
}

export default DashboardScopeGuard
