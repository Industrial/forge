import type { ReactNode } from 'react'
import { useAuthStore } from '@/features/authentication/stores'
import { shouldShowWithPermissions } from '@/lib/permissions'

/**
 * Renders children only when the user has at least one of the given permissions (whitelist).
 * Use for nav items, sections, buttons that require specific permissions.
 */
export function ShowWithPermissions({
  permissions,
  children,
}: {
  permissions: readonly string[]
  children: ReactNode
}) {
  const { permissions: userPermissions } = useAuthStore()
  const show = shouldShowWithPermissions(userPermissions, ...permissions)
  return show ? <>{children}</> : null
}
