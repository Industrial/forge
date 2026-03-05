/**
 * Permission checks aligned with backend has_permission:
 * - Exact match on permission key
 * - "dashboard" is granted by "dashboard" | "all.read" | "all.write" | "dashboard.*"
 * - "*.read" is granted by that key or "all.read"
 * - "*.write" is granted by that key or "all.write"
 */

export function hasPermission(
  userPermissions: string[],
  required: string | string[],
): boolean {
  const list = Array.isArray(required) ? required : [required]
  const perms = new Set(userPermissions)
  return list.some((p) => {
    if (perms.has(p)) return true
    if (p === 'dashboard') {
      return (
        perms.has('all.read') ||
        perms.has('all.write') ||
        userPermissions.some((u) => u.startsWith('dashboard.'))
      )
    }
    if (p.endsWith('.read') && perms.has('all.read')) return true
    if (p.endsWith('.write') && perms.has('all.write')) return true
    return false
  })
}
