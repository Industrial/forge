import { useEffect } from 'react'
import { Effect } from 'effect'
import { getApplicationLayer } from '@/lib/appLayer'

/**
 * Logs component mount/unmount via Effect.ts Logger (Trace level).
 * Use at the top of any component to see lifecycle in the console when
 * the app layer's minimum log level is Trace (see appLayer LoggerLayer).
 */
export function useComponentLogger(componentName: string): void {
  useEffect(() => {
    const layer = getApplicationLayer()
    Effect.runFork(
      Effect.logTrace(`${componentName}: mounted`).pipe(
        Effect.provide(layer),
      ) as Effect.Effect<void, never, never>,
    )
    return () => {
      Effect.runFork(
        Effect.logTrace(`${componentName}: unmounted`).pipe(
          Effect.provide(layer),
        ) as Effect.Effect<void, never, never>,
      )
    }
  }, [componentName])
}
