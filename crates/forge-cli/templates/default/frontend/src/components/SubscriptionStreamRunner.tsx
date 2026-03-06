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
import { runApp } from '../lib/appRuntime'
import { SubscriptionStream } from '../services/SubscriptionStream'

export function SubscriptionStreamRunner() {
  const { token } = useAuthentication()

  useEffect(() => {
    if (!token) return

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

    runApp(program)
  }, [token])

  return null
}
