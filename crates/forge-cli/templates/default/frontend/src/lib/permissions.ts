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

/**
 * Show when user has any of the required permissions (whitelist).
 * Use for nav items, pages, buttons that require specific permissions.
 */
export function shouldShowWithPermissions(
  userPermissions: readonly string[],
  ...required: string[]
): boolean {
  return hasPermission(userPermissions, required)
}

/**
 * Hide when user has any of the given permissions (blacklist).
 * Use to hide UI for users with a specific permission (e.g. suspended).
 */
export function shouldHideWithPermissions(
  userPermissions: readonly string[],
  ...required: string[]
): boolean {
  return hasPermission(userPermissions, required)
}

/** Don't show unless user has any of the required permissions. */
export function shouldntShowWithPermissions(
  userPermissions: readonly string[],
  ...required: string[]
): boolean {
  return !shouldShowWithPermissions(userPermissions, ...required)
}

/** Don't hide; show unless user has any of the blacklist permissions. */
export function shouldntHideWithPermissions(
  userPermissions: readonly string[],
  ...required: string[]
): boolean {
  return !shouldHideWithPermissions(userPermissions, ...required)
}

// ---------- WithoutPermissions: show/hide when user has none of the given keys ----------

/** Show when user has none of the given permissions. */
export function shouldShowWithoutPermissions(
  userPermissions: readonly string[],
  ...required: string[]
): boolean {
  return !hasPermission(userPermissions, required)
}

/** Hide when user has none of the given permissions. */
export function shouldHideWithoutPermissions(
  userPermissions: readonly string[],
  ...required: string[]
): boolean {
  return !hasPermission(userPermissions, required)
}

/** Don't show when user has none of the given permissions (= show when they have any). */
export function shouldntShowWithoutPermissions(
  userPermissions: readonly string[],
  ...required: string[]
): boolean {
  return !shouldShowWithoutPermissions(userPermissions, ...required)
}

/** Don't hide when user has none (= hide when they have any). */
export function shouldntHideWithoutPermissions(
  userPermissions: readonly string[],
  ...required: string[]
): boolean {
  return !shouldHideWithoutPermissions(userPermissions, ...required)
}
