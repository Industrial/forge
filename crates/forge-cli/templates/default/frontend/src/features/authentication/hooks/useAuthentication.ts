/**
 * useAuthenticationState – auth state from AuthenticationStore via runApp.
 *
 * Run AuthenticationStore.getState() on mount and when refresh() is called;
 * state is stored in React. Use inside AuthenticationRuntimeProvider so the app runtime is ready.
 */
import { Effect } from 'effect'
import { useCallback, useEffect, useState } from 'react'
import { flushSync } from 'react-dom'
import type { AuthenticationError } from '../errors/AuthenticationError'
import type { AuthenticationStateSnapshot } from '../domain/AuthenticationStateSnapshot'
import { AuthenticationStore } from '../services/AuthenticationStore'
import { runApp } from '../../../lib/appRuntime'

const getStateEffect = Effect.gen(function* () {
  const store = yield* AuthenticationStore
  return yield* store.getState()
})

export interface UseAuthenticationStateResult {
  /** Current auth snapshot, or null before first load or on error. */
  state: AuthenticationStateSnapshot | null
  /** Re-run getState (e.g. after login, logout, setScope). Resolves when React state has been updated. */
  refresh: () => Promise<void>
  /** True while the getState effect is running. */
  isPending: boolean
  /** Last error from getState, if any. */
  error: AuthenticationError | null
}

export function useAuthenticationState(): UseAuthenticationStateResult {
  const [state, setState] = useState<AuthenticationStateSnapshot | null>(null)
  const [isPending, setIsPending] = useState(true)
  const [error, setError] = useState<AuthenticationError | null>(null)

  const runGetState = useCallback((): Promise<void> => {
    setIsPending(true)
    setError(null)
    return runApp(getStateEffect)
      .then((snapshot: AuthenticationStateSnapshot) => {
        flushSync(() => {
          setState(snapshot)
          setIsPending(false)
        })
      })
      .catch((e: unknown) => {
        flushSync(() => {
          setError(e as AuthenticationError)
          setState(null)
          setIsPending(false)
        })
      }) as Promise<void>
  }, [])

  useEffect(() => {
    runGetState()
  }, [runGetState])

  return {
    state,
    refresh: runGetState,
    isPending,
    error,
  }
}
