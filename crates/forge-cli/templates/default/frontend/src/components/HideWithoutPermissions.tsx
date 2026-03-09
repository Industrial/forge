import type { ReactNode } from 'react'
import { useAuthStore } from '@/features/authentication/stores'
import { useComponentLogger } from '@/hooks'
import { shouldHideWithoutPermissions } from '@/lib/permissions'

/**
 * Renders children only when the user has at least one of the given permissions.
 * Hides content when the user has none of the listed permissions.
 */
export function HideWithoutPermissions({
  permissions,
  children,
}: {
  permissions: readonly string[]
  children: ReactNode
}) {
  useComponentLogger('HideWithoutPermissions')
  const { permissions: userPermissions } = useAuthStore()
  const hide = shouldHideWithoutPermissions(userPermissions, ...permissions)
  return hide ? null : <>{children}</>
}
