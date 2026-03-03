import { useCallback, useEffect, useRef, useState } from "react";
import { attachWsDebugLogging, getWsUrl } from "@/utils/ws";

/**
 * Live event payloads from the backend (forge-live).
 * Backend sends these on org or channel subscription.
 */
export type LiveEvent =
	| { type: "users_updated"; user_id?: string; org_id?: string }
	| {
			type: "resource_changed";
			resource: string;
			id: string;
			action?: string;
	  };

/**
 * Subscribe to a single WebSocket channel for live updates.
 * When the backend sends an event on this channel, onEvent is called.
 * Use channel "organizations" for org list, or "org:<uuid>" for org-scoped events.
 *
 * @param channel - Channel name (e.g. "organizations" or "org:uuid"), or null to skip connecting.
 * @param onEvent - Called when a LiveEvent is received (parsed from JSON). Ignore irrelevant events inside.
 * @returns { connected: boolean } - True when WebSocket is open and subscription was sent.
 */
export function useLiveChannel(
	channel: string | null,
	onEvent: (event: LiveEvent) => void,
): { connected: boolean } {
	const [connected, setConnected] = useState(false);
	const wsRef = useRef<WebSocket | null>(null);
	const onEventRef = useRef(onEvent);
	onEventRef.current = onEvent;

	const connect = useCallback(() => {
		if (!channel) {
			setConnected(false);
			return;
		}
		const ws = new WebSocket(getWsUrl());
		attachWsDebugLogging(ws);
		wsRef.current = ws;
		ws.onopen = () => {
			ws.send(JSON.stringify({ type: "subscribe", channel }));
			setConnected(true);
		};
		ws.onclose = () => setConnected(false);
		ws.onerror = () => setConnected(false);
		ws.onmessage = (event) => {
			if (typeof event.data !== "string") return;
			try {
				const data = JSON.parse(event.data) as unknown;
				if (data && typeof data === "object" && "type" in data) {
					onEventRef.current(data as LiveEvent);
				}
			} catch {
				// ignore non-JSON or non–live-event messages (e.g. tasks, audit_log)
			}
		};
	}, [channel]);

	// Delay WebSocket until after the page has loaded so the Vite proxy (dev) or
	// same-origin backend is ready; avoids "connection interrupted while page was loading".
	const CONNECT_DELAY_MS = 600;

	useEffect(() => {
		const t = setTimeout(() => connect(), CONNECT_DELAY_MS);
		return () => {
			clearTimeout(t);
			if (wsRef.current) {
				wsRef.current.close();
				wsRef.current = null;
			}
			setConnected(false);
		};
	}, [connect]);

	return { connected };
}
