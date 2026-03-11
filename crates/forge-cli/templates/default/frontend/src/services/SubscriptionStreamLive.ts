/**
 * Live implementation of SubscriptionStream using HttpClient and SSE parsing.
 *
 * GET /api/subscriptions/stream with Authorization header; parses text/event-stream
 * for data lines (ready and subscription_id). Uses HttpClient (no fetch).
 */

import { HttpClient, HttpClientRequest } from '@effect/platform'
import { Effect, Option, Stream } from 'effect'
import { Layer } from 'effect'
import { AuthenticationStateReactiveStoreTag } from '../features/authentication/stores/AuthenticationStateReactiveStore'
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
      if (done) {
        break
      }
      buffer += decoder.decode(value, { stream: true })
      const events = buffer.split('\n\n')
      buffer = events.pop() ?? ''
      for (const block of events) {
        const line = block.split('\n').find((l) => l.startsWith('data:'))
        if (!line) {
          continue
        }
        const json = line.slice(5).trim()
        if (json === '[DONE]' || !json) {
          continue
        }
        try {
          const obj = JSON.parse(json) as Record<string, unknown>
          if (obj?.type === 'ready' && typeof obj?.connection_id === 'string') {
            yield { type: 'ready', connection_id: obj.connection_id }
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

const SubscriptionStreamLiveFn = (baseUrl: string) =>
  Layer.effect(
    SubscriptionStream,
    Effect.gen(function* () {
      const client = yield* HttpClient.HttpClient
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
          const url = `${baseUrl.replace(/\/$/, '')}/api/subscriptions/stream`
          yield* Effect.logDebug(
            `SubscriptionStreamLive.openStream: url=${url}`,
          )
          const req = HttpClientRequest.get(url).pipe(
            HttpClientRequest.setHeader('Authorization', `Bearer ${token}`),
          )
          const response = yield* client
            .execute(req)
            .pipe(Effect.mapError(toError))
          const res = response as unknown as Record<string, unknown>
          const inner =
            (res.response as Record<string, unknown> | undefined) ?? res
          const status = Number(
            res.status ??
              res.statusCode ??
              inner?.status ??
              inner?.statusCode ??
              0,
          )
          const ok =
            (status >= 200 && status < 300) ||
            res.ok === true ||
            inner?.ok === true
          if (!ok) {
            return yield* Effect.fail(
              new Error(`Subscription stream failed: ${status || 'unknown'}`),
            )
          }
          const body = (res.body ?? inner?.body) as
            | ReadableStream<Uint8Array>
            | undefined
            | null
          if (!body || typeof body.getReader !== 'function') {
            return yield* Effect.fail(new Error('Subscription stream: no body'))
          }
          const reader = body.getReader()
          yield* Effect.logDebug(
            'SubscriptionStreamLive.openStream: got reader, creating stream',
          )
          const eventStream = Stream.fromAsyncIterable(
            readSSEEvents(reader),
            (e) => new Error(String(e)),
          )
          yield* Effect.logDebug(
            'SubscriptionStreamLive.openStream: returning event stream',
          )
          return eventStream
        })

      return { openStream }
    }),
  )

export { SubscriptionStreamLiveFn as SubscriptionStreamLive }
