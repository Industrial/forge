import { useCallback, useEffect, useState } from "react";
import Typography from "@mui/material/Typography";
import Box from "@mui/material/Box";
import Button from "@mui/material/Button";
import TextField from "@mui/material/TextField";
import Table from "@mui/material/Table";
import TableBody from "@mui/material/TableBody";
import TableCell from "@mui/material/TableCell";
import TableContainer from "@mui/material/TableContainer";
import TableHead from "@mui/material/TableHead";
import TableRow from "@mui/material/TableRow";
import Paper from "@mui/material/Paper";
import IconButton from "@mui/material/IconButton";
import AddIcon from "@mui/icons-material/Add";
import DeleteIcon from "@mui/icons-material/Delete";
import EditIcon from "@mui/icons-material/Edit";
import Alert from "@mui/material/Alert";
import CircularProgress from "@mui/material/CircularProgress";
import Chip from "@mui/material/Chip";
import FormDialog from "../../../../components/FormDialog";
import FormControl from "@mui/material/FormControl";
import InputLabel from "@mui/material/InputLabel";
import Select from "@mui/material/Select";
import MenuItem from "@mui/material/MenuItem";
import { useLiveUpdates } from "../../../../context/LiveWs";

type Role = {
	id: string;
	org_id: string;
	name: string;
	display_name: string | null;
	created_at: string;
	updated_at: string;
};

type Organization = { id: string; name: string; slug: string };

export default function RolesPage() {
	const [liveRefreshTrigger, setLiveRefreshTrigger] = useState(0);
	const { connected: wsConnected } = useLiveUpdates("roles", () => {
		setLiveRefreshTrigger((n) => n + 1);
	});

	const [roles, setRoles] = useState<Role[]>([]);
	const [organizations, setOrganizations] = useState<Organization[]>([]);
	const [loading, setLoading] = useState(true);
	const [error, setError] = useState<string | null>(null);
	const [addOpen, setAddOpen] = useState(false);
	const [addName, setAddName] = useState("");
	const [addDisplayName, setAddDisplayName] = useState("");
	const [addOrgId, setAddOrgId] = useState("");
	const [adding, setAdding] = useState(false);
	const [editRole, setEditRole] = useState<Role | null>(null);
	const [editName, setEditName] = useState("");
	const [editDisplayName, setEditDisplayName] = useState("");
	const [saving, setSaving] = useState(false);
	const [deletingId, setDeletingId] = useState<string | null>(null);

	const fetchRoles = useCallback(async () => {
		setError(null);
		try {
			const res = await fetch("/api/dashboard/roles", { credentials: "include" });
			if (res.status === 403) {
				setError("You do not have permission to view roles.");
				setRoles([]);
				return;
			}
			if (!res.ok) {
				setError("Failed to load roles.");
				setRoles([]);
				return;
			}
			const data = await res.json();
			setRoles(data.roles ?? []);
		} catch {
			setError("Failed to load roles.");
			setRoles([]);
		} finally {
			setLoading(false);
		}
	}, []);

	const fetchOrgs = useCallback(async () => {
		try {
			const res = await fetch("/api/dashboard/organizations", { credentials: "include" });
			if (res.ok) {
				const data = await res.json();
				setOrganizations(data.organizations ?? []);
			}
		} catch {
			// optional
		}
	}, []);

	useEffect(() => {
		fetchRoles();
	}, [fetchRoles, liveRefreshTrigger]);

	useEffect(() => {
		fetchOrgs();
	}, [fetchOrgs]);

	const handleAdd = async () => {
		const name = addName.trim();
		if (!name) {
			setError("Name is required.");
			return;
		}
		setAdding(true);
		setError(null);
		try {
			const orgId = addOrgId || organizations[0]?.id;
			const body: { name: string; display_name?: string; org_id?: string } = {
				name,
				display_name: addDisplayName.trim() || undefined,
			};
			if (orgId) body.org_id = orgId;
			const res = await fetch("/api/dashboard/roles", {
				method: "POST",
				headers: { "Content-Type": "application/json" },
				credentials: "include",
				body: JSON.stringify(body),
			});
			if (res.status === 403) {
				setError("You do not have permission to create roles.");
				return;
			}
			if (!res.ok) {
				const data = await res.json().catch(() => ({}));
				setError(data.error ?? "Failed to create role.");
				return;
			}
			setAddName("");
			setAddDisplayName("");
			setAddOrgId(organizations[0]?.id ?? "");
			setAddOpen(false);
			await fetchRoles();
		} catch {
			setError("Failed to create role.");
		} finally {
			setAdding(false);
		}
	};

	const openEdit = (role: Role) => {
		setEditRole(role);
		setEditName(role.name);
		setEditDisplayName(role.display_name ?? "");
	};

	const handleSaveEdit = async () => {
		if (!editRole) return;
		setSaving(true);
		setError(null);
		try {
			const res = await fetch("/api/dashboard/roles", {
				method: "PATCH",
				headers: { "Content-Type": "application/json" },
				credentials: "include",
				body: JSON.stringify({
					id: editRole.id,
					name: editName.trim() || undefined,
					display_name: editDisplayName.trim() || undefined,
				}),
			});
			if (res.status === 403) {
				setError("You do not have permission to update roles.");
				return;
			}
			if (!res.ok) {
				const data = await res.json().catch(() => ({}));
				setError(data.error ?? "Failed to update role.");
				return;
			}
			setEditRole(null);
			await fetchRoles();
		} catch {
			setError("Failed to update role.");
		} finally {
			setSaving(false);
		}
	};

	const handleDelete = async (id: string) => {
		setDeletingId(id);
		setError(null);
		try {
			const res = await fetch("/api/dashboard/roles", {
				method: "DELETE",
				headers: { "Content-Type": "application/json" },
				credentials: "include",
				body: JSON.stringify({ id }),
			});
			if (res.status === 403) {
				setError("You do not have permission to delete roles.");
				return;
			}
			if (res.ok) {
				await fetchRoles();
			} else {
				const data = await res.json().catch(() => ({}));
				setError(data.error ?? "Failed to delete role.");
			}
		} catch {
			setError("Failed to delete role.");
		} finally {
			setDeletingId(null);
		}
	};

	return (
		<>
			<Box sx={{ display: "flex", alignItems: "center", gap: 1, mb: 0 }}>
				<Typography variant="h4" component="h1" gutterBottom sx={{ mb: 0 }}>
					Roles
				</Typography>
				{wsConnected && <Chip label="Live" color="success" size="small" />}
			</Box>
			<Typography variant="body2" color="text.secondary" sx={{ mb: 2 }}>
				Manage organization roles. Template roles (owner, admin, editor, viewer) are created when an org is created; you can add custom roles here.
			</Typography>

			{error && (
				<Alert severity="error" sx={{ mb: 2 }} onClose={() => setError(null)}>
					{error}
				</Alert>
			)}

			<Box sx={{ mb: 2 }}>
				<Button
					variant="contained"
					startIcon={<AddIcon />}
					onClick={() => {
						setAddOrgId(organizations[0]?.id ?? "");
						setAddOpen(true);
					}}
				>
					Add role
				</Button>
			</Box>

			{loading ? (
				<Box sx={{ display: "flex", justifyContent: "center", py: 4 }}>
					<CircularProgress />
				</Box>
			) : (
				<TableContainer component={Paper}>
					<Table size="small" aria-label="Roles">
						<TableHead>
							<TableRow>
								<TableCell>Organization</TableCell>
								<TableCell>Name</TableCell>
								<TableCell>Display name</TableCell>
								<TableCell align="right">Actions</TableCell>
							</TableRow>
						</TableHead>
						<TableBody>
							{roles.length === 0 ? (
								<TableRow>
									<TableCell colSpan={4} align="center">
										No roles. Add a role or ensure your organization has template roles.
									</TableCell>
								</TableRow>
							) : (
								roles.map((role) => (
									<TableRow key={role.id}>
										<TableCell>
											{organizations.find((o) => o.id === role.org_id)?.name ?? role.org_id}
										</TableCell>
										<TableCell sx={{ fontWeight: 500 }}>{role.name}</TableCell>
										<TableCell>{role.display_name ?? "—"}</TableCell>
										<TableCell align="right">
											<IconButton
												size="small"
												aria-label="Edit"
												onClick={() => openEdit(role)}
											>
												<EditIcon />
											</IconButton>
											<IconButton
												size="small"
												aria-label="Delete"
												onClick={() => handleDelete(role.id)}
												disabled={deletingId === role.id}
											>
												<DeleteIcon />
											</IconButton>
										</TableCell>
									</TableRow>
								))
							)}
						</TableBody>
					</Table>
				</TableContainer>
			)}

			<FormDialog
				open={addOpen}
				onClose={() => setAddOpen(false)}
				title="Add role"
				submitLabel="Add"
				submittingLabel="Adding…"
				onSubmit={handleAdd}
				submitDisabled={!addName.trim() || (organizations.length > 1 && !addOrgId)}
				submitting={adding}
			>
				<Box sx={{ display: "flex", flexDirection: "column", gap: 2, pt: 1, minWidth: 320 }}>
					{organizations.length > 1 && (
						<FormControl fullWidth size="small" disabled={adding} required>
							<InputLabel>Organization</InputLabel>
							<Select
								label="Organization"
								value={addOrgId}
								onChange={(e) => setAddOrgId(e.target.value)}
							>
								{organizations.map((org) => (
									<MenuItem key={org.id} value={org.id}>
										{org.name}
									</MenuItem>
								))}
							</Select>
						</FormControl>
					)}
					<TextField
						label="Name"
						value={addName}
						onChange={(e) => setAddName(e.target.value)}
						placeholder="e.g. writer"
						required
						disabled={adding}
						fullWidth
					/>
					<TextField
						label="Display name (optional)"
						value={addDisplayName}
						onChange={(e) => setAddDisplayName(e.target.value)}
						placeholder="e.g. Content writer"
						disabled={adding}
						fullWidth
					/>
				</Box>
			</FormDialog>

			<FormDialog
				open={Boolean(editRole)}
				onClose={() => setEditRole(null)}
				title="Edit role"
				submitLabel="Save"
				submittingLabel="Saving…"
				onSubmit={handleSaveEdit}
				submitDisabled={!editName.trim()}
				submitting={saving}
			>
				<Box sx={{ display: "flex", flexDirection: "column", gap: 2, pt: 1, minWidth: 320 }}>
					<TextField
						label="Name"
						value={editName}
						onChange={(e) => setEditName(e.target.value)}
						required
						disabled={saving}
						fullWidth
					/>
					<TextField
						label="Display name (optional)"
						value={editDisplayName}
						onChange={(e) => setEditDisplayName(e.target.value)}
						disabled={saving}
						fullWidth
					/>
				</Box>
			</FormDialog>
		</>
	);
}
