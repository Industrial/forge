/**
 * Subscribe to an entity list via RPC and register for invalidation refetch.
 * When the subscription stream emits this subscription_id, onRefetch is called
 * (caller should refetch with same params and replace state).
 *
 * @see subscriptionRegistry, SubscriptionStream, RpcApi (Epic 8)
 */

import { useEffect, useRef } from 'react'
import { useAuthentication } from '../context/AuthenticationContext'
import { register, unregister } from '../lib/subscriptionRegistry'
import type { ListQueryParams } from '../services/EntityApi'
import { runApp } from '../lib/appRuntime'
import { Effect } from 'effect'
import { RpcApi } from '../services/RpcApi'

export function useEntitySubscription(
  entityId: string,
  params: ListQueryParams | undefined,
  onRefetch: () => void,
): void {
  const { token } = useAuthentication()
  const subscriptionIdRef = useRef<string | null>(null)
  const onRefetchRef = useRef(onRefetch)
  onRefetchRef.current = onRefetch

  useEffect(() => {
    if (!token) return

    const subscribeEffect = Effect.gen(function* () {
      const rpc = yield* RpcApi
      return yield* rpc.subscribe(entityId, params)
    })

    runApp(subscribeEffect).then((result) => {
      subscriptionIdRef.current = result.subscription_id
      register(result.subscription_id, {
        entityId,
        params,
        onInvalidate: () => onRefetchRef.current(),
      })
    })

    return () => {
      const id = subscriptionIdRef.current
      if (id) {
        unregister(id)
        subscriptionIdRef.current = null
      }
    }
  }, [entityId, token, JSON.stringify(params ?? {})])
}
