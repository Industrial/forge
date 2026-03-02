import { useCallback, useEffect, useRef, useState } from "react";
import Box from "@mui/material/Box";
import Chip from "@mui/material/Chip";
import CircularProgress from "@mui/material/CircularProgress";
import Paper from "@mui/material/Paper";
import Table from "@mui/material/Table";
import TableBody from "@mui/material/TableBody";
import TableCell from "@mui/material/TableCell";
import TableContainer from "@mui/material/TableContainer";
import TableHead from "@mui/material/TableHead";
import TableRow from "@mui/material/TableRow";
import Typography from "@mui/material/Typography";

export type TaskStatus = "planned" | "running" | "ran";

export type Task = {
	id: string;
	name: string;
	status: TaskStatus;
	scheduled_at?: string;
	started_at?: string;
	finished_at?: string;
};

function formatDate(iso: string | undefined): string {
	if (!iso) return "—";
	try {
		return new Date(iso).toLocaleString();
	} catch {
		return iso;
	}
}

function StatusChip({ status }: { status: TaskStatus }) {
	const label =
		status === "planned"
			? "Planned"
			: status === "running"
				? "Running"
				: "Ran";
	const color =
		status === "planned"
			? "default"
			: status === "running"
				? "primary"
				: "success";
	return <Chip label={label} color={color} size="small" />;
}

export default function TasksPage() {
	const [tasks, setTasks] = useState<Task[]>([]);
	const [loading, setLoading] = useState(true);
	const [wsConnected, setWsConnected] = useState(false);
	const wsRef = useRef<WebSocket | null>(null);

	const connectWs = useCallback(() => {
		const protocol = window.location.protocol === "https:" ? "wss:" : "ws:";
		const wsUrl = `${protocol}//${window.location.host}/ws`;
		const ws = new WebSocket(wsUrl);
		wsRef.current = ws;
		ws.onopen = () => {
			setWsConnected(true);
			ws.send(
				JSON.stringify({ type: "subscribe", channel: "tasks" }),
			);
		};
		ws.onclose = () => setWsConnected(false);
		ws.onmessage = (event) => {
			if (typeof event.data !== "string") return;
			try {
				const data = JSON.parse(event.data);
				if (data?.type === "tasks" && Array.isArray(data.tasks)) {
					setTasks(data.tasks);
				}
			} catch {
				// ignore non-JSON or invalid
			}
		};
	}, []);

	useEffect(() => {
		let cancelled = false;
		async function fetchTasks() {
			try {
				const res = await fetch("/api/dashboard/tasks", {
					credentials: "include",
				});
				if (res.ok && !cancelled) {
					const data = await res.json();
					setTasks(data.tasks ?? []);
				}
			} catch {
				if (!cancelled) setTasks([]);
			} finally {
				if (!cancelled) setLoading(false);
			}
		}
		fetchTasks();
		connectWs();
		return () => {
			cancelled = true;
			if (wsRef.current) {
				wsRef.current.close();
				wsRef.current = null;
			}
			setWsConnected(false);
		};
	}, [connectWs]);

	// Order: Running first, then Planned, then Ran (active and upcoming at top)
	const statusOrder = (t: Task) =>
		t.status === "running" ? 0 : t.status === "planned" ? 1 : 2;
	const orderedTasks = [...tasks].sort(
		(a, b) => statusOrder(a) - statusOrder(b),
	);

	return (
		<>
			<Box sx={{ display: "flex", alignItems: "center", gap: 1, mb: 0 }}>
				<Typography variant="h4" component="h1" gutterBottom sx={{ mb: 0 }}>
					Tasks
				</Typography>
				{wsConnected && (
					<Chip label="Live" color="success" size="small" />
				)}
			</Box>
			<Typography variant="body2" color="text.secondary" sx={{ mb: 2 }}>
				Scheduled and recent task runs. Updates in real time over WebSocket.
			</Typography>
			{loading ? (
				<Box sx={{ display: "flex", justifyContent: "center", py: 4 }}>
					<CircularProgress />
				</Box>
			) : (
				<TableContainer component={Paper}>
					<Table size="small" aria-label="Tasks">
						<TableHead>
							<TableRow>
								<TableCell>Name</TableCell>
								<TableCell>Status</TableCell>
								<TableCell>Scheduled at</TableCell>
								<TableCell>Started at</TableCell>
								<TableCell>Finished at</TableCell>
							</TableRow>
						</TableHead>
						<TableBody>
							{orderedTasks.length === 0 ? (
								<TableRow>
									<TableCell colSpan={5} align="center">
										No tasks.
									</TableCell>
								</TableRow>
							) : (
								orderedTasks.map((task) => (
									<TableRow key={task.id}>
										<TableCell sx={{ fontWeight: 500 }}>
											{task.name}
										</TableCell>
										<TableCell>
											<StatusChip status={task.status} />
										</TableCell>
										<TableCell sx={{ whiteSpace: "nowrap" }}>
											{formatDate(task.scheduled_at)}
										</TableCell>
										<TableCell sx={{ whiteSpace: "nowrap" }}>
											{formatDate(task.started_at)}
										</TableCell>
										<TableCell sx={{ whiteSpace: "nowrap" }}>
											{formatDate(task.finished_at)}
										</TableCell>
									</TableRow>
								))
							)}
						</TableBody>
					</Table>
				</TableContainer>
			)}
		</>
	);
}
