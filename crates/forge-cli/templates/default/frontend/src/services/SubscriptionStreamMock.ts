import { Effect, Layer, Stream } from 'effect'
import type {
  SubscriptionStreamEvent,
  SubscriptionStreamService,
} from './SubscriptionStream'
import { SubscriptionStream } from './SubscriptionStream'

/**
 * Mock SubscriptionStream for tests: openStream returns a stream that emits
 * a single "ready" event then ends. No real SSE connection.
 */
function makeSubscriptionStreamMock(): SubscriptionStreamService {
  return {
    openStream: () =>
      Effect.succeed(
        Stream.fromIterable([{ type: 'ready' } as const] as SubscriptionStreamEvent[]),
      ),
  }
}

/**
 * Layer that provides a mock SubscriptionStream for tests.
 * No dependencies; no real fetch or SSE.
 */
export const SubscriptionStreamMock = Layer.succeed(
  SubscriptionStream,
  makeSubscriptionStreamMock(),
)
