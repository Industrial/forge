import { useState, useEffect, useRef } from 'react';

const wsUrl = '/ws';

export default function WsDemo() {
  const [messages, setMessages] = useState<string[]>([]);
  const [input, setInput] = useState('');
  const wsRef = useRef<WebSocket | null>(null);

  useEffect(() => {
    const protocol = window.location.protocol === 'https:' ? 'wss:' : 'ws:';
    const url = `${protocol}//${window.location.host}${wsUrl}`;
    const ws = new WebSocket(url);
    wsRef.current = ws;
    ws.onmessage = (e) => setMessages((m) => [...m, e.data]);
    ws.onclose = () => (wsRef.current = null);
    return () => { ws.close(); };
  }, []);

  const send = () => {
    if (wsRef.current?.readyState === WebSocket.OPEN && input.trim()) {
      wsRef.current.send(input);
      setInput('');
    }
  };

  return (
    <div>
      <h1>WebSocket Demo</h1>
      <p>Connected to {wsUrl}. Type and send messages.</p>
      <input value={input} onChange={(e) => setInput(e.target.value)} onKeyDown={(e) => e.key === 'Enter' && send()} />
      <button onClick={send}>Send</button>
      <ul>{messages.map((msg, i) => <li key={i}>{msg}</li>)}</ul>
      <a href="/">Home</a>
    </div>
  );
}
