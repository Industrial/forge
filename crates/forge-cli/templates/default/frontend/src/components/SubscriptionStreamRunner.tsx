/**
 * Opens the subscription invalidation stream and triggers refetch for
 * any subscription_id received. Mount when authenticated so invalidations
 * are dispatched to the subscription registry.
 *
 * @see subscriptionRegistry, useEntitySubscription (Epic 8)
 */

import { useEffect } from 'react'
import { Effect, Stream } from 'effect'
import { useAuthentication } from '../context/AuthenticationContext'
import { trigger } from '../lib/subscriptionRegistry'
import { runWithAppRuntime, type AppServices } from '../lib/appLayer'
import { SubscriptionStream } from '../services/SubscriptionStream'
import { useEffectRuntime } from 'react-effect-hooks'

export function SubscriptionStreamRunner() {
  const { token } = useAuthentication()
  const { runtime } = useEffectRuntime<AppServices>()

  useEffect(() => {
    if (!token || !runtime) return

    const program = Effect.gen(function* () {
      const svc = yield* SubscriptionStream
      const stream = yield* svc.openStream()
      yield* Effect.fork(
        Stream.runForEach(stream, (e) =>
          Effect.sync(() => {
            if ('subscription_id' in e) trigger(e.subscription_id)
          }),
        ),
      )
    })

    runWithAppRuntime(runtime, program).catch(() => {})
  }, [token, runtime])

  return null
}
