/**
 * Hooks to subscribe to the authentication state reactive store. Use
 * {@link useAuthStoreWithInit} when you need {@link initialized} to avoid
 * redirecting before the store has received a value (e.g. after restoreSession).
 */
import { useRunWithAppLayer } from '@/lib/appLayer'
import { useReactiveStore, useReactiveStoreWithInit } from '@/lib/ReactiveStore'
import {
  AuthenticationStateReactiveStoreTag,
  initialAuthenticationState,
  type AuthenticationState,
} from '@/features/authentication/stores/AuthenticationStateReactiveStore'

/** Bound hook: returns current auth state only. */
export function useAuthStore(): AuthenticationState {
  const { run, runFork } = useRunWithAppLayer()
  return useReactiveStore(
    AuthenticationStateReactiveStoreTag,
    initialAuthenticationState,
    run,
    runFork,
  )
}

/** Bound hook: returns auth state and initialized flag (for protected routes). */
export function useAuthStoreWithInit(): {
  authentication: AuthenticationState
  initialized: boolean
} {
  const { run, runFork } = useRunWithAppLayer()
  const snapshot = useReactiveStoreWithInit(
    AuthenticationStateReactiveStoreTag,
    initialAuthenticationState,
    run,
    runFork,
  )
  return {
    authentication: snapshot.value,
    initialized: snapshot.initialized,
  }
}

/** @deprecated Use useAuthStoreWithInit instead. */
export function useAuthenticationStateReactiveStore(): {
  authentication: AuthenticationState
  initialized: boolean
} {
  return useAuthStoreWithInit()
}
