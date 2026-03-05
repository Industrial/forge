import React, { useEffect, useState } from 'react'
import { Navigate } from 'react-router-dom'
import { Effect } from 'effect'
import { useAuthentication } from '../../../context/AuthenticationContext'
import { AuthenticationStore } from '../../authentication/services/AuthenticationStore'
import { useEffectRuntime } from 'react-effect-hooks'
import { runWithAppRuntime, type AppServices } from '../../../lib/appLayer'
import Box from '@mui/material/Box'
import LoadingSpinner from '../../../components/LoadingSpinner'

type DashboardProfileGuardProps = { children: React.ReactNode }

/**
 * When the user is logged in but needs scope: redirects to scope selection, or
 * auto-selects the single scope so single-org users do not see the selection screen (§7.5).
 * Use inside ProtectedRoute so it only runs for authenticated users.
 */
export default function DashboardProfileGuard({
  children,
}: DashboardProfileGuardProps) {
  const { user, needs_profile_select, profiles, loading, refresh } =
    useAuthentication()
  const { runtime } = useEffectRuntime<AppServices>()
  const [autoSelecting, setAutoSelecting] = useState(false)

  useEffect(() => {
    if (
      !loading &&
      user != null &&
      needs_profile_select &&
      profiles.length === 1 &&
      !autoSelecting
    ) {
      const p = profiles[0]
      const orgId = p.org_id
      const roleId = p.role_id ?? ''
      const roleName = p.role ?? ''
      setAutoSelecting(true)
      runWithAppRuntime(
        runtime,
        Effect.gen(function* () {
          const store = yield* AuthenticationStore
          yield* store.setScope(orgId, roleId, roleName)
          yield* store.fetchMe()
        }),
      )
        .then(() => refresh().then(() => setAutoSelecting(false)))
        .catch(() => setAutoSelecting(false))
    }
  }, [
    loading,
    user,
    needs_profile_select,
    profiles,
    runtime,
    autoSelecting,
    refresh,
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
  if (user == null) return null // ProtectedRoute handles unauthenticated
  if (needs_profile_select) {
    return <Navigate to="/authentication/select-profile" replace />
  }
  return <>{children}</>
}
