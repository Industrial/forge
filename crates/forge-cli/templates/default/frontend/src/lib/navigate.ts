/**
 * Helpers for React Router navigation that return Effect.
 *
 * React Router v7's useNavigate() returns a function whose return type is
 * `void | Promise<void>`. We normalize to a promise internally so we can
 * convert to Effect; the public API returns Effect.
 */

import { Effect } from 'effect'

/** Type of the function returned by useNavigate() from react-router-dom. */
export type NavigateFunction = (to: string, options?: { replace?: boolean }) => void | Promise<void>

/**
 * Normalizes React Router v7 navigate (void | Promise<void>) to a single
 * Promise so it can be awaited or turned into an Effect.
 */
function navigateToPromise(
  navigate: NavigateFunction,
  to: string,
  options?: { replace?: boolean },
): Promise<void> {
  const result = navigate(to, options)
  return result != null && typeof (result as Promise<unknown>).then === 'function'
    ? (result as Promise<void>)
    : Promise.resolve()
}

/**
 * Performs navigation and returns an Effect. Use inside Effect.gen with yield*.
 * Handles both sync and async navigate return types (React Router v7).
 *
 * @example
 * const navigate = useNavigate()
 * yield* navigateTo(navigate, '/', { replace: true })
 */
export function navigateTo(
  navigate: NavigateFunction,
  to: string,
  options?: { replace?: boolean },
): Effect.Effect<void, Error, never> {
  return Effect.tryPromise({
    try: () => navigateToPromise(navigate, to, options),
    catch: (e) => (e instanceof Error ? e : new Error(String(e))),
  })
}
