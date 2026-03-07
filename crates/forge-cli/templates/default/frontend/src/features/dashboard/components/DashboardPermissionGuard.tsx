import React from 'react'
import { Navigate } from 'react-router-dom'
import { Option } from 'effect'

import { useAuthenticationStateReactiveStore } from '@/features/authentication/stores'
import { hasPermission } from '@/lib/permissions'

type DashboardPermissionGuardProps = {
  /** Single permission or list; access allowed if user has any of them (e.g. .read or .write). */
  permission: string | string[]
  children: React.ReactNode
}

/**
 * Renders children only if the current session has at least one of the required permissions.
 * Otherwise redirects to /dashboard. Use for dashboard sub-routes.
 */
export default function DashboardPermissionGuard({
  permission,
  children,
}: DashboardPermissionGuardProps) {
  const authentication = useAuthenticationStateReactiveStore()
  const user = Option.getOrElse(authentication.user, () => null)
  // Permissions not in reactive store yet; extend store/me API and set permissions here.
  const permissions: readonly string[] = user != null ? [] : []
  const loading = false

  if (loading) {
    return null
  }

  if (!hasPermission(permissions, permission)) {
    const to =
      (Array.isArray(permission) ? permission[0] : permission) === 'dashboard'
        ? '/'
        : '/dashboard'
    return <Navigate to={to} replace />
  }

  return <>{children}</>
}
