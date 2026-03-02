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
import Dialog from "@mui/material/Dialog";
import DialogTitle from "@mui/material/DialogTitle";
import DialogContent from "@mui/material/DialogContent";
import DialogActions from "@mui/material/DialogActions";
import FormControlLabel from "@mui/material/FormControlLabel";
import Checkbox from "@mui/material/Checkbox";
import FormControl from "@mui/material/FormControl";
import InputLabel from "@mui/material/InputLabel";
import Select from "@mui/material/Select";
import MenuItem from "@mui/material/MenuItem";
import { useSession } from "../../../../context/Session";

type Membership = {
	org_id: string;
	org_name: string;
	role: string;
};

type User = {
	id: string;
	email: string;
	is_active: boolean;
	is_admin: boolean;
	created_at: string;
	memberships: Membership[];
};

type Organization = {
	id: string;
	name: string;
	slug: string;
};

const USERS_READ = "dashboard.users.read";
const USERS_WRITE = "dashboard.users.write";
const ROLES = ["owner", "admin", "editor", "viewer"];

type UserFormAddProps = {
	mode: "add";
	email: string;
	password: string;
	orgId: string;
	role: string;
	organizations: Organization[];
	onEmailChange: (value: string) => void;
	onPasswordChange: (value: string) => void;
	onOrgIdChange: (value: string) => void;
	onRoleChange: (value: string) => void;
	disabled?: boolean;
};

type UserFormEditProps = {
	mode: "edit";
	email: string;
	active: boolean;
	onEmailChange: (value: string) => void;
	onActiveChange: (value: boolean) => void;
	disabled?: boolean;
};

type UserFormProps = UserFormAddProps | UserFormEditProps;

function UserForm(props: UserFormProps) {
	const sx = {
		display: "flex",
		flexDirection: "column" as const,
		gap: 2,
		pt: 1,
		minWidth: 320,
	};
	if (props.mode === "add") {
		return (
			<Box sx={sx}>
				<TextField
					label="Email"
					type="email"
					fullWidth
					value={props.email}
					onChange={(e) => props.onEmailChange(e.target.value)}
					required
					disabled={props.disabled}
				/>
				<TextField
					label="Password"
					type="password"
					fullWidth
					value={props.password}
					onChange={(e) => props.onPasswordChange(e.target.value)}
					placeholder="Min 8 characters"
					disabled={props.disabled}
				/>
				<FormControl fullWidth size="small" disabled={props.disabled}>
					<InputLabel>Organization</InputLabel>
					<Select
						label="Organization"
						value={props.orgId}
						onChange={(e) => props.onOrgIdChange(e.target.value)}
					>
						{props.organizations.map((org) => (
							<MenuItem key={org.id} value={org.id}>
								{org.name}
							</MenuItem>
						))}
					</Select>
				</FormControl>
				<FormControl fullWidth size="small" disabled={props.disabled}>
					<InputLabel>Role</InputLabel>
					<Select
						label="Role"
						value={props.role}
						onChange={(e) => props.onRoleChange(e.target.value)}
					>
						{ROLES.map((r) => (
							<MenuItem key={r} value={r}>
								{r}
							</MenuItem>
						))}
					</Select>
				</FormControl>
			</Box>
		);
	}
	return (
		<Box sx={sx}>
			<TextField
				label="Email"
				type="email"
				fullWidth
				value={props.email}
				onChange={(e) => props.onEmailChange(e.target.value)}
				disabled={props.disabled}
			/>
			<FormControlLabel
				control={
					<Checkbox
						checked={props.active}
						onChange={(e) => props.onActiveChange(e.target.checked)}
						disabled={props.disabled}
					/>
				}
				label="Active"
			/>
		</Box>
	);
}

function formatDate(iso: string) {
	try {
		return new Date(iso).toLocaleString();
	} catch {
		return iso;
	}
}

function membershipsSummary(memberships: Membership[]) {
	if (!memberships.length) return "—";
	return memberships.map((m) => `${m.org_name} (${m.role})`).join(", ");
}

export default function UsersPage() {
	const { permissions } = useSession();
	const canRead = permissions.includes(USERS_READ);
	const canWrite = permissions.includes(USERS_WRITE);

	const [users, setUsers] = useState<User[]>([]);
	const [organizations, setOrganizations] = useState<Organization[]>([]);
	const [loading, setLoading] = useState(true);
	const [error, setError] = useState<string | null>(null);
	const [addDialogOpen, setAddDialogOpen] = useState(false);
	const [addEmail, setAddEmail] = useState("");
	const [addPassword, setAddPassword] = useState("");
	const [addOrgId, setAddOrgId] = useState("");
	const [addRole, setAddRole] = useState("editor");
	const [adding, setAdding] = useState(false);
	const [filterEmail, setFilterEmail] = useState("");
	const [filterOrgId, setFilterOrgId] = useState("");
	const [filterRole, setFilterRole] = useState("");
	const [filterActive, setFilterActive] = useState<"" | "yes" | "no">("");
	const [filterAdmin, setFilterAdmin] = useState<"" | "yes" | "no">("");
	const [editUser, setEditUser] = useState<User | null>(null);
	const [editEmail, setEditEmail] = useState("");
	const [editActive, setEditActive] = useState(true);
	const [saving, setSaving] = useState(false);
	const [deletingId, setDeletingId] = useState<string | null>(null);

	const fetchUsers = useCallback(async () => {
		if (!canRead) {
			setLoading(false);
			setUsers([]);
			return;
		}
		setError(null);
		try {
			const res = await fetch("/api/dashboard/users", {
				credentials: "include",
			});
			if (res.status === 403) {
				setError("You do not have permission to view users.");
				setUsers([]);
				return;
			}
			if (res.status === 401) {
				setError("Session expired or not logged in. Please log in again.");
				setUsers([]);
				return;
			}
			if (!res.ok) {
				const text = await res.text();
				let msg: string;
				try {
					const json = JSON.parse(text) as { error?: string };
					msg = (json.error ?? text) || "Failed to load users.";
				} catch {
					msg = text || "Failed to load users.";
				}
				setError(msg);
				setUsers([]);
				return;
			}
			const data = await res.json();
			setUsers(data.users ?? []);
		} catch {
			setError("Failed to load users (network or server error).");
			setUsers([]);
		} finally {
			setLoading(false);
		}
	}, [canRead]);

	const fetchOrgs = useCallback(async () => {
		if (!canRead) return;
		try {
			const res = await fetch("/api/dashboard/organizations", {
				credentials: "include",
			});
			if (res.ok) {
				const data = await res.json();
				const orgs = data.organizations ?? [];
				setOrganizations(orgs);
				setAddOrgId((prev) =>
					prev && orgs.some((o: Organization) => o.id === prev)
						? prev
						: orgs[0]?.id ?? "",
				);
			}
		} catch {
			// optional for filters and add form
		}
	}, [canRead]);

	useEffect(() => {
		fetchUsers();
	}, [fetchUsers]);

	useEffect(() => {
		fetchOrgs();
	}, [fetchOrgs]);

	const handleAdd = async () => {
		const email = addEmail.trim();
		if (!email) return;
		if (addPassword.length < 8) {
			setError("Password must be at least 8 characters.");
			return;
		}
		if (!addOrgId) {
			setError("Select an organization.");
			return;
		}
		setAdding(true);
		setError(null);
		try {
			const res = await fetch("/api/dashboard/users", {
				method: "POST",
				headers: { "Content-Type": "application/json" },
				credentials: "include",
				body: JSON.stringify({
					email,
					password: addPassword,
					org_id: addOrgId,
					role: addRole,
				}),
			});
			if (res.status === 403) {
				setError("You do not have permission to create users.");
				return;
			}
			if (!res.ok) {
				const data = await res.json().catch(() => ({}));
				setError(data.error ?? "Failed to create user.");
				return;
			}
			setAddEmail("");
			setAddPassword("");
			setAddDialogOpen(false);
			await fetchUsers();
		} catch {
			setError("Failed to create user.");
		} finally {
			setAdding(false);
		}
	};

	const openEdit = (user: User) => {
		setEditUser(user);
		setEditEmail(user.email);
		setEditActive(user.is_active);
	};

	const handleSaveEdit = async () => {
		if (!editUser) return;
		setSaving(true);
		setError(null);
		try {
			const res = await fetch("/api/dashboard/users", {
				method: "PATCH",
				headers: { "Content-Type": "application/json" },
				credentials: "include",
				body: JSON.stringify({
					id: editUser.id,
					email: editEmail.trim() || undefined,
					is_active: editActive,
				}),
			});
			if (res.status === 403) {
				setError("You do not have permission to update users.");
				return;
			}
			if (!res.ok) {
				const data = await res.json().catch(() => ({}));
				setError(data.error ?? "Failed to update user.");
				return;
			}
			setEditUser(null);
			await fetchUsers();
		} catch {
			setError("Failed to update user.");
		} finally {
			setSaving(false);
		}
	};

	const handleDelete = async (id: string) => {
		setDeletingId(id);
		setError(null);
		try {
			const res = await fetch("/api/dashboard/users", {
				method: "DELETE",
				headers: { "Content-Type": "application/json" },
				credentials: "include",
				body: JSON.stringify({ id }),
			});
			if (res.status === 403) {
				setError("You do not have permission to delete users.");
				return;
			}
			if (res.ok) {
				await fetchUsers();
			} else {
				const data = await res.json().catch(() => ({}));
				setError(data.error ?? "Failed to delete user.");
			}
		} catch {
			setError("Failed to delete user.");
		} finally {
			setDeletingId(null);
		}
	};

	const filteredUsers = users.filter((user) => {
		const emailMatch =
			!filterEmail.trim() ||
			user.email.toLowerCase().includes(filterEmail.trim().toLowerCase());
		const orgMatch =
			!filterOrgId ||
			user.memberships.some((m) => m.org_id === filterOrgId);
		const roleMatch =
			!filterRole ||
			user.memberships.some(
				(m) => m.role.toLowerCase() === filterRole.toLowerCase(),
			);
		const activeMatch =
			filterActive === "" ||
			(filterActive === "yes" && user.is_active) ||
			(filterActive === "no" && !user.is_active);
		const adminMatch =
			filterAdmin === "" ||
			(filterAdmin === "yes" && user.is_admin) ||
			(filterAdmin === "no" && !user.is_admin);
		return emailMatch && orgMatch && roleMatch && activeMatch && adminMatch;
	});

	if (!canRead) {
		return (
			<>
				<Typography variant="h4" component="h1" gutterBottom>
					Users
				</Typography>
				<Alert severity="info">You do not have permission to view users.</Alert>
			</>
		);
	}

	return (
		<>
			<Typography variant="h4" component="h1" gutterBottom>
				Users
			</Typography>
			<Typography variant="body2" color="text.secondary" sx={{ mb: 2 }}>
				View and manage users. Write actions require <code>dashboard.users.write</code>.
			</Typography>

			{error && (
				<Alert severity="error" sx={{ mb: 2 }} onClose={() => setError(null)}>
					{error}
				</Alert>
			)}

			<Paper sx={{ p: 2, mb: 2 }}>
				<Typography variant="subtitle2" gutterBottom>
					Filters
				</Typography>
				<Box sx={{ display: "flex", flexWrap: "wrap", gap: 2, alignItems: "flex-end" }}>
					<TextField
						label="Email"
						type="search"
						size="small"
						value={filterEmail}
						onChange={(e) => setFilterEmail(e.target.value)}
						placeholder="Search by email"
						sx={{ minWidth: 220 }}
					/>
					<FormControl size="small" sx={{ minWidth: 180 }}>
						<InputLabel>Organization</InputLabel>
						<Select
							label="Organization"
							value={filterOrgId}
							onChange={(e) => setFilterOrgId(e.target.value)}
						>
							<MenuItem value="">All</MenuItem>
							{organizations.map((org) => (
								<MenuItem key={org.id} value={org.id}>
									{org.name}
								</MenuItem>
							))}
						</Select>
					</FormControl>
					<FormControl size="small" sx={{ minWidth: 120 }}>
						<InputLabel>Role</InputLabel>
						<Select
							label="Role"
							value={filterRole}
							onChange={(e) => setFilterRole(e.target.value)}
						>
							<MenuItem value="">All</MenuItem>
							{ROLES.map((r) => (
								<MenuItem key={r} value={r}>
									{r}
								</MenuItem>
							))}
						</Select>
					</FormControl>
					<FormControl size="small" sx={{ minWidth: 100 }}>
						<InputLabel>Active</InputLabel>
						<Select
							label="Active"
							value={filterActive}
							onChange={(e) =>
								setFilterActive(e.target.value as "" | "yes" | "no")
							}
						>
							<MenuItem value="">All</MenuItem>
							<MenuItem value="yes">Yes</MenuItem>
							<MenuItem value="no">No</MenuItem>
						</Select>
					</FormControl>
					<FormControl size="small" sx={{ minWidth: 100 }}>
						<InputLabel>Admin</InputLabel>
						<Select
							label="Admin"
							value={filterAdmin}
							onChange={(e) =>
								setFilterAdmin(e.target.value as "" | "yes" | "no")
							}
						>
							<MenuItem value="">All</MenuItem>
							<MenuItem value="yes">Yes</MenuItem>
							<MenuItem value="no">No</MenuItem>
						</Select>
					</FormControl>
				</Box>
			</Paper>

			{canWrite && (
				<Box sx={{ mb: 2 }}>
					<Button
						variant="contained"
						startIcon={<AddIcon />}
						onClick={() => setAddDialogOpen(true)}
					>
						Add user
					</Button>
				</Box>
			)}

			{loading ? (
				<Box sx={{ display: "flex", justifyContent: "center", py: 4 }}>
					<CircularProgress />
				</Box>
			) : (
				<TableContainer component={Paper}>
					<Table size="small" aria-label="Users">
						<TableHead>
							<TableRow>
								<TableCell>Email</TableCell>
								<TableCell>Organizations / Role</TableCell>
								<TableCell>Active</TableCell>
								<TableCell>Admin</TableCell>
								<TableCell>Created</TableCell>
								{canWrite && <TableCell align="right">Actions</TableCell>}
							</TableRow>
						</TableHead>
						<TableBody>
							{filteredUsers.length === 0 ? (
								<TableRow>
									<TableCell
										colSpan={canWrite ? 6 : 5}
										align="center"
									>
										{users.length === 0
											? "No users."
											: "No users match the filters."}
									</TableCell>
								</TableRow>
							) : (
								filteredUsers.map((user) => (
									<TableRow key={user.id}>
										<TableCell sx={{ fontWeight: 500 }}>{user.email}</TableCell>
										<TableCell sx={{ maxWidth: 280 }}>
											{membershipsSummary(user.memberships)}
										</TableCell>
										<TableCell>{user.is_active ? "Yes" : "No"}</TableCell>
										<TableCell>{user.is_admin ? "Yes" : "No"}</TableCell>
										<TableCell sx={{ whiteSpace: "nowrap" }}>
											{formatDate(user.created_at)}
										</TableCell>
										{canWrite && (
											<TableCell align="right">
												<IconButton
													size="small"
													aria-label="Edit"
													onClick={() => openEdit(user)}
												>
													<EditIcon />
												</IconButton>
												<IconButton
													size="small"
													aria-label="Delete"
													onClick={() => handleDelete(user.id)}
													disabled={deletingId === user.id}
												>
													<DeleteIcon />
												</IconButton>
											</TableCell>
										)}
									</TableRow>
								))
							)}
						</TableBody>
					</Table>
				</TableContainer>
			)}

			<Dialog
				open={addDialogOpen}
				onClose={() => !adding && setAddDialogOpen(false)}
				maxWidth="sm"
				fullWidth
			>
				<DialogTitle>Add user</DialogTitle>
				<DialogContent>
					<UserForm
						mode="add"
						email={addEmail}
						password={addPassword}
						orgId={addOrgId}
						role={addRole}
						organizations={organizations}
						onEmailChange={setAddEmail}
						onPasswordChange={setAddPassword}
						onOrgIdChange={setAddOrgId}
						onRoleChange={setAddRole}
						disabled={adding}
					/>
				</DialogContent>
				<DialogActions>
					<Button
						onClick={() => setAddDialogOpen(false)}
						disabled={adding}
					>
						Cancel
					</Button>
					<Button
						variant="contained"
						onClick={handleAdd}
						disabled={
							adding ||
							!addEmail.trim() ||
							addPassword.length < 8 ||
							!addOrgId
						}
					>
						{adding ? "Adding…" : "Add"}
					</Button>
				</DialogActions>
			</Dialog>

			<Dialog open={Boolean(editUser)} onClose={() => setEditUser(null)}>
				<DialogTitle>Edit user</DialogTitle>
				<DialogContent>
					<UserForm
						mode="edit"
						email={editEmail}
						active={editActive}
						onEmailChange={setEditEmail}
						onActiveChange={setEditActive}
						disabled={saving}
					/>
				</DialogContent>
				<DialogActions>
					<Button onClick={() => setEditUser(null)}>Cancel</Button>
					<Button variant="contained" onClick={handleSaveEdit} disabled={saving}>
						{saving ? "Saving…" : "Save"}
					</Button>
				</DialogActions>
			</Dialog>
		</>
	);
}
