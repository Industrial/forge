import { useCallback, useEffect, useState } from "react";
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
import TableHead from "@mui/material/TableHead";
import TableRow from "@mui/material/TableRow";
import IconButton from "@mui/material/IconButton";
import DeleteIcon from "@mui/icons-material/Delete";
import Alert from "@mui/material/Alert";
import CircularProgress from "@mui/material/CircularProgress";

type Assignment = {
	scope: string;
	role_name: string;
	permission_key: string;
};

const SCOPES = ["org", "global"] as const;
const ORG_ROLES = ["owner", "admin", "editor", "viewer"] as const;
const GLOBAL_ROLES = ["platform_admin"] as const;

export default function PermissionsPage() {
	const [assignments, setAssignments] = useState<Assignment[]>([]);
	const [permissions, setPermissions] = useState<string[]>([]);
	const [loading, setLoading] = useState(true);
	const [error, setError] = useState<string | null>(null);
	const [addScope, setAddScope] = useState<string>("org");
	const [addRole, setAddRole] = useState<string>("owner");
	const [addPermission, setAddPermission] = useState<string>("");
	const [adding, setAdding] = useState(false);
	const [deletingKey, setDeletingKey] = useState<string | null>(null);

	const fetchData = useCallback(async () => {
		setError(null);
		try {
			const [assignRes, permRes] = await Promise.all([
				fetch("/api/dashboard/role-permissions", { credentials: "include" }),
				fetch("/api/dashboard/permissions", { credentials: "include" }),
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
	}, []);

	useEffect(() => {
		fetchData();
	}, [fetchData]);

	const handleAdd = async () => {
		if (!addScope || !addRole || !addPermission) return;
		setAdding(true);
		setError(null);
		try {
			const res = await fetch("/api/dashboard/role-permissions", {
				method: "POST",
				headers: { "Content-Type": "application/json" },
				credentials: "include",
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
		const key = `${a.scope}:${a.role_name}:${a.permission_key}`;
		setDeletingKey(key);
		setError(null);
		try {
			const res = await fetch("/api/dashboard/role-permissions", {
				method: "DELETE",
				headers: { "Content-Type": "application/json" },
				credentials: "include",
				body: JSON.stringify({
					scope: a.scope,
					role_name: a.role_name,
					permission_key: a.permission_key,
				}),
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

	if (loading) {
		return (
			<Box sx={{ display: "flex", justifyContent: "center", py: 4 }}>
				<CircularProgress />
			</Box>
		);
	}

	return (
		<>
			<Typography variant="h4" component="h1" gutterBottom>
				Permissions
			</Typography>
			<Typography color="text.secondary" sx={{ mb: 2 }}>
				Manage role–permission assignments. Only users with{" "}
				<code>dashboard.permissions.manage</code> can see this page.
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

			<Table size="small">
				<TableHead>
					<TableRow>
						<TableCell>Scope</TableCell>
						<TableCell>Role</TableCell>
						<TableCell>Permission</TableCell>
						<TableCell align="right">Actions</TableCell>
					</TableRow>
				</TableHead>
				<TableBody>
					{assignments.map((a) => {
						const key = `${a.scope}:${a.role_name}:${a.permission_key}`;
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
					})}
				</TableBody>
			</Table>
			{assignments.length === 0 && !loading && (
				<Typography color="text.secondary" sx={{ mt: 2 }}>
					No assignments yet. Add one above.
				</Typography>
			)}
		</>
	);
}
