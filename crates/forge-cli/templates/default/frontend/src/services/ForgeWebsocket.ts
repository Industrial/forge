import { Context, Data, Effect } from "effect";

/**
 * Keys for keyed live-update subscriptions.
 * Maps backend message types (e.g. users_updated, resource_changed) to subscription keys.
 */
export type ForgeWebsocketKey =
	| "audit-log"
	| "users"
	| "roles"
	| "role_permissions"
	| "organizations";

export class ForgeWebsocketError extends Data.TaggedError("ForgeWebsocketError")<{
	readonly message: string;
	readonly cause?: unknown;
}> {}

/**
 * ForgeWebsocket service: keyed live updates on top of a WebSocket connection.
 *
 * Built on top of the Websocket service. Subscribe by key (e.g. "users", "audit-log");
 * only messages that match that key are delivered to the listener. All methods
 * return `Effect<A, ForgeWebsocketError, never>` (or never fail) so callers have
 * no requirements; the Live layer depends on Websocket and does key dispatch.
 *
 * @remarks
 * - **getConnected**: Whether the underlying connection is open; never fails.
 * - **subscribe(key, listener)**: Registers a listener for messages matching `key`. Returns an Effect that yields an unsubscribe function.
 * - **send(message)**: Sends a string (typically JSON) on the connection; fails if not connected.
 *
 * @see Websocket – transport service this is built on
 * @see ForgeWebsocketLive – implementation that uses Websocket and routes by key
 */

export interface ForgeWebsocketService {
	/** Whether the underlying WebSocket connection is open. */
	readonly getConnected: () => Effect.Effect<boolean, never, never>;
	/**
	 * Subscribes to messages for the given key. The listener is called only when
	 * an incoming message matches that key (e.g. type "users_updated" → key "users").
	 * Returns an Effect that yields a function to remove the listener.
	 */
	readonly subscribe: (
		key: ForgeWebsocketKey,
		listener: (data: unknown) => void,
	) => Effect.Effect<() => void, never, never>;
	/** Sends a string message (typically JSON). Fails if the connection is not open. */
	readonly send: (message: string) => Effect.Effect<void, ForgeWebsocketError, never>;
}

/**
 * Tag for the ForgeWebsocket service.
 *
 * Use this to access keyed live updates in Effects. Provide the service with
 * ForgeWebsocketLive (built on Websocket) in your Layer composition.
 *
 * @example
 * ```ts
 * const program = Effect.gen(function* () {
 *   const forgeWs = yield* ForgeWebsocket;
 *   const unsub = yield* forgeWs.subscribe("users", (data) => console.log(data));
 *   // later: unsub()
 * });
 * ```
 */
export const ForgeWebsocket = Context.GenericTag<ForgeWebsocketService>(
	"@forge/ForgeWebsocket",
);
