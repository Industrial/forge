import { Effect, Runtime } from 'effect'
import { useCallback, useRef, useState } from 'react'
import { useEffectRuntime } from './EffectRuntime'

/** Accepted argument for the setter returned by {@link useEffectState}. */
type SetStateAction<A, E, R> = A | ((prev: A) => A) | Effect.Effect<A, E, R>

/** Effect-returning setter for use in Effect.gen (e.g. yield* setStateAsEffect(null)). */
export type SetStateAsEffect<A> = (
  action: A | ((prev: A) => A),
) => Effect.Effect<void, never, never>

/**
 * Mirrors React's `useState`, but the setter can accept a value, an updater
 * function, or an Effect.
 *
 * - **Value**: sets state directly (same as `useState`).
 * - **Updater** `(prev => next)`: same as React's functional update.
 * - **Effect**: run with the context runtime; on success state is set to the
 *   result; on failure `onError` is called if provided.
 *
 * Use this when some updates are effectful (e.g. fetch then set) and you want
 * a single, consistent setter API.
 *
 * The third element of the tuple is a setter that returns an Effect so you can
 * compose it in Effect.gen: `yield* setStateAsEffect(null)`.
 *
 * @typeParam A - State value type.
 * @typeParam E - Error type when the setter receives an Effect (default: `unknown`).
 * @typeParam R - Requirements for the Effect (from context runtime).
 * @param initialState - Initial state or a lazy initializer `() => A`.
 * @param onError - Optional. Called when an Effect passed to the setter fails.
 * @returns Tuple `[state, setState, setStateAsEffect]`; `setStateAsEffect(value)` returns `Effect<void>` so you can `yield*` it.
 *
 * @example
 * ```tsx
 * const [user, setUser, setUserAsEffect] = useEffectState<User | null>(null, (e) => toast.error(String(e)));
 *
 * setUser(await fetchUser(id));        // sync: setUser(value)
 * setUser(prev => prev ?? defaultUser); // updater
 * setUser(fetchUser(id));               // Effect: runs then sets on success
 * yield* setUserAsEffect(null);         // inside Effect.gen: compose state update as effect
 * ```
 */
export function useEffectState<A, E = unknown, R = never>(
  initialState: A | (() => A),
  onError?: (error: E) => void,
): [A, (action: SetStateAction<A, E, R>) => void, SetStateAsEffect<A>] {
  const { runtime } = useEffectRuntime<R>()
  const [state, setState] = useState<A>(initialState)
  const runCountRef = useRef(0)

  const set = useCallback(
    (action: SetStateAction<A, E, R>) => {
      if (Effect.isEffect(action)) {
        const runId = ++runCountRef.current
        Runtime.runPromise(runtime)(action as Effect.Effect<A, E, R>)
          .then((value) => {
            if (runId === runCountRef.current) {
              setState(value)
            }
          })
          .catch((error) => {
            if (runId === runCountRef.current) {
              onError?.(error as E)
            }
          })
        return
      }
      if (typeof action === 'function') {
        setState((prev) => (action as (prev: A) => A)(prev))
        return
      }
      setState(action as A)
    },
    [runtime, onError],
  )

  const setAsEffect = useCallback((action: A | ((prev: A) => A)) => {
    return Effect.sync(() => {
      if (typeof action === 'function') {
        setState((prev) => (action as (prev: A) => A)(prev))
      } else {
        setState(action as A)
      }
    })
  }, [])

  return [state, set, setAsEffect]
}
