import { Effect, Layer, Runtime } from 'effect'
import { type ReactNode, useEffect, useMemo, useState } from 'react'
import { EffectRuntimeProvider } from './react-effect'
import type { HttpClientWithAuthConfig } from './httpClientWithAuth'
import { AppLayer, runWithAppRuntime, type AppServices } from './appLayer'
import { AuthenticationStore } from '../features/authentication/services/AuthenticationStore'
import { Websocket } from '../services/Websocket'
import { getWsUrl } from '../services/WebsocketLive'

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
 * and provides it via EffectRuntimeProvider. Connects the WebSocket when the user has a token.
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

  useEffect(() => {
    const layer = AppLayer(config)
    const program = Effect.scoped(
      Effect.gen(function* () {
        return yield* Layer.toRuntime(layer)
      }),
    )
    // Layer.toRuntime infers a union type; we know AppLayer provides exactly AppServices
    Effect.runPromise(program).then((r) =>
      setRuntime(r as Runtime.Runtime<AppServices>),
    )
  }, [config.baseUrl, config.token, config.organizationId, config.roleId])

  // Connect WebSocket when runtime is ready and user has a token
  useEffect(() => {
    if (runtime == null || !config.token) return
    const connectEffect = Effect.gen(function* () {
      const store = yield* AuthenticationStore
      const state = yield* store.getState()
      if (state.token) {
        const ws = yield* Websocket
        yield* ws.connect(getWsUrl())
      }
    })
    runWithAppRuntime(runtime, connectEffect).catch(() => {
      // Connection may fail (e.g. no backend); ignore
    })
  }, [runtime, config.token])

  if (runtime == null) {
    return null
  }
  return (
    <EffectRuntimeProvider runtime={runtime}>{children}</EffectRuntimeProvider>
  )
}
