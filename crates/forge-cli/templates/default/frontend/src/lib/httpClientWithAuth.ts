import {
  FetchHttpClient,
  HttpClient,
  HttpClientRequest,
} from '@effect/platform'
import { Effect, Layer } from 'effect'
import {
  AuthStateRef,
  on401HandlerRef,
  onScopeRequiredRef,
} from './authStateRef'

/** Path prefixes that require scope (X-Organization-Id, X-Role-Id). Epic 1. */
const SCOPED_PATH_PREFIXES = ['/api/entities', '/api/dashboard', '/api/org']
function isScopedPath(url: string): boolean {
  try {
    const path = new URL(url, 'http://x').pathname
    return SCOPED_PATH_PREFIXES.some((p) => path.startsWith(p))
  } catch {
    return false
  }
}

/**
 * Config for app bootstrap: base URL only. Token and scope are read at request time
 * from AuthStateRef (updated by AuthenticationStore). Kept for rehydration and provider API.
 */
export interface HttpClientWithAuthConfig {
  /** Base URL for API requests (e.g. window.location.origin). */
  readonly baseUrl: string
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
      const client = {
        ...withAuth,
        execute: (req: HttpClientRequest.HttpClientRequest) =>
          Effect.gen(function* () {
            const c = authStateRef.current
            const url = (req as unknown as { url: string }).url
            if (
              isScopedPath(url) &&
              c.token != null &&
              c.needs_scope_select &&
              (c.organizationId == null || c.roleId == null)
            ) {
              onScopeRequiredRef.current()
              return yield* Effect.fail(
                new Error(
                  'Scope required; select a scope before using this feature',
                ),
              )
            }
            const res = yield* withAuth.execute(req)
            if (res.status === 401) {
              on401HandlerRef.current()
            }
            return res
          }),
      }
      return client as typeof base
    }),
  ).pipe(Layer.provide(FetchHttpClient.layer))
