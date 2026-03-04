import { Effect, Layer } from "effect";
import type { ForgeWebsocketKey, ForgeWebsocketService } from "./ForgeWebsocket";
import { ForgeWebsocket } from "./ForgeWebsocket";

/**
 * Mock ForgeWebsocket for tests: no real connection, no keyed dispatch.
 * getConnected returns false; subscribe returns a no-op unsubscribe; send succeeds.
 */
function makeForgeWebsocketMock(): ForgeWebsocketService {
	const listeners = new Map<ForgeWebsocketKey, Set<(data: unknown) => void>>();

	return {
		getConnected: () => Effect.succeed(false),

		subscribe: (key, listener) =>
			Effect.sync(() => {
				let set = listeners.get(key);
				if (!set) {
					set = new Set();
					listeners.set(key, set);
				}
				set.add(listener);
				return () => {
					listeners.get(key)?.delete(listener);
				};
			}),

		send: () => Effect.void,
	};
}

/**
 * Layer that provides a mock ForgeWebsocket for tests.
 * No dependencies; no real connection or message routing.
 */
export const ForgeWebsocketMock = Layer.succeed(
	ForgeWebsocket,
	makeForgeWebsocketMock(),
);
