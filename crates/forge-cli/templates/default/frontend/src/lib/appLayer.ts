/**
 * App-wide Effect layer: auth (HttpClient + AuthenticationStore) + Websocket + ForgeWebsocket.
 * Used by AuthenticationRuntimeProvider to build the single runtime for the app.
 */
import type { HttpClient } from '@effect/platform'
import { Effect, Layer, Runtime } from 'effect'
import type { HttpClientWithAuthConfig } from './httpClientWithAuth'
import { AuthenticationFeatureLayer } from '../features/authentication/layer'
import type { OrganizationsService } from '../features/dashboard/services/Organizations'
import { OrganizationsLive } from '../features/dashboard/services/OrganizationsLive'
import type { AuthenticationStoreService } from '../services/AuthenticationStore'
import { ForgeWebsocketLive } from '../services/ForgeWebsocketLive'
import type { ForgeWebsocketService } from '../services/ForgeWebsocket'
import type { WebsocketService } from '../services/Websocket'
import { WebsocketLive } from '../services/WebsocketLive'

/**
 * Union of all service types provided by the app runtime.
 * Matches the type inferred from AppLayer. Use for {@link useEffectRuntime}<AppServices>
 * and for typing the runtime so effects run without casts.
 */
export type AppServices =
  | HttpClient.HttpClient
  | AuthenticationStoreService
  | WebsocketService
  | ForgeWebsocketService
  | OrganizationsService

/**
 * Runs an effect with the app runtime. Use this instead of Runtime.runPromise(runtime)(effect)
 * so that effects requiring only a subset of AppServices type-check without casts at call sites.
 *
 * Single cast boundary: Effect's Runtime.runPromise types require the effect's R to match the
 * runtime's R exactly; our runtime provides AppServices so running an effect that requires
 * R extends AppServices is safe.
 */
export function runWithAppRuntime<A, E, R extends AppServices>(
  runtime: Runtime.Runtime<AppServices>,
  effect: Effect.Effect<A, E, R>,
): Promise<A> {
  return Runtime.runPromise(runtime)(effect as Effect.Effect<A, E, AppServices>)
}

/**
 * Builds the full app layer: AuthenticationFeatureLayer (HttpClient + AuthenticationStore),
 * WebsocketLive, ForgeWebsocketLive, and OrganizationsLive. ForgeWebsocketLive and
 * OrganizationsLive receive their dependencies from the base layer.
 */
export const AppLayer = (config: HttpClientWithAuthConfig) => {
  const base = Layer.mergeAll(
    AuthenticationFeatureLayer(config),
    WebsocketLive,
    ForgeWebsocketLive.pipe(Layer.provide(WebsocketLive)),
  )
  return Layer.mergeAll(base, OrganizationsLive.pipe(Layer.provide(base)))
}
