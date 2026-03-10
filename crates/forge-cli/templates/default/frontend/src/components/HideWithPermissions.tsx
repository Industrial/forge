import type { ReactNode } from 'react'
import { useAuthStore } from '@/features/authentication/stores'
import { shouldHideWithPermissions } from '@/lib/permissions'

/**
 * Renders children only when the user has none of the given permissions (blacklist).
 * Use to hide UI for users with a specific permission (e.g. suspended).
 */
export function HideWithPermissions({
  permissions,
  children,
}: {
  permissions: readonly string[]
  children: ReactNode
}) {
  const { permissions: userPermissions } = useAuthStore()
  const hide = shouldHideWithPermissions(userPermissions, ...permissions)
  return hide ? null : children
}
