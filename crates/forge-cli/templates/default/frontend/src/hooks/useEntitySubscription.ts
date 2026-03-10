/**
 * Subscribe to an entity list via RPC and register for invalidation refetch.
 * When the subscription stream emits this subscription_id, onRefetch is called
 * (caller should refetch with same params and replace state).
 *
 * @see subscriptionRegistry, SubscriptionStream, RpcApi (Epic 8)
 */
import { useEffect, useRef } from 'react'
import { Effect, Option } from 'effect'
import { useAuthStore } from '@/features/authentication/stores'
import { register, unregister } from '@/lib/subscriptionRegistry'
import {
  SubscriptionStreamStatusStoreTag,
  initialSubscriptionStreamStatus,
} from '@/lib/subscriptionStreamStatusStore'
import { useReactiveStore } from '@/lib/ReactiveStore'
import { useRunWithAppLayer } from '@/lib/appLayer'
import { RpcApi } from '@/services/RpcApi'
import type { ListQueryParams } from '@/services/EntityApi'

export function useEntitySubscription(
  entityId: string,
  params: ListQueryParams | undefined,
  onRefetch: () => void,
): void {
  const authentication = useAuthStore()
  const hasToken = Option.isSome(authentication.token)
  const { run, runFork } = useRunWithAppLayer()
  const status = useReactiveStore(
    SubscriptionStreamStatusStoreTag,
    initialSubscriptionStreamStatus,
    run,
    runFork,
  )
  const connectionId = status.connectionId
  const subscriptionIdRef = useRef<string | null>(null)
  const onRefetchRef = useRef(onRefetch)
  onRefetchRef.current = onRefetch

  useEffect(() => {
    if (!hasToken || !connectionId) {
      return
    }

    const subscribeParams = {
      ...(params ?? {}),
      connection_id: connectionId,
    }
    const subscribeEffect = Effect.gen(function* () {
      const rpc = yield* RpcApi
      return yield* rpc.subscribe(entityId, subscribeParams)
    })

    run(subscribeEffect)
      .then((result) => {
        subscriptionIdRef.current = result.subscription_id
        register(result.subscription_id, {
          entityId,
          params: params ?? undefined,
          onInvalidate: () => onRefetchRef.current(),
        })
      })
      .catch(() => {})

    return () => {
      const id = subscriptionIdRef.current
      if (id) {
        unregister(id)
        subscriptionIdRef.current = null
      }
    }
  }, [entityId, hasToken, connectionId, run, params])
}
