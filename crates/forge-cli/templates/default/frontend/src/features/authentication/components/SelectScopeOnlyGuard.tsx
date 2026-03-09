import React from 'react'
import { RouteGuard } from '@/components/RouteGuard'

type SelectScopeOnlyGuardProps = { children: React.ReactNode }

/**
 * Allows access only when the user is logged in and needs_scope_select is true.
 * Use inside ProtectedRoute so unauthenticated users are already redirected.
 */
export default function SelectScopeOnlyGuard({
  children,
}: SelectScopeOnlyGuardProps) {
  return (
    <RouteGuard requireScopeSelectOnly redirectTo="/dashboard">
      {children}
    </RouteGuard>
  )
}
