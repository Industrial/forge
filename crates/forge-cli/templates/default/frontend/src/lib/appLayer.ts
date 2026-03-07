/**
 * Application layer for Effect.ts dependency injection.
 *
 * @packageDocumentation
 *
 * This module defines the single application-wide Effect layer that provides all
 * runtime services (auth, HTTP, subscription stream, entity/RPC APIs, dashboard services,
 * and the authentication state reactive store). Use {@link getApplicationLayer}
 * to obtain the singleton layer; run effects at boundaries with
 * `Effect.runPromise(effect.pipe(Effect.provide(getApplicationLayer())))`.
 * Use {@link buildApplicationLayer} in tests or when you need a fresh layer or
 * custom composition.
 */
import { FetchHttpClient, type HttpClient } from '@effect/platform'
import { useMemo } from 'react'
import { Effect } from 'effect'
import { Layer, Logger, LogLevel } from 'effect'

import type { AuditLogService } from '@/features/dashboard/services/AuditLog'
import type { Authentication as AuthenticationService } from '@/features/authentication/services/Authentication'
import type { AuthenticationState } from '@/features/authentication/stores/AuthenticationStateReactiveStore'
import type { DashboardService } from '@/features/dashboard/services/Dashboard'
import type { EntityApiService } from '@/services/EntityApi'
import type { PermissionsService } from '@/features/dashboard/services/Permissions'
import type { ReactiveStore, RunEffect, RunFork } from '@/lib/ReactiveStore'
import type { RolesService } from '@/features/dashboard/services/Roles'
import type { RpcApiService } from '@/services/RpcApi'
import type { SubscriptionStreamService } from '@/services/SubscriptionStream'
import type { SubscriptionStreamStatusStore } from '@/lib/subscriptionStreamStatusStore'
import type { TokenStorageService } from '@/services/TokenStorage'
import type { UsersService } from '@/features/dashboard/services/Users'
import { AuditLogLive } from '@/features/dashboard/services/AuditLogLive'
import { AuthenticationLive } from '@/features/authentication/services/AuthenticationLive'
import { DashboardLive } from '@/features/dashboard/services/DashboardLive'
import { EntityApiLive } from '@/services/EntityApiLive'
import { PermissionsLive } from '@/features/dashboard/services/PermissionsLive'
import { RolesLive } from '@/features/dashboard/services/RolesLive'
import { RpcApiLive } from '@/services/RpcApiLive'
import { SubscriptionStreamLive } from '@/services/SubscriptionStreamLive'
import { TokenStorageLive } from '@/services/TokenStorageLive'
import { UsersLive } from '@/features/dashboard/services/UsersLive'
import { getAuthenticationStateStoreLayer } from '@/features/authentication/stores/AuthenticationStateReactiveStore'
import { getBaseUrl } from '@/lib/baseUrl'
import { getSubscriptionStreamStatusStoreLayer } from '@/lib/subscriptionStreamStatusStore'

/**
 * Union of all service types provided by the application layer.
 *
 * Includes authentication, HTTP client, entity/RPC APIs, subscription stream,
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
  | SubscriptionStreamService
  | SubscriptionStreamStatusStore
  | AuditLogService
  | PermissionsService
  | RolesService
  | UsersService
  | DashboardService
  | TokenStorageService
  | ReactiveStore<AuthenticationState>

/** Logger layer: sets minimum log level to Trace. */
const LoggerLayer: Layer.Layer<never, never, never> = Logger.minimumLogLevel(
  LogLevel.Trace,
)

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
 * {@link AuthenticationLive}, subscription stream (SSE) and status store, dashboard
 * services (entity API, RPC API, audit log, permissions, roles, users, dashboard),
 * and logger. The auth store is obtained internally; no arguments are required.
 * Call this once per application (or per test run) when you need a fresh layer.
 *
 * @returns A layer providing all {@link AppServices}. Has no requirements and no
 *   layer construction errors (`Layer<AppServices, never, never>`).
 */
export function buildApplicationLayer() {
  const authStoreLayer = getAuthenticationStateStoreLayer()
  const subscriptionStreamStatusLayer = getSubscriptionStreamStatusStoreLayer()

  const AuthLayer = Layer.mergeAll(
    authStoreLayer,
    subscriptionStreamStatusLayer,
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
    SubscriptionStreamLive(getBaseUrl()).pipe(Layer.provide(authStoreLayer)),
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

  const result = Layer.mergeAll(BaseLayer, DashboardServicesLayer, LoggerLayer)

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
 * Runs an effect with the application layer provided. Use at boundaries (e.g. event
 * handlers) when you need to run an effect outside of a hook. For hooks, prefer
 * {@link useRunWithAppLayer}.
 */
export function runWithAppLayer<A, E, R>(
  effect: Effect.Effect<A, E, R>,
): Promise<A> {
  return Effect.runPromise(
    effect.pipe(Effect.provide(getApplicationLayer())) as Effect.Effect<
      A,
      E,
      never
    >,
  )
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
