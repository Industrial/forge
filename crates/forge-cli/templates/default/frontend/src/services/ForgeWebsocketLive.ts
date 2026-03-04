import { Effect, Layer, Ref, Runtime } from "effect";
import type { ForgeWebsocketKey, ForgeWebsocketService } from "./ForgeWebsocket";
import {
	ForgeWebsocket,
	ForgeWebsocketError,
} from "./ForgeWebsocket";
import { Websocket } from "./Websocket";

/** Maps backend message type/resource to ForgeWebsocketKey(s). */
function messageKeys(data: unknown): ForgeWebsocketKey[] {
	if (!data || typeof data !== "object" || !("type" in data)) return [];
	const t = (data as { type: string }).type;
	if (t === "audit_log") return ["audit-log"];
	if (t === "users_updated") return ["users"];
	if (t === "resource_changed") {
		const r = (data as { resource?: string }).resource;
		if (r === "organizations") return ["organizations"];
		if (r === "roles") return ["roles"];
		if (r === "role_permissions") return ["role_permissions"];
		if (r === "users") return ["users"];
	}
	return [];
}

type Listener = (data: unknown) => void;

/**
 * Live ForgeWebsocket: keyed live updates on top of the Websocket service.
 * Subscribes to Websocket once and dispatches incoming messages to listeners by key.
 * Layer requires Websocket.
 */
export const ForgeWebsocketLive = Layer.effect(
	ForgeWebsocket,
	Effect.gen(function* () {
		const ws = yield* Websocket;
		const runtime = yield* Effect.runtime<never>();
		const keyedListenersRef = yield* Ref.make<
			Map<ForgeWebsocketKey, Set<Listener>>
		>(new Map());

		const run = <A>(effect: Effect.Effect<A>) =>
			Runtime.runPromise(runtime)(effect);

		// Single subscription to underlying Websocket; dispatch by key to keyed listeners
		yield* ws.subscribe((data: unknown) => {
			const keys = messageKeys(data);
			run(Ref.get(keyedListenersRef)).then((map) => {
				for (const k of keys) {
					map.get(k)?.forEach((l) => l(data));
				}
			});
		});

		const service: ForgeWebsocketService = {
			getConnected: () =>
				Effect.gen(function* () {
					const status = yield* ws.getStatus();
					return status === "open";
				}),

			subscribe: (key, listener) =>
				Effect.gen(function* () {
					yield* Ref.update(keyedListenersRef, (map) => {
						const set = map.get(key) ?? new Set<Listener>();
						set.add(listener);
						map.set(key, set);
						return map;
					});
					return () => {
						run(
							Ref.update(keyedListenersRef, (map) => {
								const set = map.get(key);
								if (set) {
									set.delete(listener);
									if (set.size === 0) map.delete(key);
									else map.set(key, set);
								}
								return map;
							}),
						);
					};
				}),

			send: (message) =>
				ws.send(message).pipe(
					Effect.mapError(
						(e) =>
							new ForgeWebsocketError({
								message: e.message,
								cause: e,
							}),
					),
				),
		};

		return service;
	}),
);
