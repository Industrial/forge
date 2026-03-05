import React from 'react'
import { Navigate } from 'react-router-dom'
import { useAuthentication } from '../../../context/AuthenticationContext'

type SelectProfileOnlyGuardProps = { children: React.ReactNode }

/**
 * Allows access only when the user is logged in and needs_profile_select is true.
 * Redirects to /dashboard when the user has already selected a profile (profile
 * switching is then done via the user profile dropdown only).
 * Use inside ProtectedRoute so unauthenticated users are already redirected.
 */
export default function SelectProfileOnlyGuard({
  children,
}: SelectProfileOnlyGuardProps) {
  const { user, needs_profile_select, loading } = useAuthentication()

  if (loading) return null
  if (user == null) return null // ProtectedRoute handles unauthenticated
  if (!needs_profile_select) {
    return <Navigate to="/dashboard" replace />
  }
  return <>{children}</>
}
