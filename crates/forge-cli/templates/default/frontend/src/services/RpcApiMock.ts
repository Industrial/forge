import { Effect, Layer } from 'effect'
import type { RpcApiService, SubscribeResult } from './RpcApi'
import { RpcApi } from './RpcApi'
import type { ListQueryParams } from './EntityApi'

/**
 * Mock RpcApi for tests: subscribe returns a fake subscription_id, no real HTTP.
 */
function makeRpcApiMock(): RpcApiService {
  let subscriptionCounter = 0
  return {
    subscribe: (_entityId: string, _params?: ListQueryParams) =>
      Effect.succeed({
        subscription_id: `mock-sub-${++subscriptionCounter}`,
      } satisfies SubscribeResult),
  }
}

/**
 * Layer that provides a mock RpcApi for tests.
 * No dependencies; no real RPC calls.
 */
export const RpcApiMock = Layer.succeed(RpcApi, makeRpcApiMock())
