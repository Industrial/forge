/**
 * Subscribes to a ReactiveStore and re-renders when the store changes.
 * Uses React's useSyncExternalStore so the component always sees the latest value
 * and subscriptions are cleaned up on unmount.
 *
 * The store must be provided by the app runtime (add the layer from makeReactiveStore
 * to your AppLayer). If the runtime is not ready, returns the initial value.
 *
 * @see ReactiveStore, makeReactiveStore
 */

import { useMemo, useSyncExternalStore } from 'react'
import { Context, Effect, Fiber, Runtime, Stream } from 'effect'
import type { ReactiveStore } from './ReactiveStore'
import { getAppRuntime, runApp } from './appRuntime'

function createExternalStore<A>(
  tag: Context.Tag<ReactiveStore<A>, ReactiveStore<A>>,
  initial: A,
): {
  subscribe: (onStoreChange: () => void) => () => void
  getSnapshot: () => A
  getServerSnapshot: () => A
} {
  let cache: A = initial
  const listeners = new Set<() => void>()
  let fiber: Fiber.RuntimeFiber<never, never> | null = null
  let started = false

  const subscribe = (onStoreChange: () => void) => {
    listeners.add(onStoreChange)

    if (!started) {
      started = true
      const runtime = getAppRuntime()
      if (runtime != null) {
        const effect = Effect.gen(function* () {
          const store = yield* tag
          const current = yield* store.get()
          cache = current
          listeners.forEach((l) => l())
          yield* Stream.runForEach(store.changes, (a) =>
            Effect.sync(() => {
              cache = a
              listeners.forEach((l) => l())
            }),
          )
        })
        fiber = Runtime.runFork(runtime as Runtime.Runtime<never>)(
          effect as Effect.Effect<never, never, never>,
        )
      }
    }

    return () => {
      listeners.delete(onStoreChange)
      if (listeners.size === 0 && fiber != null) {
        runApp(Fiber.interrupt(fiber))
        fiber = null
        started = false
      }
    }
  }

  const getSnapshot = () => cache
  const getServerSnapshot = () => initial

  return { subscribe, getSnapshot, getServerSnapshot }
}

/**
 * Returns the current value of the reactive store and re-renders when it changes.
 * Use with a tag and initial value from makeReactiveStore. The store's layer must
 * be included in your app's Layer (e.g. AppLayer).
 *
 * @param tag - The Context tag for the store (from makeReactiveStore)
 * @param initial - Initial value shown before the first value is read from the store
 */
export function useReactiveStore<A>(
  tag: Context.Tag<ReactiveStore<A>, ReactiveStore<A>>,
  initial: A,
): A {
  const store = useMemo(
    () => createExternalStore(tag, initial),
    [tag, initial],
  )

  return useSyncExternalStore(
    store.subscribe,
    store.getSnapshot,
    store.getServerSnapshot,
  )
}
