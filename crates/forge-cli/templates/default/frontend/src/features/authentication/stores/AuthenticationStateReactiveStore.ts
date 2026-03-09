/**
 * Authentication state as a reactive store. Single source of truth for auth in the Effect layer.
 * DSL: define once, merge layer in app layer, use useAuthStore() / useAuthStoreWithInit() in React.
 */
import { Option } from 'effect'

import { defineStore, type ReactiveStore } from '@/lib/ReactiveStore'
import type { AuthenticationUser } from '@/features/authentication/domain/AuthenticationUser'

export interface CurrentScope {
  readonly organizationId: string
  readonly roleId: string
}

export interface AuthenticationState {
  token: Option.Option<string>
  user: Option.Option<AuthenticationUser>
  needsScopeSelect: Option.Option<boolean>
  /** Permission keys from /api/auth/me (e.g. for usePermission). */
  permissions: readonly string[]
  /** Current ReBAC scope (org + role) when set; none when not selected. */
  currentScope: Option.Option<CurrentScope>
}

export const initialAuthenticationState: AuthenticationState = {
  token: Option.none(),
  user: Option.none(),
  needsScopeSelect: Option.none(),
  permissions: [],
  currentScope: Option.none(),
}

/** DSL: one store definition; use .tag in Effect, .layer in app layer, useAuthStore in React. */
export const AuthStore = defineStore(
  '@forge/AuthenticationStateReactiveStore',
  initialAuthenticationState,
)

export const AuthStoreTag = AuthStore.tag
export const authStoreLayer = AuthStore.layer

/** @deprecated Use AuthStoreTag. */
export const AuthenticationStateReactiveStoreTag = AuthStoreTag

export type AuthenticationStateReactiveStore =
  ReactiveStore<AuthenticationState>

/**
 * Returns the auth state store layer. Use from buildApplicationLayer / getApplicationLayer.
 */
export function getAuthenticationStateStoreLayer() {
  return authStoreLayer
}
