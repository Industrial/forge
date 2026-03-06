/**
 * Reactive store: Effect.ts-style state container that notifies subscribers on change.
 *
 * @packageDocumentation
 *
 * Built from Effect's {@link https://effect.website/docs/guides/essentials/refs | SubscriptionRef}
 * (a Ref plus a stream of changes). Use {@link makeReactiveStore} to create a layer and a
 * context tag; add the layer to your app layer. Use {@link useReactiveStore} in React to
 * subscribe and re-render when the store changes; the caller must pass {@link run} and
 * {@link runFork} that provide the layer containing the store (e.g. from an app-level hook).
 */
import { useMemo, useSyncExternalStore } from 'react'
import { Context, Effect, Fiber, Layer, Stream, SubscriptionRef } from 'effect'

/**
 * Reactive store interface: current value, updates, and a stream of changes.
 *
 * Implementations are created by {@link makeReactiveStore}. Subscribers run the
 * `changes` stream (e.g. via {@link useReactiveStore}) to react to updates.
 *
 * @typeParam A - The type of the stored value.
 */
export interface ReactiveStore<A> {
  /** Returns the current value. */
  readonly get: () => Effect.Effect<A, never, never>

  /** Updates state with a function; all subscribers see the new value via `changes`. */
  readonly update: (f: (a: A) => A) => Effect.Effect<void, never, never>

  /** Stream of the current value and every subsequent update. Use for subscription. */
  readonly changes: Stream.Stream<A, never, never>
}

/**
 * Creates a layer that provides a {@link ReactiveStore}<A> with the given initial value.
 *
 * Use the returned `tag` in effects (`yield* tag`) and include the returned `layer` in your
 * app's layer composition (e.g. merge into the application layer). In React components,
 * use the same `tag` with {@link useReactiveStore}(tag, initial) to read and subscribe.
 *
 * @param name - Unique name for the store (used as the context tag identifier).
 * @param initial - Initial value of the store.
 * @returns An object with `tag` (for requiring the store in effects and for
 *   {@link useReactiveStore}) and `layer` (to merge into your app layer).
 *
 * @example
 * ```ts
 * const { tag, layer } = makeReactiveStore('MyStore', initialValue)
 * // In app layer: Layer.mergeAll(..., layer)
 * // In component: const value = useReactiveStore(tag, initialValue)
 * ```
 */
export function makeReactiveStore<A>(
  name: string,
  initial: A,
): {
  tag: Context.Tag<ReactiveStore<A>, ReactiveStore<A>>
  layer: Layer.Layer<ReactiveStore<A>, never, never>
} {
  const tag = Context.GenericTag<ReactiveStore<A>>(name)

  const layer: Layer.Layer<ReactiveStore<A>, never, never> = Layer.scoped(
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

/** Run an effect to completion (e.g. at the boundary with the app layer). */
export type RunEffect = <A, E, R>(
  effect: Effect.Effect<A, E, R>,
) => Promise<A>

/** Run an effect in the background; returns a fiber (interrupt to cancel). */
export type RunFork = <A, E, R>(
  effect: Effect.Effect<A, E, R>,
) => Fiber.RuntimeFiber<A, E>

/**
 * Builds a `useSyncExternalStore`-compatible store that reads and subscribes to a
 * {@link ReactiveStore}. Internal helper for {@link useReactiveStore}. The caller
 * supplies {@link run} and {@link runFork} so the store can run effects (e.g. with
 * the app layer provided); this module does not depend on any app or layer.
 *
 * @param tag - Context tag for the reactive store.
 * @param initial - Value used before first read and for `getServerSnapshot`.
 * @param run - Run effect to completion (used for initial get).
 * @param runFork - Run effect in background (used for changes stream).
 * @returns Object with `subscribe`, `getSnapshot`, and `getServerSnapshot`.
 */
function createExternalStore<A>(
  tag: Context.Tag<ReactiveStore<A>, ReactiveStore<A>>,
  initial: A,
  run: RunEffect,
  runFork: RunFork,
): {
  subscribe: (onStoreChange: () => void) => () => void
  getSnapshot: () => A
  getServerSnapshot: () => A
} {
  let cache: A = initial
  const listeners = new Set<() => void>()
  let fiber: Fiber.RuntimeFiber<unknown, never> | null = null
  let started = false

  const subscribe = (onStoreChange: () => void) => {
    listeners.add(onStoreChange)

    if (!started) {
      started = true

      const initialEffect = Effect.gen(function* () {
        const store = yield* tag
        return yield* store.get()
      })

      run(initialEffect).then((current) => {
        cache = current
        listeners.forEach((l) => l())
      })

      const streamEffect = Effect.gen(function* () {
        const store = yield* tag
        yield* Stream.runForEach(store.changes, (a) =>
          Effect.sync(() => {
            cache = a
            listeners.forEach((l) => l())
          }),
        )
      })

      fiber = runFork(streamEffect)
    }

    return () => {
      listeners.delete(onStoreChange)

      if (listeners.size === 0 && fiber != null) {
        Effect.runPromise(Fiber.interrupt(fiber))
        fiber = null
        started = false
      }
    }
  }

  const getSnapshot = () => cache
  const getServerSnapshot = () => initial

  return {
    subscribe,
    getSnapshot,
    getServerSnapshot,
  }
}

/**
 * Returns the current value of the reactive store and re-renders when it changes.
 *
 * The store identified by `tag` must be provided by whatever layer (or context)
 * is used when {@link run} and {@link runFork} execute effects. Use the same
 * `initial` value as passed to `makeReactiveStore` so that the first render and
 * SSR have a consistent value before the store is read.
 *
 * Uses React's `useSyncExternalStore` so the component always sees the latest
 * value and subscriptions are cleaned up on unmount.
 *
 * @param tag - Context tag for the store (from {@link makeReactiveStore}).
 * @param initial - Value shown before the first value is read from the store.
 * @param run - Run effect to completion (e.g. from an app hook that provides the layer).
 * @param runFork - Run effect in background (e.g. from the same app hook).
 * @returns The current value of type `A`.
 *
 * @example
 * ```ts
 * const { run, runFork } = useRunWithAppLayer()
 * const value = useReactiveStore(tag, initialValue, run, runFork)
 * ```
 */
export function useReactiveStore<A>(
  tag: Context.Tag<ReactiveStore<A>, ReactiveStore<A>>,
  initial: A,
  run: RunEffect,
  runFork: RunFork,
): A {
  const store = useMemo(
    () => createExternalStore(tag, initial, run, runFork),
    [tag, initial, run, runFork],
  )

  return useSyncExternalStore(
    store.subscribe,
    store.getSnapshot,
    store.getServerSnapshot,
  )
}
