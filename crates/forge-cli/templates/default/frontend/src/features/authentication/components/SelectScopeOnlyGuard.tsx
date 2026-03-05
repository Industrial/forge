import React from 'react'
import { Navigate } from 'react-router-dom'
import { useAuthentication } from '../../../context/AuthenticationContext'

type SelectScopeOnlyGuardProps = { children: React.ReactNode }

/**
 * Allows access only when the user is logged in and needs_scope_select is true.
 * Redirects to /dashboard when the user has already selected a scope (scope
 * switching is then done via the user dropdown only).
 * Use inside ProtectedRoute so unauthenticated users are already redirected.
 */
export default function SelectScopeOnlyGuard({
  children,
}: SelectScopeOnlyGuardProps) {
  const { user, needs_scope_select, loading } = useAuthentication()

  if (loading) return null
  if (user == null) return null // ProtectedRoute handles unauthenticated
  if (!needs_scope_select) {
    return <Navigate to="/dashboard" replace />
  }
  return <>{children}</>
}
