import { type Runtime, Runtime as RuntimeModule } from 'effect'
import { createContext, useContext, useMemo, type ReactNode } from 'react'

/**
 * Context value holding the Effect runtime used by all react-effect hooks.
 *
 * The runtime is provided once via {@link EffectRuntimeProvider}; hooks call
 * {@link useEffectRuntime} to obtain it and run or fork Effects. This allows
 * a single runtime (optionally with Layers/Services) to be shared across the tree.
 *
 * @typeParam R - The type of requirements (context) the runtime can provide to Effects.
 */
export interface EffectRuntimeContext<R = never> {
  /** The Effect runtime. Use with `Runtime.runPromise(runtime)(effect)` or similar. */
  readonly runtime: Runtime.Runtime<R>
}

const EffectRuntimeContext = createContext<EffectRuntimeContext<never> | null>(
  null,
)

/**
 * Props for {@link EffectRuntimeProvider}.
 *
 * @typeParam R - The type of requirements (context) the runtime provides.
 */
export interface EffectRuntimeProviderProps<R> {
  /**
   * Optional custom runtime. If omitted, Effect's `defaultRuntime` is used (no requirements).
   * Pass a runtime built with Layers when your Effects require services (e.g. HttpClient).
   */
  readonly runtime?: Runtime.Runtime<R>
  /** Child tree that can use react-effect hooks. */
  readonly children: ReactNode
}

const defaultRuntime = RuntimeModule.defaultRuntime as Runtime.Runtime<never>

/**
 * Provides the Effect runtime to the subtree so that hooks can run Effects.
 *
 * Wrap your app (or the subtree that uses react-effect hooks) with this provider.
 * If `runtime` is not passed, Effect's default runtime is used. Pass a custom
 * runtime when your Effects have requirements (e.g. from Layers).
 *
 * @param props - {@link EffectRuntimeProviderProps}
 * @returns A React element that provides the runtime to children.
 *
 * @example
 * ```tsx
 * <EffectRuntimeProvider>
 *   <App />
 * </EffectRuntimeProvider>
 * ```
 *
 * @example
 * ```tsx
 * const runtime = Runtime.defaultRuntime.pipe(
 *   Runtime.provide(liveHttpClientLayer)
 * );
 * <EffectRuntimeProvider runtime={runtime}>
 *   <App />
 * </EffectRuntimeProvider>
 * ```
 */
export function EffectRuntimeProvider<R>({
  runtime,
  children,
}: EffectRuntimeProviderProps<R>) {
  const value = useMemo<EffectRuntimeContext<R>>(
    () => ({ runtime: (runtime ?? defaultRuntime) as Runtime.Runtime<R> }),
    [runtime],
  )
  return (
    <EffectRuntimeContext.Provider value={value as EffectRuntimeContext<never>}>
      {children}
    </EffectRuntimeContext.Provider>
  )
}

/**
 * Returns the Effect runtime from React context.
 *
 * Must be used within a tree wrapped by {@link EffectRuntimeProvider};
 * otherwise throws. Use the returned `runtime` with `Runtime.runPromise(runtime)(effect)`
 * or other Runtime helpers to run Effects.
 *
 * @typeParam R - The type of requirements the runtime provides (defaults to `never`).
 * @returns {@link EffectRuntimeContext} with the current runtime.
 * @throws Error if called outside {@link EffectRuntimeProvider}.
 *
 * @example
 * ```ts
 * const { runtime } = useEffectRuntime();
 * const value = await Runtime.runPromise(runtime)(myEffect);
 * ```
 */
export function useEffectRuntime<R = never>(): EffectRuntimeContext<R> {
  const ctx = useContext(EffectRuntimeContext)
  if (ctx === null) {
    throw new Error(
      'useEffectRuntime must be used within an EffectRuntimeProvider',
    )
  }
  return ctx as EffectRuntimeContext<R>
}
