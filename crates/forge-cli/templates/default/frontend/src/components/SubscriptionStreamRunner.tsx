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
import { useAuthStore } from '@/features/authentication/stores'
import { Option } from 'effect'
import { getApplicationLayer } from '@/lib/appLayer'
import { trigger } from '@/lib/subscriptionRegistry'
import { SubscriptionStreamStatusStoreTag } from '@/lib/subscriptionStreamStatusStore'
import { SubscriptionStream } from '@/services/SubscriptionStream'

export function SubscriptionStreamRunner() {
  const authentication = useAuthStore()
  const hasToken = Option.isSome(authentication.token)

  useEffect(() => {
    if (!hasToken) return

    const layer = getApplicationLayer()

    const program = Effect.gen(function* () {
      yield* Effect.logTrace('SubscriptionStreamRunner: starting')
      const svc = yield* SubscriptionStream
      const statusStore = yield* SubscriptionStreamStatusStoreTag
      yield* Effect.logDebug('SubscriptionStreamRunner: opening stream')
      const stream = yield* svc.openStream()
      yield* Effect.logDebug(
        'SubscriptionStreamRunner: stream opened, forking runForEach',
      )

      yield* Effect.fork(
        Stream.runForEach(stream, (e) =>
          Effect.gen(function* () {
            if ('type' in e && e.type === 'ready') {
              yield* Effect.logDebug(
                'SubscriptionStreamRunner: received ready, set connected=true',
              )
              yield* statusStore.update(() => ({ connected: true }))
            }
            if ('subscription_id' in e) {
              yield* Effect.logDebug(
                `SubscriptionStreamRunner: received subscription_id=${e.subscription_id}, triggering`,
              )
              trigger(e.subscription_id)
            }
          }),
        ).pipe(
          Effect.ensuring(
            Effect.gen(function* () {
              yield* Effect.logDebug(
                'SubscriptionStreamRunner: stream ended, set connected=false',
              )
              yield* statusStore.update(() => ({ connected: false }))
            }),
          ),
        ),
      )
      yield* Effect.logTrace('SubscriptionStreamRunner: forked, exiting')
    })

    Effect.runFork(program.pipe(Effect.provide(layer)))
  }, [hasToken])

  return null
}
