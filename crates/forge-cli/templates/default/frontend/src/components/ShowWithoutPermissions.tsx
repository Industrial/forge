import type { ReactNode } from 'react'
import { useAuthStore } from '@/features/authentication/stores'
import { shouldShowWithoutPermissions } from '@/lib/permissions'

/**
 * Renders children only when the user has none of the given permissions.
 * Use when content should be visible only to users who lack the listed permissions.
 */
export function ShowWithoutPermissions({
  permissions,
  children,
}: {
  permissions: readonly string[]
  children: ReactNode
}) {
  const { permissions: userPermissions } = useAuthStore()
  const show = shouldShowWithoutPermissions(userPermissions, ...permissions)
  return show ? children : null
}
