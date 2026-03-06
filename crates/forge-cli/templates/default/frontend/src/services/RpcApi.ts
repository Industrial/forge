/**
 * RPC API service for subscribe/unsubscribe (Epic 8).
 * POST /api/rpc with method subscribe (entity_id + params); returns subscription_id.
 *
 * @see RpcApiLive – implementation using HttpClient
 */

import { Context, Effect } from 'effect'
import type { ListQueryParams } from './EntityApi'

export interface SubscribeResult {
  readonly subscription_id: string
}

export interface RpcApiService {
  /** POST /api/rpc { method: "subscribe", entity_id, params }; returns subscription_id. */
  readonly subscribe: (
    entityId: string,
    params?: ListQueryParams,
  ) => Effect.Effect<SubscribeResult, Error, never>
}

export const RpcApi = Context.GenericTag<RpcApiService>('@forge/RpcApi')
