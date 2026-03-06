/**
 * Authentication state as a reactive store. Single source of truth for auth in the Effect runtime;
 * components subscribe via useReactiveStore(AuthenticationStateReactiveStoreTag, initialAuthenticationState).
 */
import type { ReactiveStore } from '@/lib/ReactiveStore'
import { makeReactiveStore } from '@/lib/ReactiveStore'

export interface AuthenticationState {
  token: string | null
  user: { email: string } | null
}

export const initialAuthenticationState: AuthenticationState = {
  token: null,
  user: null,
}

const { tag, layer } = makeReactiveStore<AuthenticationState>(
  '@forge/AuthenticationStateReactiveStore',
  initialAuthenticationState,
)

export const AuthenticationStateReactiveStoreTag = tag
export const AuthenticationStateReactiveStoreLayer = layer

export type AuthenticationStateReactiveStore =
  ReactiveStore<AuthenticationState>
