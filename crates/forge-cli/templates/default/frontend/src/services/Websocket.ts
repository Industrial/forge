/**
 * WebSocket service and connection types.
 *
 * Effect service for managing a single WebSocket connection: connect, disconnect,
 * send messages, and subscribe to incoming messages. All methods use
 * `Effect<A, WebsocketError, never>` (or never fail); the Live implementation
 * owns the browser WebSocket and notifies subscribers on each message.
 *
 * @see WebsocketLive – browser implementation with Ref + WebSocket API
 * @see WebsocketMock – test double
 */

import { Context, Data, Effect } from "effect";

/**
 * Current WebSocket connection state.
 *
 * - **connecting**: Socket created, waiting for `open`
 * - **open**: Connected; can send and receive
 * - **closing**: Close requested, waiting for `close`
 * - **closed**: Not connected
 */
export type ConnectionStatus = "connecting" | "open" | "closing" | "closed";

/**
 * Error produced by WebSocket operations.
 *
 * @remarks
 * Tagged Data error for pattern matching. Used when connect fails (e.g. network),
 * send fails (e.g. not open), or the socket errors.
 */
export class WebsocketError extends Data.TaggedError("WebsocketError")<{
	readonly message: string;
	readonly cause?: unknown;
}> {}

/**
 * WebSocket service interface.
 *
 * Manages one WebSocket connection: lifecycle (connect/disconnect), sending
 * messages, and subscribing to incoming messages. Methods return
 * `Effect<A, WebsocketError, never>` (or never fail for getStatus/subscribe)
 * so callers have no requirements; the Live layer owns the socket.
 *
 * @remarks
 * - **getStatus**: Current connection state; never fails.
 * - **connect(url)**: Opens the connection; idempotent if already open.
 * - **disconnect**: Closes the connection and clears subscribers.
 * - **send(message)**: Sends a string (typically JSON); fails if not open.
 * - **subscribe(listener)**: Registers a callback for each incoming message (parsed as JSON in Live); returns an Effect that yields an unsubscribe function.
 */
export interface WebsocketService {
	/** Returns the current connection status. Never fails. */
	readonly getStatus: () => Effect.Effect<ConnectionStatus, never, never>;
	/** Opens the connection to `url` (e.g. `ws://host/ws`). No-op if already open. */
	readonly connect: (url: string) => Effect.Effect<void, WebsocketError, never>;
	/** Closes the connection. */
	readonly disconnect: () => Effect.Effect<void, WebsocketError, never>;
	/** Sends a string message (typically JSON). Fails if the socket is not open. */
	readonly send: (message: string) => Effect.Effect<void, WebsocketError, never>;
	/**
	 * Subscribes to incoming messages. The listener is called with the parsed
	 * payload (e.g. JSON in Live). Returns an Effect that yields a function
	 * to remove the listener.
	 */
	readonly subscribe: (
		listener: (data: unknown) => void,
	) => Effect.Effect<() => void, never, never>;
}

/**
 * Tag for the WebSocket service.
 *
 * Use this to access the service in Effects (e.g. `yield* Websocket` or
 * `Effect.flatMap(Websocket, ws => ws.getStatus())`). Provide the service
 * with `WebsocketLive` or `WebsocketMock` in your Layer composition.
 *
 * @example
 * ```ts
 * const program = Effect.gen(function* () {
 *   const ws = yield* Websocket;
 *   yield* ws.connect("wss://example.com/ws");
 *   const status = yield* ws.getStatus();
 *   return status;
 * });
 * ```
 */
export const Websocket = Context.GenericTag<WebsocketService>("@forge/Websocket");
