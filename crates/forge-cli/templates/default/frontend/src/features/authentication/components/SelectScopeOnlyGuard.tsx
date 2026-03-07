import React from 'react'
import { Navigate } from 'react-router-dom'
import { Option } from 'effect'

import { useAuthenticationStateReactiveStore } from '@/features/authentication/stores'

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
  const authentication = useAuthenticationStateReactiveStore()
  const user = Option.getOrElse(authentication.user, () => null)
  const needs_scope_select = Option.getOrElse(
    authentication.needsScopeSelect,
    () => false,
  )
  const loading = false

  if (loading) {
    return null
  }
  // ProtectedRoute handles unauthenticated
  if (user == null) {
    return null
  }
  if (!needs_scope_select) {
    return <Navigate to="/dashboard" replace />
  }
  return <>{children}</>
}
