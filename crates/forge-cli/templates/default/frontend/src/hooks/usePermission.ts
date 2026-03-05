import { useAuthentication } from '../context/AuthenticationContext'
import { hasPermission } from '../lib/permissions'

/**
 * Returns whether the current user has the given permission (matches backend has_permission: exact, dashboard↔all.read/all.write, *.read↔all.read, *.write↔all.write).
 */
export function usePermission(permission: string): boolean {
  const { permissions } = useAuthentication()
  return hasPermission(permissions, permission)
}
