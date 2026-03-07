/**
 * Live implementation of SubscriptionStream using fetch and SSE parsing.
 *
 * GET /api/subscriptions/stream with Authorization header; parses text/event-stream
 * for data lines (ready and subscription_id). Uses auth reactive store for token.
 */

import { Effect, Option, Stream } from 'effect'
import { Layer } from 'effect'
import { AuthenticationStateReactiveStoreTag } from '@/features/authentication/stores/AuthenticationStateReactiveStore'
import type {
  SubscriptionStreamEvent,
  SubscriptionStreamService,
} from './SubscriptionStream'
import { SubscriptionStream } from './SubscriptionStream'

function toError(e: unknown): Error {
  return e instanceof Error ? e : new Error(String(e))
}

async function* readSSEEvents(
  reader: ReadableStreamDefaultReader<Uint8Array>,
): AsyncGenerator<SubscriptionStreamEvent, void, unknown> {
  const decoder = new TextDecoder()
  let buffer = ''
  try {
    while (true) {
      const { done, value } = await reader.read()
      if (done) break
      buffer += decoder.decode(value, { stream: true })
      const events = buffer.split('\n\n')
      buffer = events.pop() ?? ''
      for (const block of events) {
        const line = block.split('\n').find((l) => l.startsWith('data:'))
        if (!line) continue
        const json = line.slice(5).trim()
        if (json === '[DONE]' || !json) continue
        try {
          const obj = JSON.parse(json) as Record<string, unknown>
          if (obj?.type === 'ready') {
            yield { type: 'ready' }
            continue
          }
          if (typeof obj?.subscription_id === 'string') {
            yield { subscription_id: obj.subscription_id }
          }
        } catch {
          // skip malformed
        }
      }
    }
  } finally {
    reader.releaseLock()
  }
}

export const SubscriptionStreamLive = (baseUrl: string) =>
  Layer.effect(
    SubscriptionStream,
    Effect.gen(function* () {
      const authStore = yield* AuthenticationStateReactiveStoreTag

      const openStream: SubscriptionStreamService['openStream'] = () =>
        Effect.gen(function* () {
          yield* Effect.logTrace('SubscriptionStreamLive.openStream')
          const state = yield* authStore.get()
          const token = Option.getOrElse(state.token, () => null)
          if (!token) {
            yield* Effect.logDebug(
              'SubscriptionStreamLive.openStream: no token',
            )
            return yield* Effect.fail(
              new Error('Not authenticated; cannot open subscription stream'),
            )
          }
          yield* Effect.logDebug(
            `SubscriptionStreamLive.openStream: url=${baseUrl.replace(/\/$/, '')}/api/subscriptions/stream`,
          )
          const url = `${baseUrl.replace(/\/$/, '')}/api/subscriptions/stream`
          const res = yield* Effect.tryPromise({
            try: () =>
              fetch(url, {
                method: 'GET',
                headers: { Authorization: `Bearer ${token}` },
              }),
            catch: toError,
          })
          if (!res.ok) {
            return yield* Effect.fail(
              new Error(`Subscription stream failed: ${res.status}`),
            )
          }
          const body = res.body
          if (!body) {
            return yield* Effect.fail(new Error('Subscription stream: no body'))
          }
          const reader = body.getReader()
          const stream = Stream.fromAsyncIterable(
            readSSEEvents(reader),
            (e) => new Error(String(e)),
          )
          yield* Effect.logDebug(
            'SubscriptionStreamLive.openStream: stream opened',
          )
          return stream
        })

      return { openStream }
    }),
  )
