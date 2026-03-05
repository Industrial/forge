import { Effect, Stream } from 'effect'

/**
 * Runs a stream and pushes each emission into a state setter that returns an
 * Effect. Use with streamWithPendingState and useEffectState's setStateAsEffect
 * to drive React state from a stream (e.g. list refresh, mutation-then-list).
 */
export function runStreamInto<A, E, R>(
  stream: Stream.Stream<A, E, R>,
  setState: (a: A) => Effect.Effect<void, never, never>,
): Effect.Effect<void, E, R> {
  return Stream.runForEach(stream, setState)
}
