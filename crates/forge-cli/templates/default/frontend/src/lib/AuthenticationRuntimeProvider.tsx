import { Effect, Runtime } from 'effect'
import { type ReactNode, useEffect, useMemo, useRef, useState } from 'react'
import FullPageLoader from '../components/FullPageLoader'
import type { HttpClientWithAuthConfig } from './httpClientWithAuth'
import { AppLayer, type AppServices } from './appLayer'
import { runApp, setAppRuntime } from './appRuntime'
import { AuthenticationStore } from '../features/authentication/services/AuthenticationStore'
import { Websocket } from '../services/Websocket'
import { getWebsocketUrl } from '../services/WebsocketLive'

const LOG = '[AuthRuntimeProvider]'

function getInitialConfig(): HttpClientWithAuthConfig {
  if (typeof window === 'undefined') {
    console.debug(LOG, 'getInitialConfig: SSR, baseUrl=""')
    return { baseUrl: '' }
  }
  const c = {
    baseUrl: window.location.origin,
    token: localStorage.getItem('token') ?? undefined,
    organizationId: localStorage.getItem('currentOrgId') ?? undefined,
    roleId: localStorage.getItem('currentRoleId') ?? undefined,
  }
  console.debug(LOG, 'getInitialConfig', {
    ...c,
    token: c.token ? '(set)' : '(none)',
  })
  return c
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
  console.debug(LOG, 'render', {
    baseUrl: config.baseUrl,
    hasToken: !!config.token,
  })
  const [runtime, setRuntime] = useState<Runtime.Runtime<AppServices> | null>(
    null,
  )
  const cacheRef = useRef<{
    key: string
    runtime: Runtime.Runtime<AppServices>
  } | null>(null)
  const inFlightKeyRef = useRef<string | null>(null)

  useEffect(() => {
    const key = config.baseUrl
    console.debug(LOG, 'useEffect: start', {
      key,
      cachedKey: cacheRef.current?.key,
      inFlight: inFlightKeyRef.current,
    })
    const cached = cacheRef.current
    if (cached?.key === key) {
      console.debug(LOG, 'useEffect: cache hit, setRuntime from cache')
      setAppRuntime(cached.runtime)
      setRuntime(cached.runtime)
      return
    }
    if (inFlightKeyRef.current === key) {
      console.debug(LOG, 'useEffect: same key in flight, skip')
      return
    }
    inFlightKeyRef.current = key
    let cancelled = false
    console.debug(LOG, 'useEffect: building layer and runtime')

    const layer = AppLayer(config)
    const program = Effect.scoped(Layer.toRuntime(layer))
    const rehydrateEffect = Effect.gen(function* () {
      yield* Effect.logTrace(`${LOG} rehydrate: start`)
      const store = yield* AuthenticationStore
      yield* Effect.logDebug(`${LOG} rehydrate: calling store.fetchMe`)
      yield* store.fetchMe(config.token!)
      yield* Effect.logDebug(
        `${LOG} rehydrate: done (token=${config.token != null})`,
      )
    })
    const rehydrate = (r: Runtime.Runtime<AppServices>) => {
      setAppRuntime(r)
      if (config.token) {
        console.debug(LOG, 'rehydrate: running rehydrateEffect with token')
        return runApp(rehydrateEffect).then(() => {
          console.debug(LOG, 'rehydrate: rehydrateEffect resolved')
          return r
        })
      }
      console.debug(LOG, 'rehydrate: no token, skip')
      return Promise.resolve(r)
    }

    const programPromise = Effect.runPromise(
      program as unknown as Effect.Effect<
        Runtime.Runtime<AppServices>,
        unknown,
        never
      >,
    )
    console.debug(LOG, 'useEffect: program (Layer.toRuntime) started')
    programPromise
      .then((r) => {
        console.debug(LOG, 'useEffect: program resolved, running rehydrate')
        return rehydrate(r as Runtime.Runtime<AppServices>)
      })
      .then((r) => {
        if (cancelled) {
          console.debug(LOG, 'useEffect: cancelled, not setting runtime')
          inFlightKeyRef.current = null
          return
        }
        console.debug(LOG, 'useEffect: setting cache and runtime')
        cacheRef.current = { key, runtime: r }
        inFlightKeyRef.current = null
        setRuntime(r)
      })
      .catch((e) => {
        console.warn(LOG, 'useEffect: program or rehydrate failed', e)
        inFlightKeyRef.current = null
      })

    return () => {
      console.debug(LOG, 'useEffect: cleanup (cancelled=true)')
      cancelled = true
      inFlightKeyRef.current = null
    }
  }, [config.baseUrl])

  const connectEffect = useMemo(
    () =>
      Effect.gen(function* () {
        yield* Effect.logTrace(`${LOG} WebSocket connect check: start`)
        const store = yield* AuthenticationStore
        yield* Effect.logDebug(`${LOG} WebSocket: got AuthenticationStore`)
        const state = yield* store.getState()
        yield* Effect.logDebug(
          `${LOG} WebSocket: getState done (hasToken=${state?.token != null})`,
        )
        if (state.token) {
          yield* Effect.logDebug(`${LOG} WebSocket: connecting (has token)`)
          const ws = yield* Websocket
          yield* Effect.logTrace(`${LOG} WebSocket: got Websocket service`)
          yield* ws.connect(getWebsocketUrl(state.token))
          yield* Effect.logDebug(`${LOG} WebSocket: connect started`)
        } else {
          yield* Effect.logDebug(`${LOG} WebSocket: skip (no token)`)
        }
      }).pipe(
        Effect.catchAll((e) =>
          Effect.logWarning(`${LOG} WebSocket connect failed`, e),
        ),
      ),
    [],
  )

  useEffect(() => {
    if (runtime == null) return
    console.debug(LOG, 'connectEffect: running', { hasToken: !!config.token })
    runApp(connectEffect)
  }, [runtime, config.token])

  if (runtime == null) {
    console.debug(LOG, 'render: runtime null -> FullPageLoader')
    return <FullPageLoader />
  }
  return <>{children}</>
}
