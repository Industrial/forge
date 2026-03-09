import type { ReactNode } from 'react'
import { RouteGuard } from './RouteGuard'

export type PermissionGuardProps = {
  permissions: readonly string[]
  redirectTo?: string
  children: ReactNode
}

/**
 * Route guard: renders children only if the user has at least one of the required permissions.
 */
export function PermissionGuard({
  permissions,
  redirectTo = '/dashboard',
  children,
}: PermissionGuardProps) {
  return (
    <RouteGuard requirePermissions={permissions} redirectTo={redirectTo}>
      {children}
    </RouteGuard>
  )
}
