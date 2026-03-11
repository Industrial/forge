/**
 * Reactive store: Effect.ts-style state container that notifies subscribers on change.
 *
 * @packageDocumentation
 *
 * Use {@link defineStore} (or {@link makeReactiveStore}) to create a store with in-memory
 * state and a {@link Layer.sync} so the same instance is shared by restoreSession and React.
 * Add the returned layer to your app layer; use {@link useReactiveStore} in React (or a
 * bound hook from your store module) to subscribe. No ref/scoped wiring in main.
 */
import { useMemo, useSyncExternalStore } from 'react'
import {
  Chunk,
  Context,
  Effect,
  type Fiber,
  Layer,
  Option,
  pipe,
  Stream,
} from 'effect'

/**
 * Reactive store interface: current value, updates, and a stream of changes.
 *
 * Implementations are created by {@link defineStore} or {@link makeReactiveStore}. Subscribers run the
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

/** Registry so the layer can push to the React cache as soon as the external store exists (no subscribe needed). */
const syncRegistryByTag = new WeakMap<
  object,
  { setter: ((a: unknown) => void) | null }
>()

/**
 * Defines a reactive store with in-memory state and a {@link Layer.sync} so the same
 * instance is shared by every {@link Effect.provide} (e.g. restoreSession and React).
 * No ref or scoped wiring; add the returned layer to your app layer once.
 *
 * @param name - Unique name for the store (context tag identifier).
 * @param initial - Initial value of the store.
 * @returns `{ tag, layer }`. Use `tag` in effects (`yield* tag`) and merge `layer` into
 *   your app layer. In React, use {@link useReactiveStore}(tag, initial, run, runFork) or
 *   a bound hook in your store module that calls useRunWithAppLayer + useReactiveStore.
 */
export function defineStore<A>(
  name: string,
  initial: A,
): {
  tag: Context.Tag<ReactiveStore<A>, ReactiveStore<A>>
  layer: Layer.Layer<ReactiveStore<A>, never, never>
} {
  const tag = Context.GenericTag<ReactiveStore<A>>(name)
  const registry = { setter: null as ((a: A) => void) | null }
  syncRegistryByTag.set(
    tag as object,
    registry as { setter: ((a: unknown) => void) | null },
  )

  let current: A = initial
  const changeListeners = new Set<(a: A) => void>()

  const notify = (a: A) => {
    current = a
    if (registry.setter) {
      registry.setter(a)
    }
    changeListeners.forEach((l) => l(a))
  }

  const changes = Stream.async<A, never, never>((emit) => {
    emit(Effect.succeed(Chunk.of(current)))
    const listener = (a: A) => {
      emit(Effect.succeed(Chunk.of(a)))
    }
    changeListeners.add(listener)
    return Effect.sync(() => {
      changeListeners.delete(listener)
    })
  })

  const store: ReactiveStore<A> = {
    get: () => Effect.succeed(current),
    update: (f: (a: A) => A) =>
      Effect.sync(() => {
        notify(f(current))
      }),
    changes,
  }

  const layer = Layer.sync(tag, () => store)
  return { tag, layer }
}

/**
 * Creates a layer that provides a {@link ReactiveStore}<A> with the given initial value.
 * Alias for {@link defineStore}; same in-memory state and {@link Layer.sync} behavior.
 */
export const makeReactiveStore = defineStore

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
  let cache: A = initial
  let initialized = false
  const listeners = new Set<() => void>()
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
    }
  }

  const subscribe = (onStoreChange: () => void) => {
    listeners.add(onStoreChange)

    if (!started) {
      started = true

      const initialEffect = Effect.gen(function* () {
        const store = yield* tag
        return yield* store.get()
      })

      run(initialEffect).then((current) => {
        setCache(current)
      })

      const streamEffect = Effect.gen(function* () {
        const store = yield* tag
        yield* Stream.runForEach(store.changes, (a) =>
          Effect.sync(() => {
            setCache(a)
          }),
        )
      })

      runFork(streamEffect)
    }

    return () => {
      listeners.delete(onStoreChange)
    }
  }

  let lastSnapshot: ReactiveStoreSnapshot<A> | null = null
  const serverSnapshot: ReactiveStoreSnapshot<A> = {
    value: initial,
    initialized: false,
  }

  const getSnapshot = (): ReactiveStoreSnapshot<A> => {
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
 * Builds a `useSyncExternalStore`-compatible store that subscribes to a derived stream:
 * `pipe(store.changes, Stream.map(...), Stream.filter(...), ...)`. Use with
 * {@link useDerivedReactiveStore} so components can derive values without creating new stores.
 */
function createDerivedExternalStore<A, B>(
  tag: Context.Tag<ReactiveStore<A>, ReactiveStore<A>>,
  initialB: B,
  _run: RunEffect,
  runFork: RunFork,
  derive: (
    stream: Stream.Stream<A, never, never>,
  ) => Stream.Stream<B, never, never>,
): {
  subscribe: (onStoreChange: () => void) => () => void
  getSnapshot: () => ReactiveStoreSnapshot<B>
  getServerSnapshot: () => ReactiveStoreSnapshot<B>
} {
  let cache: B = initialB
  let initialized = false
  const listeners = new Set<() => void>()
  let started = false

  const setCache = (b: B) => {
    cache = b
    initialized = true
    listeners.forEach((l) => l())
  }

  const subscribe = (onStoreChange: () => void) => {
    listeners.add(onStoreChange)
    if (!started) {
      started = true
      const streamEffect = Effect.gen(function* () {
        const store = yield* tag
        const derived = derive(store.changes)
        yield* Stream.runForEach(derived, (b) => Effect.sync(() => setCache(b)))
      })
      runFork(streamEffect)
    }
    return () => {
      listeners.delete(onStoreChange)
    }
  }

  let lastSnapshot: ReactiveStoreSnapshot<B> | null = null
  const serverSnapshot: ReactiveStoreSnapshot<B> = {
    value: initialB,
    initialized: false,
  }
  const getSnapshot = (): ReactiveStoreSnapshot<B> => {
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
  const getServerSnapshot = (): ReactiveStoreSnapshot<B> => serverSnapshot

  return {
    subscribe,
    getSnapshot,
    getServerSnapshot,
  }
}

/**
 * Cache for derived external stores: keyed by tag then by derivationKey.
 * Same (tag, derivationKey) shares one subscription and cache.
 */
const derivedStoreCache = new WeakMap<
  object,
  Map<string, ReturnType<typeof createDerivedExternalStore<unknown, unknown>>>
>()

/**
 * One external store per tag so all components share the same cache and subscription.
 * Key is the tag (object); value is the store. Using `object` for the key avoids
 * tag type casts; we assert the stored value to ExternalStoreShape<A> when reading.
 */
const externalStoreCache = new WeakMap<object, ExternalStoreShape<unknown>>()

/**
 * Test-only: clear the cached external store for a tag so the next useReactiveStore
 * creates a new store (e.g. after setting application layer override). Only use in tests.
 */
export function clearReactiveStoreCacheForTesting(tag: object): void {
  externalStoreCache.delete(tag)
}

/**
 * Returns the current derived value from a reactive store by subscribing to a
 * transformed stream. Use `pipe(store.changes, Stream.map(...), Stream.filter(...))`
 * so components can map/filter/reduce without creating new stores.
 *
 * The same `derivationKey` across components shares one subscription and cache.
 * Use a stable string that describes the derivation (e.g. `"user"`, `"visibleIds"`).
 *
 * @param tag - Context tag for the source store.
 * @param initialB - Value shown before the first emission from the derived stream.
 * @param run - Run effect to completion (from e.g. useRunWithAppLayer).
 * @param runFork - Run effect in background (from e.g. useRunWithAppLayer).
 * @param derivationKey - Stable string identifying this derivation (used for cache sharing).
 * @param derive - Function that takes the store's `changes` stream and returns a derived stream.
 * @returns The current value of type `B`.
 *
 * @example
 * ```ts
 * const { run, runFork } = useRunWithAppLayer()
 * const user = useDerivedReactiveStore(
 *   AuthStoreTag,
 *   Option.none(),
 *   run,
 *   runFork,
 *   'user',
 *   (s) => pipe(s, Stream.map((a) => a.user)),
 * )
 * ```
 */
export function useDerivedReactiveStore<A, B>(
  tag: Context.Tag<ReactiveStore<A>, ReactiveStore<A>>,
  initialB: B,
  run: RunEffect,
  runFork: RunFork,
  derivationKey: string,
  derive: (
    stream: Stream.Stream<A, never, never>,
  ) => Stream.Stream<B, never, never>,
): B {
  const store = useMemo(() => {
    let byKey = derivedStoreCache.get(tag as object)
    if (!byKey) {
      byKey = new Map()
      derivedStoreCache.set(tag as object, byKey)
    }
    let cached = byKey.get(derivationKey) as
      | ReturnType<typeof createDerivedExternalStore<A, B>>
      | undefined
    if (!cached) {
      cached = createDerivedExternalStore(tag, initialB, run, runFork, derive)
      byKey.set(
        derivationKey,
        cached as ReturnType<
          typeof createDerivedExternalStore<unknown, unknown>
        >,
      )
    }
    return cached
  }, [tag, initialB, run, runFork, derivationKey, derive])

  const snapshot = useSyncExternalStore(
    store.subscribe,
    store.getSnapshot,
    store.getServerSnapshot,
  )
  return snapshot.value
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
