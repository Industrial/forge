/**
 * Authentication state as a reactive store. Single source of truth for auth in the Effect layer.
 * Built with defineStore (Layer.sync, in-memory). Use getAuthenticationStateStoreLayer() for app composition.
 * Subscribe via useAuthStore(), useAuthStoreWithInit(), or useAuthenticationStateReactiveStore().
 */
import { Option } from 'effect'

import { defineStore, type ReactiveStore } from '@/lib/ReactiveStore'
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

const store = defineStore(
  '@forge/AuthenticationStateReactiveStore',
  initialAuthenticationState,
)

export const AuthenticationStateReactiveStoreTag = store.tag
const authenticationStateStoreLayer = store.layer

export type AuthenticationStateReactiveStore =
  ReactiveStore<AuthenticationState>

/**
 * Returns the auth state store layer. Use from buildApplicationLayer / getApplicationLayer.
 * The layer is created once (by makeReactiveStore) and shared.
 */
export function getAuthenticationStateStoreLayer() {
  return authenticationStateStoreLayer
}
