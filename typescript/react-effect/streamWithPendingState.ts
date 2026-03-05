/**
 * Builds a Stream that emits async state for an operation: first **pending**,
 * then **success(value)** or **failure(error)** when the effect completes.
 *
 * Use with `Stream.runForEach(stream, setStateAsEffect)` (or similar) to drive
 * React state from the stream. Composes with `useRunEffect` for refresh-on-deps
 * or with `runWithAppRuntime` for one-off runs (e.g. mutations).
 *
 * @typeParam A - Success value type.
 * @typeParam E - Error type.
 * @typeParam R - Requirements for the effect (from context runtime).
 * @param effect - The effect to run (e.g. list fetch, create-then-list).
 * @returns A stream that emits `pending()`, then `success(a)` or `failure(e)`.
 *
 * @example
 * ```ts
 * const refreshStream = streamWithPendingState(listEffect);
 * useRunEffect(
 *   Stream.runForEach(refreshStream, setListStateAsEffect),
 *   [trigger]
 * );
 * ```
 */
import { Effect, Stream } from 'effect'
import type { AsyncState } from './AsyncState'
import { pending, success, failure } from './AsyncState'

export function streamWithPendingState<A, E, R>(
  effect: Effect.Effect<A, E, R>,
): Stream.Stream<AsyncState<A, E>, never, R> {
  const stateEffect = effect.pipe(
    Effect.match({
      onFailure: (e: E) => failure<A, E>(e) as AsyncState<A, E>,
      onSuccess: (value: A) => success<A, E>(value) as AsyncState<A, E>,
    }),
  )
  return Stream.concat(
    Stream.succeed(pending<A, E>()),
    Stream.fromEffect(stateEffect),
  )
}
