import { Effect, Layer, Ref, Runtime } from 'effect'
import type { ConnectionStatus, WebsocketService } from './Websocket'
import { Websocket, WebsocketError } from './Websocket'

/**
 * Returns the WebSocket URL for the app's /ws endpoint.
 * In dev (Vite), uses VITE_BACKEND_URL so the socket connects directly to the backend.
 * In production, uses the same origin as the page.
 */
export function getWsUrl(): string {
  if (typeof window === 'undefined') return 'ws://localhost/ws'
  const backendUrl = import.meta.env.VITE_BACKEND_URL as string | undefined
  if (import.meta.env.DEV && backendUrl) {
    try {
      const url = new URL(backendUrl)
      url.protocol = url.protocol === 'https:' ? 'wss:' : 'ws:'
      url.pathname = '/ws'
      return url.toString()
    } catch {
      // fall through to same-origin
    }
  }
  const protocol = window.location.protocol === 'https:' ? 'wss:' : 'ws:'
  return `${protocol}//${window.location.host}/ws`
}

const WS_DEBUG_KEY = 'forge_ws_debug'

function isWsDebugEnabled(): boolean {
  if (typeof window === 'undefined') return false
  return (
    import.meta.env.DEV || window.localStorage.getItem(WS_DEBUG_KEY) === '1'
  )
}

function attachWsDebugLogging(ws: WebSocket): void {
  if (!isWsDebugEnabled()) return
  const originalSend = ws.send.bind(ws)
  ws.send = function (data: string | ArrayBufferLike | Blob) {
    try {
      const payload = typeof data === 'string' ? JSON.parse(data) : data
      console.log('[Forge WS] →', payload)
    } catch {
      console.log('[Forge WS] →', data)
    }
    originalSend(data)
  }
  ws.addEventListener('message', (event) => {
    try {
      const payload =
        typeof event.data === 'string' ? JSON.parse(event.data) : event.data
      console.log('[Forge WS] ←', payload)
    } catch {
      console.log('[Forge WS] ←', event.data)
    }
  })
}

/**
 * Live WebSocket service: manages a single browser WebSocket.
 * Uses Ref for status and listeners; connect/disconnect/send/subscribe.
 * No external dependencies.
 */
export const WebsocketLive = Layer.effect(
  Websocket,
  Effect.gen(function* () {
    const runtime = yield* Effect.runtime<never>()
    const statusRef = yield* Ref.make<ConnectionStatus>('closed')
    const wsRef = yield* Ref.make<WebSocket | null>(null)
    const listenersRef = yield* Ref.make<Set<(data: unknown) => void>>(
      new Set(),
    )

    const run = <A>(effect: Effect.Effect<A>) =>
      Runtime.runPromise(runtime)(effect)

    const service: WebsocketService = {
      getStatus: () => Ref.get(statusRef),

      connect: (url) =>
        Effect.gen(function* () {
          const current = yield* Ref.get(wsRef)
          if (current?.readyState === WebSocket.OPEN) return

          yield* Ref.set(statusRef, 'connecting')
          const ws = new WebSocket(url)
          attachWsDebugLogging(ws)
          yield* Ref.set(wsRef, ws)

          yield* Effect.async<void, WebsocketError>((resume) => {
            ws.onopen = () => {
              run(Ref.set(statusRef, 'open')).then(() =>
                resume(Effect.succeed(undefined)),
              )
            }
            ws.onerror = (e) => {
              run(Ref.set(statusRef, 'closed')).then(() =>
                resume(
                  Effect.fail(
                    new WebsocketError({
                      message: 'WebSocket error',
                      cause: e,
                    }),
                  ),
                ),
              )
            }
            ws.onclose = () => {
              run(Ref.set(statusRef, 'closed'))
              run(Ref.set(wsRef, null))
            }
            ws.onmessage = (event) => {
              try {
                const data =
                  typeof event.data === 'string'
                    ? JSON.parse(event.data)
                    : event.data
                run(Ref.get(listenersRef)).then(
                  (set: Set<(data: unknown) => void>) =>
                    set.forEach((l: (data: unknown) => void) => l(data)),
                )
              } catch {
                // ignore parse errors
              }
            }
          })
        }),

      disconnect: () =>
        Effect.gen(function* () {
          const ws = yield* Ref.get(wsRef)
          if (ws) {
            yield* Ref.set(statusRef, 'closing')
            ws.close()
            yield* Ref.set(wsRef, null)
            yield* Ref.set(statusRef, 'closed')
          }
        }),

      send: (message) =>
        Effect.gen(function* () {
          const ws = yield* Ref.get(wsRef)
          if (ws?.readyState !== WebSocket.OPEN) {
            return yield* Effect.fail(
              new WebsocketError({
                message: 'WebSocket is not open',
              }),
            )
          }
          return yield* Effect.tryPromise({
            try: () =>
              new Promise<void>((resolve, reject) => {
                try {
                  ws.send(message)
                  resolve()
                } catch (e) {
                  reject(e)
                }
              }),
            catch: (e) =>
              new WebsocketError({
                message: e instanceof Error ? e.message : 'Send failed',
                cause: e,
              }),
          })
        }),

      subscribe: (listener) =>
        Effect.gen(function* () {
          yield* Ref.update(listenersRef, (set) => {
            set.add(listener)
            return set
          })
          return () => {
            run(
              Ref.update(listenersRef, (set) => {
                set.delete(listener)
                return set
              }),
            )
          }
        }),
    }

    return service
  }),
)
