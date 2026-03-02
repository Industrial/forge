import { useEffect, useRef, useState } from "react";

export default function WebsocketsDemoPage() {
	const [message, setMessage] = useState("");
	const [echoed, setEchoed] = useState<string[]>([]);
	const wsRef = useRef<WebSocket | null>(null);

	useEffect(() => {
		try {
			const protocol = window.location.protocol === "https:" ? "wss:" : "ws:";
			const wsUrl = `${protocol}//${window.location.host}/ws`;
			const ws = new WebSocket(wsUrl);
			wsRef.current = ws;
			ws.onmessage = (event) => {
				if (typeof event.data === "string") {
					setEchoed((prev) => [...prev, event.data]);
				}
			};
			return () => {
				ws.close();
				wsRef.current = null;
			};
		} catch {
			return () => {};
		}
	}, []);

	function handleSend() {
		if (wsRef.current?.readyState === WebSocket.OPEN && message.trim()) {
			wsRef.current.send(message.trim());
			setMessage("");
		}
	}

	return (
		<div data-testid="ws-demo-root">
			<h1>WebSocket demo</h1>
			<div
				style={{
					display: "flex",
					gap: 8,
					alignItems: "center",
					marginBottom: 16,
				}}
			>
				<input
					data-testid="ws-demo-input"
					type="text"
					value={message}
					onChange={(e) => setMessage(e.target.value)}
					onKeyDown={(e) => e.key === "Enter" && handleSend()}
					aria-label="Message"
				/>
				<button data-testid="ws-demo-send" type="button" onClick={handleSend}>
					Send
				</button>
			</div>
			<div data-testid="ws-demo-messages">
				{echoed.length > 0 ? echoed.join(", ") : "No messages yet."}
			</div>
		</div>
	);
}
