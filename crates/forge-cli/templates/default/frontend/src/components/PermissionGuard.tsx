import type { ReactNode } from 'react'
import { Navigate } from 'react-router-dom'
import { Option } from 'effect'
import Box from '@mui/material/Box'
import { useAuthStore } from '@/features/authentication/stores'
import { hasPermission } from '@/lib/permissions'
import LoadingSpinner from './LoadingSpinner'

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
  const { permissions: userPermissions, user } = useAuthStore()
  // If user is authenticated but permissions are empty, permissions are still loading
  // Show loading spinner instead of blocking or redirecting
  const isAuthenticated = Option.isSome(user)
  const permissionsLoading = isAuthenticated && userPermissions.length === 0
  
  if (permissionsLoading) {
    // Permissions are loading, show loading spinner
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
  
  const allowed = hasPermission(userPermissions, permissions)
  if (!allowed) {
    return <Navigate to={redirectTo} replace />
  }
  return <>{children}</>
}
