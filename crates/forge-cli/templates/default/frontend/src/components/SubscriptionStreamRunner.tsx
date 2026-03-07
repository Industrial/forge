/**
 * Opens the subscription invalidation stream and triggers refetch for
 * any subscription_id received. Mount when authenticated so invalidations
 * are dispatched to the subscription registry. Sets stream connection
 * status (ready → connected, stream end → disconnected).
 *
 * @see subscriptionRegistry, useEntitySubscription, useLiveRefreshTrigger (Epic 8)
 */
import { useEffect } from 'react'
import { Effect, Stream } from 'effect'
import { useAuthenticationStateReactiveStore } from '@/features/authentication/hooks/useAuthenticationStateReactiveStore'
import { Option } from 'effect'
import { getApplicationLayer } from '@/lib/appLayer'
import { trigger } from '@/lib/subscriptionRegistry'
import { SubscriptionStreamStatusStoreTag } from '@/lib/subscriptionStreamStatusStore'
import { SubscriptionStream } from '@/services/SubscriptionStream'

export function SubscriptionStreamRunner() {
  const { authentication } = useAuthenticationStateReactiveStore()
  const hasToken = Option.isSome(authentication.token)

  useEffect(() => {
    if (!hasToken) return

    const layer = getApplicationLayer()

    const program = Effect.gen(function* () {
      const svc = yield* SubscriptionStream
      const statusStore = yield* SubscriptionStreamStatusStoreTag
      const stream = yield* svc.openStream()

      yield* Effect.fork(
        Stream.runForEach(stream, (e) =>
          Effect.gen(function* () {
            if ('type' in e && e.type === 'ready') {
              yield* statusStore.update(() => ({ connected: true }))
            }
            if ('subscription_id' in e) {
              trigger(e.subscription_id)
            }
          }),
        ).pipe(
          Effect.ensuring(statusStore.update(() => ({ connected: false }))),
        ),
      )
    })

    Effect.runFork(program.pipe(Effect.provide(layer)))
  }, [hasToken])

  return null
}
