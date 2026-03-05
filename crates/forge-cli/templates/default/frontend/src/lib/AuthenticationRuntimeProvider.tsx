import { Effect, Layer, Runtime } from 'effect'
import { type ReactNode, useEffect, useMemo, useRef, useState } from 'react'
import { EffectRuntimeProvider, useRunEffect } from 'react-effect-hooks'
import FullPageLoader from '../components/FullPageLoader'
import type { HttpClientWithAuthConfig } from './httpClientWithAuth'
import { AppLayer, runWithAppRuntime, type AppServices } from './appLayer'
import { AuthenticationStore } from '../features/authentication/services/AuthenticationStore'
import { Websocket } from '../services/Websocket'
import { getWebsocketUrl } from '../services/WebsocketLive'

function getInitialConfig(): HttpClientWithAuthConfig {
  if (typeof window === 'undefined') {
    return { baseUrl: '' }
  }
  return {
    baseUrl: window.location.origin,
    token: localStorage.getItem('token') ?? undefined,
    organizationId: localStorage.getItem('currentOrgId') ?? undefined,
    roleId: localStorage.getItem('currentRoleId') ?? undefined,
  }
}

function configKey(c: HttpClientWithAuthConfig): string {
  return `${c.baseUrl}|${c.token ?? ''}|${c.organizationId ?? ''}|${c.roleId ?? ''}`
}

export interface AuthenticationRuntimeProviderProps {
  /**
   * Optional config for HttpClient + auth. If omitted, baseUrl and token/org/role
   * are read from window.location and localStorage so the runtime can be built once.
   * Pass config when you need to rebuild the runtime after login/scope change.
   */
  readonly config?: HttpClientWithAuthConfig
  readonly children: ReactNode
}

/**
 * Builds the Effect runtime with AppLayer (Auth + Websocket + ForgeWebsocket)
 * and provides it via EffectRuntimeProvider. Rehydrates auth when a token exists;
 * connects the WebSocket when the user has a token.
 */
export function AuthenticationRuntimeProvider({
  config: configProp,
  children,
}: AuthenticationRuntimeProviderProps) {
  const initialConfig = useMemo(getInitialConfig, [])
  const config = configProp ?? initialConfig
  const [runtime, setRuntime] = useState<Runtime.Runtime<AppServices> | null>(
    null,
  )
  const cacheRef = useRef<{
    key: string
    runtime: Runtime.Runtime<AppServices>
  } | null>(null)
  const inFlightKeyRef = useRef<string | null>(null)

  useEffect(() => {
    const key = configKey(config)
    const cached = cacheRef.current
    if (cached?.key === key) {
      setRuntime(cached.runtime)
      return
    }
    if (inFlightKeyRef.current === key) {
      return
    }
    inFlightKeyRef.current = key
    let cancelled = false

    const layer = AppLayer(config)
    const program = Effect.scoped(Layer.toRuntime(layer))
    const rehydrateEffect = Effect.gen(function* () {
      yield* Effect.logTrace('AuthenticationRuntimeProvider: rehydrate')
      const store = yield* AuthenticationStore
      yield* store.fetchMe(config.token!)
      yield* Effect.logDebug(
        `AuthenticationRuntimeProvider: rehydrate done (token=${config.token != null})`,
      )
    })
    const rehydrate = (r: Runtime.Runtime<AppServices>) =>
      config.token
        ? runWithAppRuntime(r, rehydrateEffect).then(() => r)
        : Promise.resolve(r)

    Effect.runPromise(program)
      .then((r) => rehydrate(r as Runtime.Runtime<AppServices>))
      .then((r) => {
        if (cancelled) {
          inFlightKeyRef.current = null
          return
        }
        cacheRef.current = { key, runtime: r }
        inFlightKeyRef.current = null
        setRuntime(r)
      })
      .catch(() => {
        inFlightKeyRef.current = null
      })

    return () => {
      cancelled = true
      inFlightKeyRef.current = null
    }
  }, [config.baseUrl, config.token, config.organizationId, config.roleId])

  const connectEffect = useMemo(
    () =>
      Effect.gen(function* () {
        yield* Effect.logTrace(
          'AuthenticationRuntimeProvider: WebSocket connect check',
        )
        const store = yield* AuthenticationStore
        const state = yield* store.getState()
        if (state.token) {
          yield* Effect.logDebug(
            'AuthenticationRuntimeProvider: connecting WebSocket (has token)',
          )
          const ws = yield* Websocket
          yield* ws.connect(getWebsocketUrl(state.token))
          yield* Effect.logDebug(
            'AuthenticationRuntimeProvider: WebSocket connect started',
          )
        } else {
          yield* Effect.logDebug(
            'AuthenticationRuntimeProvider: skip WebSocket (no token)',
          )
        }
      }).pipe(
        Effect.catchAll((e) =>
          Effect.logWarning(
            'AuthenticationRuntimeProvider: WebSocket connect failed',
            e,
          ),
        ),
      ),
    [],
  )

  useRunEffect(connectEffect, [runtime, config.token], runtime)

  if (runtime == null) {
    return <FullPageLoader />
  }
  return (
    <EffectRuntimeProvider runtime={runtime}>{children}</EffectRuntimeProvider>
  )
}
