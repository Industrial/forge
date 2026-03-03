/**
 * WebSocket URL for the app's /ws endpoint.
 * Uses the same origin as the page so the connection is same-origin (cookies and
 * no cross-origin issues). In dev (Vite on e.g. port 5173), set VITE_BACKEND_URL
 * to your backend (e.g. http://localhost:3000) so Vite proxies /api and /ws to it;
 * the backend must be running before opening the app.
 */
export function getWsUrl(): string {
	const protocol = window.location.protocol === "https:" ? "wss:" : "ws:";
	return `${protocol}//${window.location.host}/ws`;
}

const WS_DEBUG_KEY = "forge_ws_debug";

/** True when WebSocket debug logging should run (dev or localStorage "forge_ws_debug" set). */
export function isWsDebugEnabled(): boolean {
	if (typeof window === "undefined") return false;
	return (
		import.meta.env.DEV ||
		window.localStorage.getItem(WS_DEBUG_KEY) === "1"
	);
}

/**
 * Centralized WebSocket debug logging. Call once after creating a WebSocket;
 * all send and message traffic for that socket will be logged to the console when
 * debug is enabled (dev or localStorage "forge_ws_debug" = "1").
 */
export function attachWsDebugLogging(ws: WebSocket): void {
	if (!isWsDebugEnabled()) return;
	const originalSend = ws.send.bind(ws);
	ws.send = function (data: string | ArrayBufferLike | Blob) {
		try {
			const payload = typeof data === "string" ? JSON.parse(data) : data;
			console.log("[Forge WS] →", payload);
		} catch {
			console.log("[Forge WS] →", data);
		}
		originalSend(data);
	};
	ws.addEventListener("message", (event) => {
		try {
			const payload =
				typeof event.data === "string" ? JSON.parse(event.data) : event.data;
			console.log("[Forge WS] ←", payload);
		} catch {
			console.log("[Forge WS] ←", event.data);
		}
	});
}
