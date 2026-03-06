/**
 * Single app runtime held in a module. Components run effects via runApp(effect)
 * instead of getting a runtime from React context (EffectRuntimeProvider / useEffectRuntime).
 */
import { Effect, Runtime } from 'effect'
import type { AppServices } from './appLayer'

let appRuntime: Runtime.Runtime<AppServices> | null = null

export function setAppRuntime(r: Runtime.Runtime<AppServices> | null): void {
  appRuntime = r
}

export function getAppRuntime(): Runtime.Runtime<AppServices> | null {
  return appRuntime
}

/**
 * Runs an effect with the app runtime. Use this instead of passing runtime through
 * context or useRunEffect(..., runtime). Fails if the runtime is not ready yet
 * (e.g. before AuthenticationRuntimeProvider has finished building it).
 */
export function runApp<A, E, R extends AppServices>(
  effect: Effect.Effect<A, E, R>,
): Promise<A> {
  const r = getAppRuntime()
  if (r == null) {
    return Promise.reject(new Error('App runtime not ready'))
  }
  return Runtime.runPromise(r)(effect as Effect.Effect<A, E, AppServices>)
}
