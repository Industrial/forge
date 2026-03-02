import { useCallback, useEffect, useRef, useState } from "react";
import Typography from "@mui/material/Typography";
import Box from "@mui/material/Box";
import Button from "@mui/material/Button";
import TextField from "@mui/material/TextField";
import FormControl from "@mui/material/FormControl";
import InputLabel from "@mui/material/InputLabel";
import MenuItem from "@mui/material/MenuItem";
import Select from "@mui/material/Select";
import Table from "@mui/material/Table";
import TableBody from "@mui/material/TableBody";
import TableCell from "@mui/material/TableCell";
import TableContainer from "@mui/material/TableContainer";
import TableHead from "@mui/material/TableHead";
import TableRow from "@mui/material/TableRow";
import Paper from "@mui/material/Paper";
import Alert from "@mui/material/Alert";
import CircularProgress from "@mui/material/CircularProgress";
import TablePagination from "@mui/material/TablePagination";

type AuditLogEntry = {
	id: string;
	event_kind: string;
	actor_id: string;
	subject_id?: string | null;
	organization_id?: string | null;
	action: string;
	resource_type: string;
	resource_id?: string | null;
	outcome: string;
	reason?: string | null;
	occurred_at: string;
};

const OUTCOMES = ["success", "failure", "allowed", "denied"] as const;
const EVENT_KINDS = ["auth", "authz", "mutation", "custom"] as const;
const ACTIONS = ["read", "create", "update", "delete", "manage"] as const;

export default function AuditLogPage() {
	const [entries, setEntries] = useState<AuditLogEntry[]>([]);
	const [total, setTotal] = useState(0);
	const [loading, setLoading] = useState(true);
	const [error, setError] = useState<string | null>(null);
	const [page, setPage] = useState(0);
	const [rowsPerPage, setRowsPerPage] = useState(50);
	const [from, setFrom] = useState("");
	const [to, setTo] = useState("");
	const [outcome, setOutcome] = useState("");
	const [eventKind, setEventKind] = useState("");
	const [action, setAction] = useState("");
	const [reason, setReason] = useState("");

	const fetchData = useCallback(async () => {
		setError(null);
		setLoading(true);
		try {
			const params = new URLSearchParams();
			params.set("limit", String(rowsPerPage));
			params.set("offset", String(page * rowsPerPage));
			if (from) params.set("from", from);
			if (to) params.set("to", to);
			if (outcome) params.set("outcome", outcome);
			if (eventKind) params.set("event_kind", eventKind);
			if (action) params.set("action", action);
			if (reason.trim()) params.set("reason", reason.trim());
			const res = await fetch(`/api/dashboard/audit-log?${params.toString()}`, {
				credentials: "include",
			});
			if (res.status === 403) {
				setError("You do not have permission to view the audit log.");
				setEntries([]);
				setTotal(0);
				return;
			}
			if (!res.ok) {
				setError("Failed to load audit log.");
				setEntries([]);
				setTotal(0);
				return;
			}
			const data = await res.json();
			setEntries(data.entries ?? []);
			setTotal(data.total ?? 0);
		} catch {
			setError("Failed to load audit log.");
			setEntries([]);
			setTotal(0);
		} finally {
			setLoading(false);
		}
	}, [page, rowsPerPage, from, to, outcome, eventKind, action, reason]);

	useEffect(() => {
		fetchData();
	}, [fetchData]);

	// Live updates: subscribe to audit-log WebSocket channel and prepend new entries
	const entriesRef = useRef(entries);
	entriesRef.current = entries;
	useEffect(() => {
		const proto = window.location.protocol === "https:" ? "wss:" : "ws:";
		const wsUrl = `${proto}//${window.location.host}/ws`;
		const ws = new WebSocket(wsUrl);
		ws.onopen = () => {
			ws.send(JSON.stringify({ type: "subscribe", channel: "audit-log" }));
		};
		ws.onmessage = (event) => {
			try {
				const msg = JSON.parse(event.data);
				if (msg?.type === "audit_log" && msg?.entry) {
					setEntries((prev) => [msg.entry as AuditLogEntry, ...prev]);
					setTotal((prev) => prev + 1);
				}
			} catch {
				// ignore non-JSON or invalid messages
			}
		};
		return () => ws.close();
	}, []);

	const handleApplyFilters = () => {
		setPage(0);
	};

	const handleResetFilters = () => {
		setFrom("");
		setTo("");
		setOutcome("");
		setEventKind("");
		setAction("");
		setReason("");
		setPage(0);
	};

	const handleChangePage = (_: unknown, newPage: number) => {
		setPage(newPage);
	};

	const handleChangeRowsPerPage = (e: React.ChangeEvent<HTMLInputElement>) => {
		setRowsPerPage(parseInt(e.target.value, 10));
		setPage(0);
	};

	const formatDate = (s: string) => {
		try {
			const d = new Date(s);
			return Number.isNaN(d.getTime()) ? s : d.toLocaleString();
		} catch {
			return s;
		}
	};

	return (
		<>
			<Typography variant="h4" component="h1" gutterBottom>
				Audit log
			</Typography>
			<Typography variant="body2" color="text.secondary" sx={{ mb: 2 }}>
				Read-only list of audit events. Use filters to narrow results.
			</Typography>

			<Paper sx={{ p: 2, mb: 2 }}>
				<Typography variant="subtitle2" gutterBottom>
					Filters
				</Typography>
				<Box
					sx={{
						display: "flex",
						flexWrap: "wrap",
						gap: 2,
						alignItems: "flex-end",
					}}
				>
					<TextField
						label="From (date)"
						type="date"
						size="small"
						value={from}
						onChange={(e) => setFrom(e.target.value)}
						InputLabelProps={{ shrink: true }}
						sx={{ minWidth: 140 }}
					/>
					<TextField
						label="To (date)"
						type="date"
						size="small"
						value={to}
						onChange={(e) => setTo(e.target.value)}
						InputLabelProps={{ shrink: true }}
						sx={{ minWidth: 140 }}
					/>
					<FormControl size="small" sx={{ minWidth: 120 }}>
						<InputLabel>Outcome</InputLabel>
						<Select
							value={outcome}
							label="Outcome"
							onChange={(e) => setOutcome(e.target.value)}
						>
							<MenuItem value="">All</MenuItem>
							{OUTCOMES.map((o) => (
								<MenuItem key={o} value={o}>
									{o}
								</MenuItem>
							))}
						</Select>
					</FormControl>
					<FormControl size="small" sx={{ minWidth: 120 }}>
						<InputLabel>Event kind</InputLabel>
						<Select
							value={eventKind}
							label="Event kind"
							onChange={(e) => setEventKind(e.target.value)}
						>
							<MenuItem value="">All</MenuItem>
							{EVENT_KINDS.map((k) => (
								<MenuItem key={k} value={k}>
									{k}
								</MenuItem>
							))}
						</Select>
					</FormControl>
					<FormControl size="small" sx={{ minWidth: 100 }}>
						<InputLabel>Action</InputLabel>
						<Select
							value={action}
							label="Action"
							onChange={(e) => setAction(e.target.value)}
						>
							<MenuItem value="">All</MenuItem>
							{ACTIONS.map((a) => (
								<MenuItem key={a} value={a}>
									{a}
								</MenuItem>
							))}
						</Select>
					</FormControl>
					<TextField
						label="Reason contains"
						size="small"
						value={reason}
						onChange={(e) => setReason(e.target.value)}
						placeholder="Search in reason"
						sx={{ minWidth: 160 }}
					/>
					<Button variant="contained" onClick={handleApplyFilters}>
						Apply
					</Button>
					<Button onClick={handleResetFilters}>Reset</Button>
				</Box>
			</Paper>

			{error && (
				<Alert severity="error" sx={{ mb: 2 }}>
					{error}
				</Alert>
			)}

			{loading ? (
				<Box sx={{ display: "flex", justifyContent: "center", py: 4 }}>
					<CircularProgress />
				</Box>
			) : (
				<>
					<TableContainer component={Paper}>
						<Table size="small" aria-label="Audit log">
							<TableHead>
								<TableRow>
									<TableCell>Time</TableCell>
									<TableCell>Actor ID</TableCell>
									<TableCell>Event</TableCell>
									<TableCell>Action</TableCell>
									<TableCell>Resource</TableCell>
									<TableCell>Outcome</TableCell>
									<TableCell>Reason</TableCell>
								</TableRow>
							</TableHead>
							<TableBody>
								{entries.length === 0 ? (
									<TableRow>
										<TableCell colSpan={7} align="center">
											No entries
										</TableCell>
									</TableRow>
								) : (
									entries.map((row) => (
										<TableRow key={row.id}>
											<TableCell sx={{ whiteSpace: "nowrap" }}>
												{formatDate(row.occurred_at)}
											</TableCell>
											<TableCell
												sx={{
													fontFamily: "monospace",
													fontSize: "0.75rem",
												}}
											>
												{row.actor_id.slice(0, 8)}…
											</TableCell>
											<TableCell>{row.event_kind}</TableCell>
											<TableCell>{row.action}</TableCell>
											<TableCell>{row.resource_type}</TableCell>
											<TableCell>{row.outcome}</TableCell>
											<TableCell
												sx={{
													maxWidth: 200,
													overflow: "hidden",
													textOverflow: "ellipsis",
												}}
											>
												{row.reason ?? "—"}
											</TableCell>
										</TableRow>
									))
								)}
							</TableBody>
						</Table>
					</TableContainer>
					<TablePagination
						component="div"
						count={total}
						page={page}
						onPageChange={handleChangePage}
						rowsPerPage={rowsPerPage}
						onRowsPerPageChange={handleChangeRowsPerPage}
						rowsPerPageOptions={[25, 50, 100, 200]}
						labelRowsPerPage="Rows per page:"
					/>
				</>
			)}
		</>
	);
}
