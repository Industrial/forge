/**
 * Hook to subscribe to the authentication state reactive store. Uses
 * {@link useRunWithAppLayer} so effects run with the application layer.
 */
import { useRunWithAppLayer } from '@/lib/appLayer'
import { useReactiveStore } from '@/lib/ReactiveStore'
import {
  AuthenticationStateReactiveStoreTag,
  initialAuthenticationState,
  type AuthenticationState,
} from '@/features/authentication/stores/AuthenticationStateReactiveStore'

export function useAuthenticationStateReactiveStore(): AuthenticationState {
  const { run, runFork } = useRunWithAppLayer()
  return useReactiveStore(
    AuthenticationStateReactiveStoreTag,
    initialAuthenticationState,
    run,
    runFork,
  )
}
