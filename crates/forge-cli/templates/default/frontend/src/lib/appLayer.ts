/**
 * Application layer for Effect.ts dependency injection.
 *
 * @packageDocumentation
 *
 * This module defines the single application-wide Effect layer that provides all
 * runtime services (auth, HTTP, WebSocket, entity/RPC APIs, dashboard services,
 * and the authentication state reactive store). Use {@link getApplicationLayer}
 * to obtain the singleton layer; run effects at boundaries with
 * `Effect.runPromise(effect.pipe(Effect.provide(getApplicationLayer())))`.
 * Use {@link buildApplicationLayer} in tests or when you need a fresh layer or
 * custom composition.
 */
import { FetchHttpClient, type HttpClient } from '@effect/platform'
import { Layer, Logger, LogLevel } from 'effect'

import type { AuditLogService } from '@/features/dashboard/services/AuditLog'
import type { Authentication as AuthenticationService } from '@/features/authentication/services/Authentication'
import type { AuthenticationState } from '@/features/authentication/stores'
import type { DashboardService } from '@/features/dashboard/services/Dashboard'
import type { EntityApiService } from '@/services/EntityApi'
import type { ForgeWebsocketService } from '@/services/ForgeWebsocket'
import type { PermissionsService } from '@/features/dashboard/services/Permissions'
import type { ReactiveStore, RunEffect, RunFork } from '@/lib/ReactiveStore'
import type { RolesService } from '@/features/dashboard/services/Roles'
import type { RpcApiService } from '@/services/RpcApi'
import type { TokenStorageService } from '@/services/TokenStorage'
import type { UsersService } from '@/features/dashboard/services/Users'
import type { WebsocketService } from '@/services/Websocket'
import { AuditLogLive } from '@/features/dashboard/services/AuditLogLive'
import { AuthenticationLive } from '@/features/authentication/services/AuthenticationLive'
import { DashboardLive } from '@/features/dashboard/services/DashboardLive'
import { EntityApiLive } from '@/services/EntityApiLive'
import { ForgeWebsocketLive } from '@/services/ForgeWebsocketLive'
import { PermissionsLive } from '@/features/dashboard/services/PermissionsLive'
import { RolesLive } from '@/features/dashboard/services/RolesLive'
import { RpcApiLive } from '@/services/RpcApiLive'
import { TokenStorageLive } from '@/services/TokenStorageLive'
import { UsersLive } from '@/features/dashboard/services/UsersLive'
import { WebsocketLive } from '@/services/WebsocketLive'
import { useMemo } from 'react'
import { Effect } from 'effect'

import { getAuthenticationStateStoreLayer } from '@/features/authentication/stores'

/**
 * Union of all service types provided by the application layer.
 *
 * Includes authentication, HTTP client, entity/RPC APIs, WebSocket services,
 * dashboard services (audit log, permissions, roles, users, dashboard), and
 * {@link ReactiveStore}<{@link AuthenticationState}. Including
 * `ReactiveStore<any>` allows effects that depend on a reactive store (e.g.
 * from `useReactiveStore`) to type-check when provided
 * {@link getApplicationLayer}.
 */
export type AppServices =
  | AuthenticationService
  | HttpClient.HttpClient
  | EntityApiService
  | RpcApiService
  | WebsocketService
  | ForgeWebsocketService
  | AuditLogService
  | PermissionsService
  | RolesService
  | UsersService
  | DashboardService
  | TokenStorageService
  | ReactiveStore<AuthenticationState>

/** Logger layer: sets minimum log level to Trace. */
const LoggerLayer: Layer.Layer<never, never, never> =
  Logger.minimumLogLevel(LogLevel.Trace)

/**
 * Plain HTTP client layer (no auth headers).
 * Used by {@link AuthenticationLive} and other services that need unauthenticated HTTP.
 */
const HttpClientLayer: Layer.Layer<HttpClient.HttpClient, never, never> =
  FetchHttpClient.layer

/**
 * Builds the full application layer.
 *
 * Composes auth store (from {@link getAuthenticationStateStoreLayer}), HTTP client,
 * {@link AuthenticationLive}, WebSocket and Forge WebSocket services, dashboard
 * services (entity API, RPC API, audit log, permissions, roles, users, dashboard),
 * and logger. The auth store is obtained internally; no arguments are required.
 * Call this once per application (or per test run) when you need a fresh layer.
 *
 * @returns A layer providing all {@link AppServices}. Has no requirements and no
 *   layer construction errors (`Layer<AppServices, never, never>`).
 */
export function buildApplicationLayer() {
  const authStoreLayer = getAuthenticationStateStoreLayer()

  const AuthLayer = Layer.mergeAll(
    authStoreLayer,
    HttpClientLayer,
    TokenStorageLive,
    AuthenticationLive.pipe(
      Layer.provide(authStoreLayer),
      Layer.provide(HttpClientLayer),
      Layer.provide(TokenStorageLive),
    ),
  )

  const BaseLayer = Layer.mergeAll(
    AuthLayer,
    WebsocketLive,
    ForgeWebsocketLive.pipe(Layer.provide(WebsocketLive)),
  )

  const DashboardServicesLayer = Layer.mergeAll(
    EntityApiLive,
    RpcApiLive,
    AuditLogLive,
    PermissionsLive,
    RolesLive,
    UsersLive,
    DashboardLive,
  )
    .pipe(Layer.provide(BaseLayer))

  const result = Layer.mergeAll(
    BaseLayer,
    DashboardServicesLayer,
    LoggerLayer,
  )

  return result
}

/** Cached singleton application layer; built lazily by {@link getApplicationLayer}. */
let applicationLayer: Layer.Layer<AppServices, never, never> | undefined

/**
 * Returns the application layer singleton, building it on first call.
 *
 * Use this when running effects at boundaries (e.g. in event handlers or
 * React context): provide the layer so the effect has access to all
 * {@link AppServices}:
 *
 * @example
 * ```ts
 * Effect.runPromise(effect.pipe(Effect.provide(getApplicationLayer())))
 * ```
 *
 * @returns The application layer (same instance on every call after the first).
 */
export function getApplicationLayer() {
  if (applicationLayer === undefined) {
    applicationLayer = buildApplicationLayer()
  }
  return applicationLayer
}

/**
 * Hook that returns stable {@link RunEffect} and {@link RunFork} functions that
 * run effects with the application layer provided. Use these when calling
 * {@link useReactiveStore} so the store subscription runs in the app layer.
 */
export function useRunWithAppLayer(): { run: RunEffect; runFork: RunFork } {
  return useMemo(() => {
    const layer = getApplicationLayer()
    // Cast: TypeScript infers Effect<A, E, Exclude<R, AppServices>> after provide;
    // we assert the layer satisfies R so runPromise/runFork accept it. Callers
    // should only pass effects whose requirements are in AppServices.
    // TODO: I'm predicting now that this will become a problem at some point.
    return {
      run: <A, E, R>(effect: Effect.Effect<A, E, R>): Promise<A> =>
        Effect.runPromise(
          effect.pipe(Effect.provide(layer)) as Effect.Effect<A, E, never>,
        ),
      runFork: <A, E, R>(effect: Effect.Effect<A, E, R>) =>
        Effect.runFork(
          effect.pipe(Effect.provide(layer)) as Effect.Effect<A, E, never>,
        ),
    }
  }, [])
}
