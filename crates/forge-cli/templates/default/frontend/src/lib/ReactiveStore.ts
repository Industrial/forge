/**
 * Reactive store: Effect.ts-style state container that notifies subscribers on change.
 * Built from SubscriptionRef (Ref + Stream of changes). Use with makeReactiveStore to
 * create a Layer, and useReactiveStore to subscribe from React.
 *
 * @see useReactiveStore – hook that triggers re-renders when the store changes
 */

import { Context, Effect, Layer, Stream, SubscriptionRef } from 'effect'

/**
 * Reactive store interface: current value (get), updates (update), and a stream of all changes.
 * Subscribers run the `changes` stream to react to updates.
 */
export interface ReactiveStore<A> {
  /** Current value. */
  readonly get: () => Effect.Effect<A, never, never>

  /** Update state with a function; all subscribers see the new value via `changes`. */
  readonly update: (f: (a: A) => A) => Effect.Effect<void, never, never>

  /** Stream of the current value and every subsequent update. Use for subscription. */
  readonly changes: Stream.Stream<A, never, never>
}

/**
 * Creates a Layer that provides a ReactiveStore<A> with the given initial value.
 * Use the returned `tag` in effects (yield* tag) and add `Layer` to your app's Layer composition.
 * Use the same `tag` with useReactiveStore(tag, initial) in React components.
 *
 * @example
 * const { tag, Layer } = makeReactiveStore('MyStore', initialValue)
 * // In appLayer: Layer.mergeAll(..., Layer)
 * // In component: const value = useReactiveStore(tag, initialValue)
 */
export function makeReactiveStore<A>(
  name: string,
  initial: A,
): {
  tag: Context.Tag<ReactiveStore<A>, ReactiveStore<A>>
  layer: Layer.Layer<ReactiveStore<A>>
} {
  const tag = Context.GenericTag<ReactiveStore<A>>(name)

  const layer = Layer.scoped(
    tag,
    SubscriptionRef.make(initial).pipe(
      Effect.map((ref) => ({
        get: () => ref.get,
        update: (f: (a: A) => A) => SubscriptionRef.update(ref, f),
        changes: ref.changes,
      })),
    ),
  )

  return { tag, layer }
}
