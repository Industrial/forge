/**
 * Authentication state as a reactive store. Single source of truth for auth in the Effect layer.
 * Built with makeReactiveStore. Use getAuthenticationStateStoreLayer() for app composition.
 * Subscribe via useAuthenticationStateReactiveStore() or useReactiveStore(AuthenticationStateReactiveStoreTag, initialAuthenticationState). No store in context.
 */
import { Option } from 'effect'

import { makeReactiveStore, type ReactiveStore } from '@/lib/ReactiveStore'
import type { AuthenticationUser } from '@/features/authentication/domain/AuthenticationUser'

export interface AuthenticationState {
  token: Option.Option<string>
  user: Option.Option<AuthenticationUser>
  needsScopeSelect: Option.Option<boolean>
  /** Permission keys from /api/auth/me (e.g. for usePermission). */
  permissions: readonly string[]
}

export const initialAuthenticationState: AuthenticationState = {
  token: Option.none(),
  user: Option.none(),
  needsScopeSelect: Option.none(),
  permissions: [],
}

const {
  tag: AuthenticationStateReactiveStoreTag,
  layer: authenticationStateStoreLayer,
} = makeReactiveStore(
  '@forge/AuthenticationStateReactiveStore',
  initialAuthenticationState,
)

export { AuthenticationStateReactiveStoreTag }

export type AuthenticationStateReactiveStore =
  ReactiveStore<AuthenticationState>

/**
 * Returns the auth state store layer. Use from buildApplicationLayer / getApplicationLayer.
 * The layer is created once (by makeReactiveStore) and shared.
 */
export function getAuthenticationStateStoreLayer() {
  return authenticationStateStoreLayer
}
