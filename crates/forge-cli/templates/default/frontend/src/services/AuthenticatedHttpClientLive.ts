/**
 * HTTP client that adds Authorization and scope headers from TokenStorage to every request.
 * Use this for EntityApi and RpcApi so dashboard requests are authenticated.
 * Depends on HttpClient (plain fetch) and TokenStorage.
 */
import { HttpClient, HttpClientRequest } from '@effect/platform'
import { Effect, Layer, Option, pipe } from 'effect'

import { AuthenticatedHttpClient } from '@/services/AuthenticatedHttpClient'
import { TokenStorage } from '@/services/TokenStorage'

function withAuthHeaders<A extends HttpClientRequest.HttpClientRequest>(
  request: A,
  token: string | null,
  scope: { organizationId: string; roleId: string } | null,
): A {
  let req = request
  if (token != null && token !== '') {
    req = pipe(
      req,
      HttpClientRequest.setHeader('Authorization', `Bearer ${token}`),
    ) as A
  }
  if (scope != null) {
    req = pipe(
      req,
      HttpClientRequest.setHeader('x-organization-id', scope.organizationId),
      HttpClientRequest.setHeader('x-role-id', scope.roleId),
    ) as A
  }
  return req
}

/**
 * Layer that provides AuthenticatedHttpClient by wrapping HttpClient (plain)
 * and adding token + scope headers from TokenStorage to each request.
 */
export const AuthenticatedHttpClientLive = Layer.effect(
  AuthenticatedHttpClient,
  Effect.gen(function* () {
    const base = yield* HttpClient.HttpClient
    const tokenStorage = yield* TokenStorage

    const execute = (request: HttpClientRequest.HttpClientRequest) =>
      Effect.gen(function* () {
        yield* Effect.logTrace('AuthenticatedHttpClientLive.execute')
        const tokenOpt = yield* tokenStorage.getToken()
        const scopeOpt = yield* tokenStorage.getScope()
        const token = Option.getOrElse(tokenOpt, () => null)
        const scope = Option.getOrElse(scopeOpt, () => null)
        const url =
          typeof request === 'object' && request && 'url' in request
            ? String((request as { url?: string }).url ?? '')
            : ''
        yield* Effect.logDebug(
          `AuthenticatedHttpClientLive.execute: url=${url || '(unknown)'}, hasToken=${token != null && token !== ''}, hasScope=${scope != null}`,
        )
        const req = withAuthHeaders(request, token, scope)
        const response = yield* base.execute(req)
        yield* Effect.logDebug(
          `AuthenticatedHttpClientLive.execute: response status=${(response as { status?: number }).status ?? 'unknown'}`,
        )
        return response
      })

    return { ...base, execute }
  }),
)
