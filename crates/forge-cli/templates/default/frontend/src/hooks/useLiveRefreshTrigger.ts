/**
 * Subscribes to live updates for a channel via RPC + SSE and exposes a trigger value
 * that increments on each invalidation. Use `trigger` in a dependency array to re-run
 * effects (e.g. refetch) when live data arrives. `connected` reflects the subscription
 * stream (SSE) connection status.
 */
import { useEffect, useRef, useState } from 'react'
import { Effect } from 'effect'
import { channelToEntityId, type ForgeWebsocketKey } from '@/lib/liveRefreshChannels'
import { register, unregister } from '@/lib/subscriptionRegistry'
import {
  SubscriptionStreamStatusStoreTag,
  initialSubscriptionStreamStatus,
} from '@/lib/subscriptionStreamStatusStore'
import { useReactiveStore } from '@/lib/ReactiveStore'
import { useRunWithAppLayer } from '@/lib/appLayer'
import { RpcApi } from '@/services/RpcApi'

export function useLiveRefreshTrigger(channel: ForgeWebsocketKey): {
  trigger: number
  connected: boolean
} {
  const [trigger, setTrigger] = useState(0)
  const subscriptionIdRef = useRef<string | null>(null)
  const { run, runFork } = useRunWithAppLayer()
  const entityId = channelToEntityId(channel)

  const status = useReactiveStore(
    SubscriptionStreamStatusStoreTag,
    initialSubscriptionStreamStatus,
    run,
    runFork,
  )
  const connected = status.connected

  useEffect(() => {
    const subscribeEffect = Effect.gen(function* () {
      const rpc = yield* RpcApi
      return yield* rpc.subscribe(entityId, undefined)
    })

    run(subscribeEffect).then((result) => {
      subscriptionIdRef.current = result.subscription_id
      register(result.subscription_id, {
        entityId,
        params: undefined,
        onInvalidate: () => setTrigger((n) => n + 1),
      })
    }).catch(() => {})

    return () => {
      const id = subscriptionIdRef.current
      if (id) {
        unregister(id)
        subscriptionIdRef.current = null
      }
    }
  }, [entityId, run])

  return { trigger, connected }
}
