/**
 * RPC API service for subscribe/unsubscribe (Epic 8).
 * POST /api/rpc with method subscribe (entity_id + params); returns subscription_id.
 * RPC and REST use the same object types (forge-query).
 *
 * @see RpcApiLive – implementation using HttpClient
 */

import { Context, Effect } from 'effect'
import type { ListQueryParams, RpcSubscribeResult } from '@/api/types'

export type { ListQueryParams }
export type SubscribeResult = RpcSubscribeResult

export interface RpcApiService {
  /** POST /api/rpc { method: "subscribe", entity_id, params }; returns subscription_id. */
  readonly subscribe: (
    entityId: string,
    params?: ListQueryParams,
  ) => Effect.Effect<SubscribeResult, Error, never>
}

export const RpcApi = Context.GenericTag<RpcApiService>('@forge/RpcApi')
