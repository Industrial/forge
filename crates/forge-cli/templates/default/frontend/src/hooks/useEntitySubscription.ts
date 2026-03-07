/**
 * Subscribe to an entity list via RPC and register for invalidation refetch.
 * When the subscription stream emits this subscription_id, onRefetch is called
 * (caller should refetch with same params and replace state).
 *
 * @see subscriptionRegistry, SubscriptionStream, RpcApi (Epic 8)
 */
import { useEffect, useRef } from 'react'
import { Effect, Option } from 'effect'
import { useAuthenticationStateReactiveStore } from '@/features/authentication/stores'
import { register, unregister } from '@/lib/subscriptionRegistry'
import { useRunWithAppLayer } from '@/lib/appLayer'
import { RpcApi } from '@/services/RpcApi'
import type { ListQueryParams } from '@/services/EntityApi'

export function useEntitySubscription(
  entityId: string,
  params: ListQueryParams | undefined,
  onRefetch: () => void,
): void {
  const { authentication } = useAuthenticationStateReactiveStore()
  const hasToken = Option.isSome(authentication.token)
  const subscriptionIdRef = useRef<string | null>(null)
  const onRefetchRef = useRef(onRefetch)
  onRefetchRef.current = onRefetch
  const { run } = useRunWithAppLayer()

  useEffect(() => {
    if (!hasToken) return

    const subscribeEffect = Effect.gen(function* () {
      const rpc = yield* RpcApi
      return yield* rpc.subscribe(entityId, params)
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
  }, [entityId, hasToken, run, JSON.stringify(params ?? {})])
}
