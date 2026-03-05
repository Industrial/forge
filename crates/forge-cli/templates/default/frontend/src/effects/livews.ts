import { Context, Effect, Layer } from 'effect'

/** Keys used to dispatch live updates (maps message type/resource to listener key). */
export type LiveUpdateKey =
  | 'audit-log'
  | 'users'
  | 'roles'
  | 'role_permissions'
  | 'organizations'

/**
 * Represents a live WebSocket connection for real-time updates
 */
export interface LiveWsConnection {
  /** Whether the WebSocket is connected */
  readonly connected: boolean
  /** Subscribe to live updates for a key. Returns unsubscribe function */
  readonly subscribe: (
    key: LiveUpdateKey,
    listener: (data: unknown) => void,
  ) => () => void
  /** Send a message through the WebSocket */
  readonly send: (message: string) => Effect.Effect<void, Error>
}

/**
 * Tag for the LiveWs service.
 * Use this to access the live WebSocket connection in Effects.
 */
export const LiveWs = Context.GenericTag<LiveWsConnection>('@forge/LiveWs')

/**
 * Effect that gets the current WebSocket connection state
 */
export const getConnection = Effect.flatMap(LiveWs, (liveWs) =>
  Effect.succeed(liveWs),
)

/**
 * Effect that subscribes to live updates for a given key
 * @param key The update key to subscribe to
 * @param listener Callback function to handle updates
 * @returns Effect that resolves to the unsubscribe function
 */
export const subscribe = (
  key: LiveUpdateKey,
  listener: (data: unknown) => void,
) =>
  Effect.flatMap(LiveWs, (liveWs) =>
    Effect.succeed(liveWs.subscribe(key, listener)),
  )

/**
 * Effect that sends a message through the WebSocket
 * @param message The message to send
 * @returns Effect that completes when message is sent
 */
export const send = (message: string) =>
  Effect.flatMap(LiveWs, (liveWs) => liveWs.send(message))

/**
 * Effect that checks if the WebSocket is connected
 * @returns Effect that resolves to connection state
 */
export const isConnected = Effect.flatMap(LiveWs, (liveWs) =>
  Effect.succeed(liveWs.connected),
)

/**
 * Creates an implementation of the LiveWs service.
 * @param connection The WebSocket connection implementation
 * @returns LiveWs service implementation
 */
export const makeLiveWs = (connection: {
  connected: boolean
  subscribe: (
    key: LiveUpdateKey,
    listener: (data: unknown) => void,
  ) => () => void
  send?: (message: string) => Promise<void>
}): LiveWsConnection => ({
  connected: connection.connected,
  subscribe: connection.subscribe,
  send: (message: string) =>
    Effect.tryPromise({
      try: () => connection.send?.(message) ?? Promise.resolve(),
      catch: (error) =>
        error instanceof Error
          ? error
          : new Error(`Failed to send WebSocket message: ${String(error)}`),
    }),
})

/**
 * Creates a Layer that provides the LiveWs service.
 * @param connection The WebSocket connection configuration
 * @returns Layer that provides LiveWs
 */
export const LiveWsLive = (connection: {
  connected: boolean
  subscribe: (
    key: LiveUpdateKey,
    listener: (data: unknown) => void,
  ) => () => void
  send?: (message: string) => Promise<void>
}) => Layer.succeed(LiveWs, makeLiveWs(connection))

/**
 * Default (no-op) implementation for testing or when WebSocket is not available
 */
export const LiveWsTest = Layer.succeed(
  LiveWs,
  makeLiveWs({
    connected: false,
    subscribe: () => () => {},
    send: () =>
      Promise.reject(new Error('WebSocket not available in test context')),
  }),
)
