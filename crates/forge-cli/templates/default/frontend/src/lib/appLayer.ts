/**
 * App-wide Effect layer: auth (HttpClient + AuthenticationStore) + Websocket + ForgeWebsocket.
 * Used by AuthenticationRuntimeProvider to build the single runtime for the app.
 */
import type { HttpClient } from '@effect/platform'
import { Effect, Layer, Logger, LogLevel, Runtime } from 'effect'
import type { HttpClientWithAuthConfig } from './httpClientWithAuth'
import { AuthenticationFeatureLayer } from '../features/authentication/layer'
import type { AuditLogService } from '../features/dashboard/services/AuditLog'
import { AuditLogLive } from '../features/dashboard/services/AuditLogLive'
import type { DashboardService } from '../features/dashboard/services/Dashboard'
import { DashboardLive } from '../features/dashboard/services/DashboardLive'
import type { OrganizationsService } from '../features/dashboard/services/Organizations'
import { OrganizationsLive } from '../features/dashboard/services/OrganizationsLive'
import type { PermissionsService } from '../features/dashboard/services/Permissions'
import { PermissionsLive } from '../features/dashboard/services/PermissionsLive'
import type { RolesService } from '../features/dashboard/services/Roles'
import { RolesLive } from '../features/dashboard/services/RolesLive'
import type { UsersService } from '../features/dashboard/services/Users'
import { UsersLive } from '../features/dashboard/services/UsersLive'
import type { AuthenticationApiService } from '../features/authentication/services/AuthenticationApi'
import type { AuthenticationStoreService } from '../features/authentication/services/AuthenticationStore'
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
  | AuthenticationApiService
  | WebsocketService
  | ForgeWebsocketService
  | OrganizationsService
  | AuditLogService
  | PermissionsService
  | RolesService
  | UsersService
  | DashboardService

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
 * Logger layer: minimum level Trace so Effect.logTrace and Effect.logDebug are emitted.
 * Merged into the app runtime so service instrumentation is visible. The default logger
 * in Effect should output to the environment (e.g. browser console when run in the app).
 */
const LoggerLayer = Logger.minimumLogLevel(LogLevel.Trace)

/**
 * Builds the full app layer: AuthenticationFeatureLayer (HttpClient + AuthenticationStore),
 * WebsocketLive, ForgeWebsocketLive, and OrganizationsLive. ForgeWebsocketLive and
 * OrganizationsLive receive their dependencies from the base layer.
 * Includes Logger at Trace level so instrumentation logs are visible.
 */
export const AppLayer = (config: HttpClientWithAuthConfig) => {
  const base = Layer.mergeAll(
    AuthenticationFeatureLayer(config),
    WebsocketLive,
    ForgeWebsocketLive.pipe(Layer.provide(WebsocketLive)),
  )
  const dashboardServices = Layer.mergeAll(
    OrganizationsLive,
    AuditLogLive,
    PermissionsLive,
    RolesLive,
    UsersLive,
    DashboardLive,
  ).pipe(Layer.provide(base))
  return Layer.mergeAll(base, dashboardServices, LoggerLayer)
}
