import { useCallback, useEffect, useState } from "react";
import { useOutletContext } from "react-router-dom";
import Typography from "@mui/material/Typography";
import Box from "@mui/material/Box";
import Button from "@mui/material/Button";
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
import TablePagination from "@mui/material/TablePagination";
import Paper from "@mui/material/Paper";
import IconButton from "@mui/material/IconButton";
import DeleteIcon from "@mui/icons-material/Delete";
import Alert from "@mui/material/Alert";
import CircularProgress from "@mui/material/CircularProgress";
import Chip from "@mui/material/Chip";
import { useTablePaginationDefaults } from "@/hooks/useTablePaginationDefaults";
import { useLiveUpdates } from "@/context/LiveWs";
import { useApi } from "../../../../utils/api";

type Assignment = {
	scope: string;
	role_name: string;
	permission_key: string;
	org_id?: string | null;
};

const SCOPES = ["org", "global"] as const;
const ORG_ROLES = ["owner", "admin", "editor", "viewer"] as const;
const GLOBAL_ROLES = ["platform_admin"] as const;

export default function PermissionsPage() {
	const { api } = useOutletContext<{ api: ReturnType<typeof useApi> }>();
	const [liveRefreshTrigger, setLiveRefreshTrigger] = useState(0);
	const { connected: wsConnected } = useLiveUpdates("role_permissions", () => {
		setLiveRefreshTrigger((n) => n + 1);
	});

	const [assignments, setAssignments] = useState<Assignment[]>([]);
	const [permissions, setPermissions] = useState<string[]>([]);
	const [loading, setLoading] = useState(true);
	const [error, setError] = useState<string | null>(null);
	const [addScope, setAddScope] = useState<string>("org");
	const [addRole, setAddRole] = useState<string>("owner");
	const [addPermission, setAddPermission] = useState<string>("");
	const [adding, setAdding] = useState(false);
	const [deletingKey, setDeletingKey] = useState<string | null>(null);
	const { defaultRowsPerPage, rowsPerPageOptions } = useTablePaginationDefaults();
	const [page, setPage] = useState(0);
	const [rowsPerPage, setRowsPerPage] = useState(defaultRowsPerPage);

	const fetchData = useCallback(async () => {
		setError(null);
		try {
			const [assignRes, permRes] = await Promise.all([
				api("/api/dashboard/role-permissions"),
				api("/api/dashboard/permissions"),
			]);
			if (!assignRes.ok || !permRes.ok) {
				if (assignRes.status === 403 || permRes.status === 403) {
					setError("You do not have permission to manage permissions.");
				} else {
					setError("Failed to load data.");
				}
				setAssignments([]);
				setPermissions([]);
				return;
			}
			const assignData = await assignRes.json();
			const permData = await permRes.json();
			const permList = permData.permissions ?? [];
			setAssignments(assignData.assignments ?? []);
			setPermissions(permList);
			setAddPermission((prev) =>
				permList.length && !permList.includes(prev) ? permList[0] : prev,
			);
		} catch {
			setError("Failed to load data.");
			setAssignments([]);
			setPermissions([]);
		} finally {
			setLoading(false);
		}
	}, [api]);

	useEffect(() => {
		fetchData();
	}, [fetchData, liveRefreshTrigger]);

	// Sync rowsPerPage when breakpoint default changes (e.g. window resize)
	useEffect(() => {
		setRowsPerPage(defaultRowsPerPage);
		setPage(0);
	}, [defaultRowsPerPage]);

	// Reset page if it goes out of range (e.g. after deleting items)
	useEffect(() => {
		const maxPage = Math.max(0, Math.ceil(assignments.length / rowsPerPage) - 1);
		if (page > maxPage) setPage(maxPage);
	}, [assignments.length, rowsPerPage, page]);

	const handleAdd = async () => {
		if (!addScope || !addRole || !addPermission) return;
		setAdding(true);
		setError(null);
		try {
			const res = await api("/api/dashboard/role-permissions", {
				method: "POST",
				headers: { "Content-Type": "application/json" },
				body: JSON.stringify({
					scope: addScope,
					role_name: addRole,
					permission_key: addPermission,
				}),
			});
			if (res.status === 403) {
				setError("You do not have permission to manage permissions.");
				return;
			}
			if (!res.ok) {
				const data = await res.json().catch(() => ({}));
				setError(data.error ?? "Failed to add assignment.");
				return;
			}
			await fetchData();
		} catch {
			setError("Failed to add assignment.");
		} finally {
			setAdding(false);
		}
	};

	const handleDelete = async (a: Assignment) => {
		const key = [a.scope, a.role_name, a.permission_key, a.org_id ?? ""].join(":");
		setDeletingKey(key);
		setError(null);
		try {
			const body: Record<string, string> = {
				scope: a.scope,
				role_name: a.role_name,
				permission_key: a.permission_key,
			};
			if (a.org_id != null && a.org_id !== "") body.org_id = a.org_id;
			const res = await api("/api/dashboard/role-permissions", {
				method: "DELETE",
				headers: { "Content-Type": "application/json" },
				body: JSON.stringify(body),
			});
			if (res.status === 403) {
				setError("You do not have permission to manage permissions.");
				return;
			}
			if (res.ok) {
				await fetchData();
			} else {
				const data = await res.json().catch(() => ({}));
				setError(data.error ?? "Failed to remove assignment.");
			}
		} catch {
			setError("Failed to remove assignment.");
		} finally {
			setDeletingKey(null);
		}
	};

	const roles = addScope === "global" ? [...GLOBAL_ROLES] : [...ORG_ROLES];

	const handleChangePage = (_: unknown, newPage: number) => {
		setPage(newPage);
	};

	const handleChangeRowsPerPage = (e: React.ChangeEvent<HTMLInputElement>) => {
		setRowsPerPage(parseInt(e.target.value, 10));
		setPage(0);
	};

	const paginatedAssignments = assignments.slice(
		page * rowsPerPage,
		page * rowsPerPage + rowsPerPage,
	);

	if (loading) {
		return (
			<Box sx={{ display: "flex", justifyContent: "center", py: 4 }}>
				<CircularProgress />
			</Box>
		);
	}

	return (
		<>
			<Box sx={{ display: "flex", alignItems: "center", gap: 1, mb: 0 }}>
				<Typography variant="h4" component="h1" gutterBottom sx={{ mb: 0 }}>
					Permissions
				</Typography>
				{wsConnected && <Chip label="Live" color="success" size="small" />}
			</Box>
			<Typography color="text.secondary" sx={{ mb: 2 }}>
				View and manage role–permission assignments. List/view requires{" "}
				<code>dashboard.permissions.read</code>; add/delete requires{" "}
				<code>dashboard.permissions.write</code>.
			</Typography>

			{error && (
				<Alert severity="error" sx={{ mb: 2 }} onClose={() => setError(null)}>
					{error}
				</Alert>
			)}

			<Box
				sx={{
					display: "flex",
					flexWrap: "wrap",
					gap: 2,
					alignItems: "center",
					mb: 3,
				}}
			>
				<FormControl size="small" sx={{ minWidth: 100 }}>
					<InputLabel>Scope</InputLabel>
					<Select
						value={addScope}
						label="Scope"
						onChange={(e) => {
							setAddScope(e.target.value);
							setAddRole(
								e.target.value === "global" ? "platform_admin" : "owner",
							);
						}}
					>
						{SCOPES.map((s) => (
							<MenuItem key={s} value={s}>
								{s}
							</MenuItem>
						))}
					</Select>
				</FormControl>
				<FormControl size="small" sx={{ minWidth: 140 }}>
					<InputLabel>Role</InputLabel>
					<Select
						value={addRole}
						label="Role"
						onChange={(e) => setAddRole(e.target.value)}
					>
						{roles.map((r) => (
							<MenuItem key={r} value={r}>
								{r}
							</MenuItem>
						))}
					</Select>
				</FormControl>
				<FormControl size="small" sx={{ minWidth: 200 }}>
					<InputLabel>Permission</InputLabel>
					<Select
						value={addPermission}
						label="Permission"
						onChange={(e) => setAddPermission(e.target.value)}
					>
						{permissions.map((p) => (
							<MenuItem key={p} value={p}>
								{p}
							</MenuItem>
						))}
					</Select>
				</FormControl>
				<Button
					variant="contained"
					onClick={handleAdd}
					disabled={adding || !addPermission}
				>
					{adding ? "Adding…" : "Add"}
				</Button>
			</Box>

			<TableContainer component={Paper}>
				<Table size="small" aria-label="Role–permission assignments">
					<TableHead>
						<TableRow>
							<TableCell>Scope</TableCell>
							<TableCell>Role</TableCell>
							<TableCell>Permission</TableCell>
							<TableCell align="right">Actions</TableCell>
						</TableRow>
					</TableHead>
					<TableBody>
						{paginatedAssignments.length === 0 ? (
							<TableRow>
								<TableCell colSpan={4} align="center">
									No assignments yet. Add one above.
								</TableCell>
							</TableRow>
						) : (
							paginatedAssignments.map((a) => {
								const key = [a.scope, a.role_name, a.permission_key, a.org_id ?? ""].join(":");
								return (
									<TableRow key={key}>
										<TableCell>{a.scope}</TableCell>
										<TableCell>{a.role_name}</TableCell>
										<TableCell>{a.permission_key}</TableCell>
										<TableCell align="right">
											<IconButton
												size="small"
												aria-label="Remove"
												onClick={() => handleDelete(a)}
												disabled={deletingKey === key}
											>
												<DeleteIcon />
											</IconButton>
										</TableCell>
									</TableRow>
								);
							})
						)}
					</TableBody>
				</Table>
			</TableContainer>
			<TablePagination
				component="div"
				count={assignments.length}
				page={page}
				onPageChange={handleChangePage}
				rowsPerPage={rowsPerPage}
				onRowsPerPageChange={handleChangeRowsPerPage}
				rowsPerPageOptions={rowsPerPageOptions}
				labelRowsPerPage="Rows per page:"
			/>
		</>
	);
}
