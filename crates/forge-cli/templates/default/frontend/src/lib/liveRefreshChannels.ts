/**
 * Live refresh channel keys and their mapping to backend entity_id for RPC subscribe.
 * Replaces the previous WebSocket channel keys.
 */
export type LiveRefreshChannel =
  | 'audit-log'
  | 'users'
  | 'roles'
  | 'role_permissions'
  | 'organizations'

/** Alias for backward compatibility with dashboard pages. */
export type ForgeWebsocketKey = LiveRefreshChannel

const CHANNEL_TO_ENTITY_ID: Record<LiveRefreshChannel, string> = {
  'audit-log': 'audit_log',
  users: 'user',
  roles: 'role',
  role_permissions: 'role_permission',
  organizations: 'organization',
}

export function channelToEntityId(channel: LiveRefreshChannel): string {
  return CHANNEL_TO_ENTITY_ID[channel]
}
