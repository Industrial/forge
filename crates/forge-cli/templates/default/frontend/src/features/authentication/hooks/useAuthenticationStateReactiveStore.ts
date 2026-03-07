/**
 * Hook to subscribe to the authentication state reactive store. Uses
 * {@link useRunWithAppLayer} so effects run with the application layer.
 * Returns both state and {@link initialized}; use {@link initialized} to avoid
 * redirecting before the store has received a value (e.g. after restoreSession).
 */
import { useRunWithAppLayer } from '@/lib/appLayer'
import { useReactiveStoreWithInit } from '@/lib/ReactiveStore'
import {
  AuthenticationStateReactiveStoreTag,
  initialAuthenticationState,
  type AuthenticationState,
} from '@/features/authentication/stores/AuthenticationStateReactiveStore'

export function useAuthenticationStateReactiveStore(): {
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
