import type { ReactNode } from 'react'
import { Navigate } from 'react-router-dom'
import { useAuthStore } from '@/features/authentication/stores'
import { hasPermission } from '@/lib/permissions'

export type PermissionGuardProps = {
  /** Required permissions (user must have at least one). If missing, redirect. */
  permissions: readonly string[]
  /** Where to redirect when the user lacks any of the required permissions. */
  redirectTo?: string
  children: ReactNode
}

/**
 * Route guard: renders children only if the user has at least one of the required permissions.
 * Otherwise redirects to redirectTo (default /dashboard). Use to protect routes by permission.
 */
export function PermissionGuard({
  permissions,
  redirectTo = '/dashboard',
  children,
}: PermissionGuardProps) {
  const { permissions: userPermissions } = useAuthStore()
  const allowed = hasPermission(userPermissions, permissions)
  if (!allowed) {
    return <Navigate to={redirectTo} replace />
  }
  return <>{children}</>
}
