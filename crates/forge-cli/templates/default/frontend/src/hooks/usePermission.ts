import { useAuthentication } from '../context/AuthenticationContext'

/**
 * Returns whether the current user has the given permission string.
 */
export function usePermission(permission: string): boolean {
  const { permissions } = useAuthentication()
  return permissions.includes(permission)
}
