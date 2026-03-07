/**
 * Permission checks for UI. Compare required permission(s) against the current session's list.
 * When the reactive store exposes permissions (e.g. from /api/auth/me), pass them here.
 */
export function hasPermission(
  permissions: readonly string[],
  required: string | readonly string[],
): boolean {
  const requiredList = Array.isArray(required) ? required : [required]
  if (permissions.includes('all.read') || permissions.includes('all.write')) {
    return true
  }
  return requiredList.some((r) => permissions.includes(r))
}
