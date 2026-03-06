/**
 * App-wide Effect layer: auth (HttpClient + AuthenticationStore) + Websocket + ForgeWebsocket.
 * Used by AuthenticationRuntimeProvider to build the single runtime for the app.
 */
import type { HttpClient } from '@effect/platform'
import { Layer, Logger, LogLevel } from 'effect'
import { AuthenticationFeatureLayer } from '@/features/authentication/layer'
import type { AuditLogService } from '@/features/dashboard/services/AuditLog'
import { AuditLogLive } from '@/features/dashboard/services/AuditLogLive'
import type { DashboardService } from '@/features/dashboard/services/Dashboard'
import { DashboardLive } from '@/features/dashboard/services/DashboardLive'
import type { PermissionsService } from '@/features/dashboard/services/Permissions'
import { PermissionsLive } from '@/features/dashboard/services/PermissionsLive'
import type { RolesService } from '@/features/dashboard/services/Roles'
import { RolesLive } from '@/features/dashboard/services/RolesLive'
import type { UsersService } from '@/features/dashboard/services/Users'
import { UsersLive } from '@/features/dashboard/services/UsersLive'
import type { AuthenticationApiService } from '@/features/authentication/services/AuthenticationApi'
import type { AuthenticationStoreService } from '@/features/authentication/services/AuthenticationStore'
import type { EntityApiService } from '@/services/EntityApi'
import { EntityApiLive } from '@/services/EntityApiLive'
import type { RpcApiService } from '@/services/RpcApi'
import { RpcApiLive } from '@/services/RpcApiLive'
import { ForgeWebsocketLive } from '@/services/ForgeWebsocketLive'
import type { SubscriptionStreamService } from '@/services/SubscriptionStream'
import { SubscriptionStreamLive } from '@/services/SubscriptionStreamLive'
import type { ForgeWebsocketService } from '@/services/ForgeWebsocket'
import type { WebsocketService } from '@/services/Websocket'
import { WebsocketLive } from '@/services/WebsocketLive'
import { getBaseUrl } from '@/lib/baseUrl'
import type { AuthenticationStateReactiveStore } from '@/lib/authenticationReactiveStore'
import { AuthenticationStateReactiveStoreLayer } from '@/lib/authenticationReactiveStore'

/**
 * Union of all service types provided by the app runtime.
 * Matches the type inferred from AppLayer. Use for typing effects and runApp so effects run without casts.
 */
export type AppServices =
  | AuthenticationStateReactiveStore
  | HttpClient.HttpClient
  | AuthenticationStoreService
  | AuthenticationApiService
  | EntityApiService
  | RpcApiService
  | SubscriptionStreamService
  | WebsocketService
  | ForgeWebsocketService
  | AuditLogService
  | PermissionsService
  | RolesService
  | UsersService
  | DashboardService

/**
 * Logger layer: minimum level Trace so Effect.logTrace and Effect.logDebug are emitted.
 * Merged into the app runtime so service instrumentation is visible. The default logger
 * in Effect should output to the environment (e.g. browser console when run in the app).
 */
const LoggerLayer = Logger.minimumLogLevel(LogLevel.Trace)

const BaseLayer = Layer.mergeAll(
  AuthenticationFeatureLayer,
  AuthenticationStateReactiveStoreLayer,
  WebsocketLive,
  ForgeWebsocketLive.pipe(Layer.provide(WebsocketLive)),
)

const SubscriptionStreamLayer = SubscriptionStreamLive(getBaseUrl()).pipe(
  Layer.provide(BaseLayer),
)

const DashboardServicesLayer = Layer.mergeAll(
  EntityApiLive,
  RpcApiLive,
  AuditLogLive,
  PermissionsLive,
  RolesLive,
  UsersLive,
  DashboardLive,
).pipe(Layer.provide(BaseLayer))

export const AppLayer = Layer.mergeAll(
  BaseLayer,
  DashboardServicesLayer,
  SubscriptionStreamLayer,
  LoggerLayer,
)
