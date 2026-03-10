/**
 * Live implementation of RpcApi using HttpClient.
 * POST /api/rpc with method subscribe; requires HttpClient (with auth/scope).
 * RPC uses same object types as REST (forge-query); response decoded via Schema.
 */

import { HttpClientRequest } from '@effect/platform'
import { AuthenticatedHttpClient } from '@/services/AuthenticatedHttpClient'
import { Effect, Layer, pipe } from 'effect'
import { Schema } from 'effect'
import {
  ApiErrorBodySchema,
  RpcResponseSchema,
  RpcSubscribeResultSchema,
  type RpcSubscribeRequest,
} from '@/api/types'
import type { RpcApiService } from './RpcApi'
import { RpcApi } from './RpcApi'

function toError(e: unknown): Error {
  return e instanceof Error ? e : new Error(String(e))
}

const RpcSubscribeResponseSchema = RpcResponseSchema(RpcSubscribeResultSchema)

const RpcApiLive = Layer.effect(
  RpcApi,
  Effect.gen(function* () {
    const client = yield* AuthenticatedHttpClient

    const subscribe: RpcApiService['subscribe'] = (entityId, params) =>
      Effect.gen(function* () {
        yield* Effect.logTrace('RpcApiLive.subscribe')
        yield* Effect.logDebug(
          `RpcApiLive.subscribe: entityId=${entityId}, params=${JSON.stringify(params ?? {})}`,
        )
        const body: RpcSubscribeRequest = {
          method: 'subscribe',
          entity_id: entityId,
          params: params
            ? {
                filter: params.filter,
                sort: params.sort,
                order: params.order,
                offset: params.offset,
                limit: params.limit,
                connection_id: params.connection_id,
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
        const ok = response.status >= 200 && response.status < 300
        if (!ok) {
          const resBody = yield* response.json.pipe(Effect.mapError(toError))
          const errDecoded = yield* pipe(
            Schema.decodeUnknown(ApiErrorBodySchema)(resBody),
            Effect.catchAll(() =>
              Effect.succeed({ message: undefined, error: undefined }),
            ),
          )
          const message =
            errDecoded.message ??
            (typeof errDecoded.error === 'string'
              ? errDecoded.error
              : errDecoded.error?.message) ??
            `RPC failed (${response.status})`
          return yield* Effect.fail(new Error(message))
        }
        const resBody = yield* response.json.pipe(Effect.mapError(toError))
        const decoded = yield* pipe(
          Schema.decodeUnknown(RpcSubscribeResponseSchema)(resBody),
          Effect.mapError(
            (e) => new Error(`Invalid RPC response: ${e.message ?? String(e)}`),
          ),
        )
        const result = decoded.result
        if (result) {
          yield* Effect.logDebug(
            `RpcApiLive.subscribe: subscription_id=${result.subscription_id}`,
          )
          return { subscription_id: result.subscription_id }
        }
        const message =
          decoded.error?.message ?? 'RPC returned no result and no error'
        return yield* Effect.fail(new Error(message))
      })

    return { subscribe }
  }),
)

export { RpcApiLive }
