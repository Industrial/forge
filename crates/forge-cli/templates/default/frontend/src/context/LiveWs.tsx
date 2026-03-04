import React, {
	createContext,
	useCallback,
	useContext,
	useEffect,
	useRef,
	useState,
} from "react";
import { attachWsDebugLogging, getWsUrl } from "@/utils/ws";
import { useSession } from "./Session";

/** Keys used to dispatch live updates (maps message type/resource to listener key). */
export type LiveUpdateKey =
	| "audit-log"
	| "users"
	| "roles"
	| "role_permissions"
	| "organizations";

type Listener = (data: unknown) => void;

type LiveWsContextValue = {
	connected: boolean;
	/** Subscribe to live updates for a key. Listener receives the parsed message. Returns unsubscribe. */
	subscribe: (key: LiveUpdateKey, listener: Listener) => () => void;
};

const LiveWsContext = createContext<LiveWsContextValue | null>(null);

function messageKeys(data: unknown): LiveUpdateKey[] {
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

export function useLiveWs(): LiveWsContextValue {
	const ctx = useContext(LiveWsContext);
	if (!ctx) throw new Error("useLiveWs must be used within LiveWsProvider");
	return ctx;
}

/**
 * Subscribe to live updates for a given key. Callback receives the parsed message
 * (e.g. LiveEvent or resource_changed). Use to refetch or update state.
 */
export function useLiveUpdates(
	key: LiveUpdateKey,
	onUpdate: (data: unknown) => void,
): { connected: boolean } {
	const { connected, subscribe } = useLiveWs();
	const onUpdateRef = useRef(onUpdate);
	onUpdateRef.current = onUpdate;
	useEffect(() => {
		const stable = (data: unknown) => onUpdateRef.current?.(data);
		return subscribe(key, stable);
	}, [key, subscribe]);
	return { connected };
}

const CONNECT_DELAY_MS = 600;

export function LiveWsProvider({ children }: { children: React.ReactNode }) {
	const { user } = useSession();
	const [connected, setConnected] = useState(false);
	const wsRef = useRef<WebSocket | null>(null);
	const listenersRef = useRef<Map<LiveUpdateKey, Set<Listener>>>(new Map());

	const subscribe = useCallback((key: LiveUpdateKey, listener: Listener) => {
		let set = listenersRef.current.get(key);
		if (!set) {
			set = new Set();
			listenersRef.current.set(key, set);
		}
		set.add(listener);
		return () => {
			set?.delete(listener);
		};
	}, []);

	useEffect(() => {
		if (!user) {
			if (wsRef.current) {
				wsRef.current.close();
				wsRef.current = null;
			}
			setConnected(false);
			return;
		}
		const t = setTimeout(() => {
			const ws = new WebSocket(getWsUrl());
			attachWsDebugLogging(ws);
			wsRef.current = ws;
			ws.onopen = () => setConnected(true);
			ws.onclose = () => setConnected(false);
			ws.onerror = () => setConnected(false);
			ws.onmessage = (event) => {
				if (typeof event.data !== "string") return;
				try {
					const data = JSON.parse(event.data) as unknown;
					const keys = messageKeys(data);
					for (const k of keys) {
						listenersRef.current.get(k)?.forEach((cb) => cb(data));
					}
				} catch {
					// ignore
				}
			};
		}, CONNECT_DELAY_MS);
		return () => {
			clearTimeout(t);
			if (wsRef.current) {
				wsRef.current.close();
				wsRef.current = null;
			}
			setConnected(false);
		};
	}, [user]);

	const value: LiveWsContextValue = React.useMemo(
		() => ({ connected, subscribe }),
		[connected, subscribe],
	);

	return (
		<LiveWsContext.Provider value={value}>{children}</LiveWsContext.Provider>
	);
}
