import { HttpClient } from '@effect/platform'
import { Effect, Layer, Runtime } from 'effect'
import { type ReactNode, useEffect, useMemo, useState } from 'react'
import { EffectRuntimeProvider } from './react-effect'
import {
  httpClientWithAuthLayer,
  type HttpClientWithAuthConfig,
} from './httpClientWithAuth'

export interface AppRuntimeProviderProps extends HttpClientWithAuthConfig {
  readonly children: ReactNode
}

/**
 * Provider that builds the Effect runtime with HttpClient (from @effect/platform)
 * configured with baseUrl and auth headers. Use at app root so effects that
 * depend on HttpClient get the configured client.
 */
export function AppRuntimeProvider({
  baseUrl,
  token,
  organizationId,
  roleId,
  children,
}: AppRuntimeProviderProps) {
  const config = useMemo(
    () => ({ baseUrl, token, organizationId, roleId }),
    [baseUrl, token, organizationId, roleId],
  )
  const [runtime, setRuntime] =
    useState<Runtime.Runtime<HttpClient.HttpClient> | null>(null)

  useEffect(() => {
    const layer = httpClientWithAuthLayer(config)
    const program = Effect.scoped(
      Effect.gen(function* () {
        return yield* Layer.toRuntime(layer)
      }),
    )
    Effect.runPromise(program).then(setRuntime)
  }, [config.baseUrl, config.token, config.organizationId, config.roleId])

  if (runtime == null) {
    return null
  }
  return (
    <EffectRuntimeProvider runtime={runtime}>{children}</EffectRuntimeProvider>
  )
}
