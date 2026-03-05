import React from 'react'
import { Navigate } from 'react-router-dom'
import { useAuthentication } from '../../../context/AuthenticationContext'

type DashboardProfileGuardProps = { children: React.ReactNode }

/**
 * Redirects to /authentication/select-profile when the user is logged in but has no session profile set.
 * Use inside ProtectedRoute so it only runs for authenticated users.
 */
export default function DashboardProfileGuard({
  children,
}: DashboardProfileGuardProps) {
  const { user, needs_profile_select, loading } = useAuthentication()

  if (loading) return null
  if (user == null) return null // ProtectedRoute handles unauthenticated
  if (needs_profile_select) {
    return <Navigate to="/authentication/select-profile" replace />
  }
  return <>{children}</>
}
