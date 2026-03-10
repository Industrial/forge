/**
 * Subscription invalidation stream service (SSE).
 *
 * Opens GET /api/subscriptions/stream with auth; parses SSE for "ready" and
 * subscription_id invalidation events. Used to refetch when backend publishes
 * an invalidation for a subscription. All methods return Effect<A, Error, never>;
 * Live implementation uses fetch + AuthStateRef for token.
 *
 * @see SubscriptionStreamLive – implementation with fetch and SSE parsing
 * @see docs/frontend-implementation-and-choices.md (Epic 8)
 */

import { Context, type Effect, type Stream } from 'effect'

export type SubscriptionStreamEvent =
  | { type: 'ready'; connection_id: string }
  | { subscription_id: string }

/**
 * Subscription stream service interface.
 *
 * openStream() opens the SSE connection and returns a Stream of events
 * (ready first, then subscription_id invalidations). Caller should close
 * the stream (or abort the fetch) when done.
 */
export interface SubscriptionStreamService {
  /**
   * Opens GET /api/subscriptions/stream with auth; returns a Stream of
   * ready and invalidation events. Stream runs until the connection closes or errors.
   */
  readonly openStream: () => Effect.Effect<
    Stream.Stream<SubscriptionStreamEvent, Error>,
    Error,
    never
  >
}

/**
 * Tag for the SubscriptionStream service.
 *
 * Use in Effects: yield* SubscriptionStream then yield* svc.openStream().
 * Provide with SubscriptionStreamLive(baseUrl) (requires AuthStateRef).
 */
export const SubscriptionStream = Context.GenericTag<SubscriptionStreamService>(
  '@forge/SubscriptionStream',
)
