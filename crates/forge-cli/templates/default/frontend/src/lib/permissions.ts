/**
 * Permission checks aligned with backend (§7):
 * - Exact match on <entity>.<action> (create, read, update, delete)
 * - all.read grants any <entity>.read
 * - all.write grants any <entity>.create, .update, .delete
 * - "dashboard" access = has at least one permission in scope (§5)
 * - Legacy dashboard.* keys are accepted when checking entity keys (§8 migration).
 */

/** Keys that grant the given required permission (entity + legacy). */
function equivalents(required: string): readonly string[] {
  const map: Record<string, readonly string[]> = {
    'organization.read': ['organization.read', 'dashboard.organizations.read'],
    'organization.create': [
      'organization.create',
      'dashboard.organizations.write',
    ],
    'organization.update': [
      'organization.update',
      'dashboard.organizations.write',
    ],
    'organization.delete': [
      'organization.delete',
      'dashboard.organizations.write',
    ],
    'user.read': ['user.read', 'dashboard.users.read'],
    'user.create': ['user.create', 'dashboard.users.write'],
    'user.update': ['user.update', 'dashboard.users.write'],
    'user.delete': ['user.delete', 'dashboard.users.write'],
    'role.read': ['role.read', 'dashboard.roles.read'],
    'role.create': ['role.create', 'dashboard.roles.write'],
    'role.update': ['role.update', 'dashboard.roles.write'],
    'role.delete': ['role.delete', 'dashboard.roles.write'],
    'permission.read': ['permission.read', 'dashboard.permissions.read'],
    'permission.create': ['permission.create', 'dashboard.permissions.write'],
    'permission.update': ['permission.update', 'dashboard.permissions.write'],
    'permission.delete': ['permission.delete', 'dashboard.permissions.write'],
    'audit.read': ['audit.read', 'dashboard.audit.read'],
  }
  return map[required] ?? [required]
}

export function hasPermission(
  userPermissions: string[],
  required: string | readonly string[],
): boolean {
  const list = Array.isArray(required) ? required : [required]
  const perms = new Set(userPermissions)
  return list.some((p) => {
    if (perms.has(p)) return true
    if (p === 'dashboard') {
      return userPermissions.length > 0
    }
    const equivs = equivalents(p)
    if (equivs.some((e) => perms.has(e))) return true
    if (p.endsWith('.read') && perms.has('all.read')) return true
    if (
      (p.endsWith('.write') ||
        p.endsWith('.create') ||
        p.endsWith('.update') ||
        p.endsWith('.delete')) &&
      perms.has('all.write')
    )
      return true
    return false
  })
}
