/**
 * Live implementation of RpcApi using HttpClient.
 * POST /api/rpc with method subscribe; requires HttpClient (with auth/scope).
 */

import { HttpClient, HttpClientRequest } from '@effect/platform'
import { Effect, Layer } from 'effect'
import type { RpcApiService, SubscribeResult } from './RpcApi'
import { RpcApi } from './RpcApi'

function toError(e: unknown): Error {
  return e instanceof Error ? e : new Error(String(e))
}

function parseError(body: unknown): string {
  if (
    typeof body === 'object' &&
    body !== null &&
    'message' in body &&
    typeof (body as { message: unknown }).message === 'string'
  ) {
    return (body as { message: string }).message
  }
  if (
    typeof body === 'object' &&
    body !== null &&
    'error' in body &&
    typeof (body as { error: unknown }).error === 'object' &&
    (body as { error: { message?: string } }).error?.message
  ) {
    return (body as { error: { message: string } }).error.message
  }
  return 'Request failed.'
}

const RpcApiLive = Layer.effect(
  RpcApi,
  Effect.gen(function* () {
    const client = yield* HttpClient.HttpClient

    const subscribe: RpcApiService['subscribe'] = (entityId, params) =>
      Effect.gen(function* () {
        const body = {
          method: 'subscribe',
          entity_id: entityId,
          params: params
            ? {
                filter: params.filter,
                sort: params.sort,
                order: params.order,
                offset: params.offset,
                limit: params.limit,
              }
            : undefined,
        }
        const response = yield* client
          .execute(
            HttpClientRequest.post('/api/rpc').pipe(
              HttpClientRequest.bodyUnsafeJson(body),
            ),
          )
          .pipe(Effect.mapError(toError))
        const resBody = yield* response.json.pipe(Effect.mapError(toError))
        if (response.status >= 200 && response.status < 300) {
          const result = (resBody as { result?: { subscription_id?: string } })
            ?.result
          const id = result?.subscription_id
          if (typeof id === 'string') {
            return { subscription_id: id } satisfies SubscribeResult
          }
        }
        return yield* Effect.fail(new Error(parseError(resBody)))
      })

    return { subscribe }
  }),
)

export { RpcApiLive }
