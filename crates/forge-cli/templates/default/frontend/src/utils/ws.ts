/**
 * WebSocket URL for the app's /ws endpoint.
 * Always uses the same origin as the page so the connection is same-origin (cookies
 * and no cross-origin issues). In dev, Vite proxies /ws to the backend.
 */
export function getWsUrl(): string {
	const protocol = window.location.protocol === "https:" ? "wss:" : "ws:";
	return `${protocol}//${window.location.host}/ws`;
}
