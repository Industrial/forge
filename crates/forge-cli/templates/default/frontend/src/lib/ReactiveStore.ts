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
import {
  Context,
  Effect,
  Fiber,
  Layer,
  Option,
  pipe,
  Stream,
  SubscriptionRef,
} from 'effect'

/**
 * Reactive store interface: current value, updates, and a stream of changes.
 *
 * Implementations are created by {@link makeReactiveStore}. Subscribers run the
 * `changes` stream (e.g. via {@link useReactiveStore}) to react to updates.
 *
 * Sync notifier: when the React external store is created it registers a setter in a
 * registry keyed by tag. Every {@link update} calls that setter in the same turn so
 * the React cache sees the new value before the stream runs.
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
/** Registry so the layer can push to the React cache as soon as the external store exists (no subscribe needed). */
const syncRegistryByTag = new WeakMap<
  object,
  { setter: ((a: unknown) => void) | null }
>()

export function makeReactiveStore<A>(
  name: string,
  initial: A,
): {
  tag: Context.Tag<ReactiveStore<A>, ReactiveStore<A>>
  layer: Layer.Layer<ReactiveStore<A>, never, never>
} {
  const tag = Context.GenericTag<ReactiveStore<A>>(name)
  const registry = { setter: null as ((a: A) => void) | null }
  syncRegistryByTag.set(tag as object, registry as { setter: ((a: unknown) => void) | null })

  const layer: Layer.Layer<ReactiveStore<A>, never, never> = Layer.scoped(
    tag,
    SubscriptionRef.make(initial).pipe(
      Effect.map((ref) => ({
        get: () => ref.get,
        update: (f: (a: A) => A) =>
          pipe(
            SubscriptionRef.update(ref, f),
            Effect.flatMap(() => ref.get),
            Effect.tap((value) =>
              Effect.sync(() => {
                const setter = registry.setter
                if (setter) {
                  if (DEBUG) log(`sync update (registry) notifying React cache`)
                  setter(value)
                }
              }),
            ),
            Effect.asVoid,
          ),
        changes: ref.changes,
      })),
    ),
  )

  return { tag, layer }
}

/**
 * Builds a {@link ReactiveStore} from an existing SubscriptionRef, using the same
 * tag (and thus the same sync registry) as {@link makeReactiveStore}. Use this when
 * you need one shared ref for the app lifetime (e.g. create ref in a long-lived scope,
 * then use this store so restoreSession and React both see the same state).
 */
export function createReactiveStoreFromRef<A>(
  ref: SubscriptionRef.SubscriptionRef<A>,
  tag: Context.Tag<ReactiveStore<A>, ReactiveStore<A>>,
): ReactiveStore<A> {
  const registry = syncRegistryByTag.get(tag as object) as
    | { setter: ((a: A) => void) | null }
    | undefined
  return {
    get: () => ref.get,
    update: (f: (a: A) => A) =>
      pipe(
        SubscriptionRef.update(ref, f),
        Effect.flatMap(() => ref.get),
        Effect.tap((value) =>
          Effect.sync(() => {
            if (registry?.setter) {
              if (DEBUG) log(`sync update (registry) notifying React cache`)
              registry.setter(value)
            }
          }),
        ),
        Effect.asVoid,
      ),
    changes: ref.changes,
  }
}

/** Run an effect to completion (e.g. at the boundary with the app layer). */
export type RunEffect = <A, E, R>(effect: Effect.Effect<A, E, R>) => Promise<A>

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
const DEBUG = true
let debugStoreId = 0
const log = (msg: string, ...args: unknown[]) => {
  if (DEBUG) console.log(`[ReactiveStore] ${msg}`, ...args)
}

export type ReactiveStoreSnapshot<A> = { value: A; initialized: boolean }

function createExternalStore<A>(
  tag: Context.Tag<ReactiveStore<A>, ReactiveStore<A>>,
  initial: A,
  run: RunEffect,
  runFork: RunFork,
): {
  subscribe: (onStoreChange: () => void) => () => void
  getSnapshot: () => ReactiveStoreSnapshot<A>
  getServerSnapshot: () => ReactiveStoreSnapshot<A>
} {
  const storeId = `#${++debugStoreId}`
  let cache: A = initial
  let initialized = false
  const listeners = new Set<() => void>()
  let fiber: Fiber.RuntimeFiber<unknown, never> | null = null
  let started = false

  const setCache = (a: A) => {
    cache = a
    initialized = true
    listeners.forEach((l) => l())
  }

  const registry = syncRegistryByTag.get(tag as object)
  if (registry) {
    registry.setter = (a) => {
      setCache(a as A)
      if (DEBUG) log(`sync update (setter) id=${storeId} cacheKeys=${Object.keys(cache as object).join(',')} notifying ${listeners.size} listeners`)
    }
  }

  const subscribe = (onStoreChange: () => void) => {
    listeners.add(onStoreChange)
    log(`subscribe id=${storeId} listeners=${listeners.size} started=${started} cacheKeys=${Object.keys(cache as object).join(',')}`)

    if (!started) {
      started = true
      log(`subscribe id=${storeId} first subscriber: starting initial get + stream`)

      const initialEffect = Effect.gen(function* () {
        const store = yield* tag
        return yield* store.get()
      })

      run(initialEffect).then((current) => {
        setCache(current)
        log(`initial get completed id=${storeId} cacheKeys=${Object.keys(cache as object).join(',')} notifying ${listeners.size} listeners`)
      })

      const streamEffect = Effect.gen(function* () {
        const store = yield* tag
        yield* Stream.runForEach(store.changes, (a) =>
          Effect.sync(() => {
            setCache(a)
            log(`stream update id=${storeId} cacheKeys=${Object.keys(cache as object).join(',')} notifying ${listeners.size} listeners`)
          }),
        )
      })

      fiber = runFork(streamEffect)
      log(`subscribe id=${storeId} stream fiber forked`)
    }

    return () => {
      listeners.delete(onStoreChange)
      log(`unsubscribe id=${storeId} listeners=${listeners.size}`)

      if (listeners.size === 0 && fiber != null) {
        log(`unsubscribe id=${storeId} last listener: keeping stream running (no interrupt)`)
      }
    }
  }

  let lastSnapshot: ReactiveStoreSnapshot<A> | null = null
  const serverSnapshot: ReactiveStoreSnapshot<A> = {
    value: initial,
    initialized: false,
  }

  let getSnapshotCallCount = 0
  const getSnapshot = (): ReactiveStoreSnapshot<A> => {
    getSnapshotCallCount++
    if (getSnapshotCallCount <= 3 || getSnapshotCallCount % 20 === 0) {
      log(`getSnapshot id=${storeId} call#=${getSnapshotCallCount} cacheKeys=${Object.keys(cache as object).join(',')} initialized=${initialized} listeners=${listeners.size}`)
    }
    if (
      lastSnapshot !== null &&
      lastSnapshot.value === cache &&
      lastSnapshot.initialized === initialized
    ) {
      return lastSnapshot
    }
    lastSnapshot = { value: cache, initialized }
    return lastSnapshot
  }
  const getServerSnapshot = (): ReactiveStoreSnapshot<A> => serverSnapshot

  return {
    subscribe,
    getSnapshot,
    getServerSnapshot,
  }
}

/** Shape of the external store returned by createExternalStore (used for cache typing). */
type ExternalStoreShape<A> = ReturnType<typeof createExternalStore<A>>

/**
 * One external store per tag so all components share the same cache and subscription.
 * Key is the tag (object); value is the store. Using `object` for the key avoids
 * tag type casts; we assert the stored value to ExternalStoreShape<A> when reading.
 */
const externalStoreCache = new WeakMap<object, ExternalStoreShape<unknown>>()

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
    () =>
      pipe(
        Option.fromNullable(externalStoreCache.get(tag)),
        Option.match({
          onSome: (cached) => {
            log('useReactiveStore cache HIT tag=', tag)
            return cached as ExternalStoreShape<A>
          },
          onNone: () => {
            log('useReactiveStore cache MISS creating new external store tag=', tag)
            const created = createExternalStore(tag, initial, run, runFork)
            externalStoreCache.set(tag, created)
            return created
          },
        }),
      ),
    [tag, initial, run, runFork],
  )

  const snapshot = useSyncExternalStore(
    store.subscribe,
    store.getSnapshot,
    store.getServerSnapshot,
  )
  return snapshot.value
}

/**
 * Like {@link useReactiveStore} but returns both value and whether the store has
 * received at least one value from the ref (sync update or initial get). Use this
 * when you must not treat "no user" as "redirect" until the store has initialized
 * (e.g. after restoreSession or first subscribe).
 */
export function useReactiveStoreWithInit<A>(
  tag: Context.Tag<ReactiveStore<A>, ReactiveStore<A>>,
  initial: A,
  run: RunEffect,
  runFork: RunFork,
): ReactiveStoreSnapshot<A> {
  const store = useMemo(
    () =>
      pipe(
        Option.fromNullable(externalStoreCache.get(tag)),
        Option.match({
          onSome: (cached) => cached as ExternalStoreShape<A>,
          onNone: () => {
            const created = createExternalStore(tag, initial, run, runFork)
            externalStoreCache.set(tag, created)
            return created
          },
        }),
      ),
    [tag, initial, run, runFork],
  )

  return useSyncExternalStore(
    store.subscribe,
    store.getSnapshot,
    store.getServerSnapshot,
  )
}
