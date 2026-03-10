import type { ReactNode } from 'react'
import { Navigate } from 'react-router-dom'
import { Option } from 'effect'
import {
  useAuthStore,
  useAuthStoreWithInit,
} from '@/features/authentication/stores'
import { hasPermission } from '@/lib/permissions'
import CenteredLoader from './CenteredLoader'

export type RouteGuardProps = {
  /** Require authenticated user; redirect to redirectTo (default /authentication/login) when not. */
  requireAuth?: boolean
  /** When requireAuth, wait for store init before deciding (show nothing until then). */
  requireAuthWithInit?: boolean
  /** When set, redirect to this path when user is authenticated (guest-only routes). */
  redirectIfAuthenticated?: string
  /** Require at least one of these permissions; show loader while permissions load, then redirect if missing. */
  requirePermissions?: readonly string[]
  /** Allow only when user is authenticated and has no scope selected (client-derived from currentScope). Use inside ProtectedRoute. */
  requireScopeSelectOnly?: boolean
  /** Redirect path when guard fails. Default: /authentication/login for requireAuth, /dashboard for permissions/scope. */
  redirectTo?: string
  children: ReactNode
}

const DEFAULT_AUTH_REDIRECT = '/authentication/login'
const DEFAULT_DENIED_REDIRECT = '/dashboard'

/**
 * Configurable route guard. Combine props to get ProtectedRoute-, GuestRoute-, PermissionGuard-, or SelectScopeOnlyGuard-style behavior.
 */
export function RouteGuard({
  requireAuth,
  requireAuthWithInit,
  redirectIfAuthenticated,
  requirePermissions,
  requireScopeSelectOnly,
  redirectTo,
  children,
}: RouteGuardProps): ReactNode {
  const authWithInit = useAuthStoreWithInit()
  const auth = useAuthStore()

  const effectiveAuth = requireAuthWithInit ? authWithInit.authentication : auth
  const initialized = requireAuthWithInit ? authWithInit.initialized : true
  const isAuthenticated = Option.isSome(effectiveAuth.user)

  // Guest-only: redirect if authenticated
  if (redirectIfAuthenticated !== undefined) {
    if (isAuthenticated) {
      return <Navigate to={redirectIfAuthenticated} replace />
    }
    return <>{children}</>
  }

  // Optional auth + init: wait for init then require auth
  if (requireAuthWithInit && !initialized) {
    return null
  }

  if (requireAuth && !isAuthenticated) {
    return <Navigate to={redirectTo ?? DEFAULT_AUTH_REDIRECT} replace />
  }

  // Scope selection only (e.g. select-scope page): allow only when authenticated and no scope selected yet.
  if (requireScopeSelectOnly) {
    const needsScopeSelect =
      isAuthenticated && Option.isNone(effectiveAuth.currentScope)
    if (!needsScopeSelect) {
      return <Navigate to={redirectTo ?? DEFAULT_DENIED_REDIRECT} replace />
    }
    return <>{children}</>
  }

  // Permission guard: show loader while permissions loading, redirect if not allowed
  if (requirePermissions !== undefined && requirePermissions.length > 0) {
    const userPermissions = effectiveAuth.permissions
    const permissionsLoading = isAuthenticated && userPermissions.length === 0
    if (permissionsLoading) {
      return <CenteredLoader />
    }
    const allowed = hasPermission(userPermissions, requirePermissions)
    if (!allowed) {
      return <Navigate to={redirectTo ?? DEFAULT_DENIED_REDIRECT} replace />
    }
  }

  return <>{children}</>
}
