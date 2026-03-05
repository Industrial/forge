import {
  FetchHttpClient,
  HttpClient,
  HttpClientRequest,
  HttpClientResponse,
} from '@effect/platform'
import { Effect, Layer } from 'effect'
import { AuthStateRef, on401HandlerRef } from './authStateRef'

/**
 * Config for app bootstrap: base URL only. Token and scope are read at request time
 * from AuthStateRef (updated by AuthenticationStore). Kept for rehydration and provider API.
 */
export interface HttpClientWithAuthConfig {
  /** Base URL for API requests (e.g. window.location.origin). */
  readonly baseUrl: string
  /** @deprecated Token is read at request time from AuthStateRef. Kept for rehydration only. */
  readonly token?: string
  /** @deprecated Scope is read at request time from AuthStateRef. */
  readonly organizationId?: string
  /** @deprecated Scope is read at request time from AuthStateRef. */
  readonly roleId?: string
}

/**
 * Layer that provides HttpClient with baseUrl and request-time auth/scope headers.
 * Reads current token and scope from AuthStateRef at each request (single runtime, no rebuild on scope switch).
 * Composes FetchHttpClient. Requires AuthStateRef.
 */
export const httpClientWithAuthLayer = (baseUrl: string) =>
  Layer.effect(
    HttpClient.HttpClient,
    Effect.gen(function* () {
      const base = yield* HttpClient.HttpClient
      const authStateRef = yield* AuthStateRef
      const withAuth = HttpClient.mapRequest(base, (req) => {
        let r = HttpClientRequest.prependUrl(req, baseUrl)
        const c = authStateRef.current
        if (c.token != null) {
          r = HttpClientRequest.setHeader(
            r,
            'Authorization',
            `Bearer ${c.token}`,
          )
        }
        if (c.organizationId != null) {
          r = HttpClientRequest.setHeader(
            r,
            'X-Organization-Id',
            c.organizationId,
          )
        }
        if (c.roleId != null) {
          r = HttpClientRequest.setHeader(r, 'X-Role-Id', c.roleId)
        }
        return r
      })
      return {
        ...withAuth,
        execute: (req: HttpClientRequest.HttpClientRequest) =>
          withAuth.execute(req).pipe(
            Effect.tap((res: HttpClientResponse.HttpClientResponse) =>
              res.status === 401
                ? Effect.sync(() => on401HandlerRef.current())
                : Effect.void,
            ),
          ),
      }
    }),
  ).pipe(Layer.provide(FetchHttpClient.layer))
