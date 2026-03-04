import { Effect, Layer } from "effect";
import type { ConnectionStatus, WebsocketService } from "./Websocket";
import { Websocket } from "./Websocket";

/**
 * Mock WebSocket service for tests: no real connection.
 * getStatus returns "closed"; connect/disconnect/send are no-ops that succeed;
 * subscribe returns a no-op unsubscribe.
 */
function makeWebsocketMock(): WebsocketService {
	let status: ConnectionStatus = "closed";
	const listeners = new Set<(data: unknown) => void>();

	return {
		getStatus: () => Effect.succeed(status),

		connect: (_url) =>
			Effect.sync(() => {
				status = "connecting";
				status = "open";
			}),

		disconnect: () =>
			Effect.sync(() => {
				status = "closed";
				listeners.clear();
			}),

		send: () => Effect.void,

		subscribe: (listener) =>
			Effect.sync(() => {
				listeners.add(listener);
				return () => {
					listeners.delete(listener);
				};
			}),
	};
}

/**
 * Layer that provides a mock WebSocket service for tests.
 * No dependencies; connect/send/subscribe are no-ops.
 */
export const WebsocketMock = Layer.succeed(Websocket, makeWebsocketMock());
