import { useAuthStore } from '@/features/authentication/stores'
import { hasPermission } from '@/lib/permissions'

/**
 * Permission → UI helper: returns whether the current user has the required permission(s).
 * Uses `state.permissions` from auth store (e.g. from /api/auth/me).
 *
 * @param required - One key (e.g. `'organization.read'`) or array (any one grants).
 * @returns true if the user has at least one of the required keys (or equivalent / all.read / all.write).
 */
export function usePermission(required: string | readonly string[]): boolean {
  const authentication = useAuthStore()
  return hasPermission(authentication.permissions, required)
}
