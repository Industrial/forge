export type MembershipLike = {
  org_name: string
  roles?: readonly string[] | null
}

export function membershipsSummary(
  memberships: readonly MembershipLike[],
): string {
  if (!memberships.length) {
    return '—'
  }
  return memberships
    .map((m) => `${m.org_name}: ${(m.roles ?? []).join(', ') || '—'}`)
    .join('; ')
}
