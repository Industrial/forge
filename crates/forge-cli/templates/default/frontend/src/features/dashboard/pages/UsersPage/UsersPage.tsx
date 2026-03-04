import { useCallback, useEffect, useState } from "react";
import { useForm, Controller } from "react-hook-form";
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
import FormControlLabel from "@mui/material/FormControlLabel";
import FormDialog from "../../../../components/FormDialog";
import Checkbox from "@mui/material/Checkbox";
import FormControl from "@mui/material/FormControl";
import InputLabel from "@mui/material/InputLabel";
import Select from "@mui/material/Select";
import MenuItem from "@mui/material/MenuItem";
import Chip from "@mui/material/Chip";
import { useLiveUpdates } from "../../../../context/LiveWs";
import { useAuth } from "../../../../context/Auth";
import { Schema } from "effect";
import { runPromise } from "../../../../lib/runEffect";
import { effectSchemaResolver } from "../../../../lib/effectSchemaResolver";
import {
	userAddFormSchemaStrict,
	userEditFormSchema,
	type UserAddFormValuesStrict,
	type UserEditFormValues,
} from "../../../../schemas/userFormSchemas";
import { fetchUsersEffect, type User } from "../../../../effects/users";

type Organization = {
	id: string;
	name: string;
	slug: string;
};

const USERS_READ = "dashboard.users.read";
const USERS_WRITE = "dashboard.users.write";
const FILTER_ROLES = ["owner", "admin", "editor", "viewer"];

type OrgRole = {
	id: string;
	name: string;
	display_name: string | null;
};

function formatDate(iso: string) {
	try {
		return new Date(iso).toLocaleString();
	} catch {
		return iso;
	}
}

function membershipsSummary(memberships: User["memberships"]) {
	if (!memberships.length) return "—";
	return memberships
		.map((m) => `${m.org_name}: ${(m.roles ?? []).join(", ") || "—"}`)
		.join("; ");
}

export default function UsersPage() {
	const { permissions } = useAuth();
	const { api } = useOutletContext<{ api: ReturnType<typeof useApi> }>();
	const canRead = permissions.includes(USERS_READ);
	const canWrite = permissions.includes(USERS_WRITE);
	const [liveRefreshTrigger, setLiveRefreshTrigger] = useState(0);
	const { connected: wsConnected } = useLiveUpdates("users", () => {
		setLiveRefreshTrigger((n) => n + 1);
	});
	const fetchUsers = useCallback(async () => {
		if (!canRead) {
			setLoading(false);
			setUsers([]);
			return;
		}
		setError(null);
		try {
			const list = await runPromise(fetchUsersEffect(api));
			setUsers(list);
		} catch (e) {
			setError(e instanceof Error ? e.message : "Failed to load users.");
			setUsers([]);
		} finally {
			setLoading(false);
		}
	}, [canRead]);

	const [users, setUsers] = useState<User[]>([]);
	const [organizations, setOrganizations] = useState<Organization[]>([]);
	const [loading, setLoading] = useState(true);
	const [error, setError] = useState<string | null>(null);
	const [addDialogOpen, setAddDialogOpen] = useState(false);
	const [orgRoles, setOrgRoles] = useState<OrgRole[]>([]);
	const [adding, setAdding] = useState(false);
	const [filterEmail, setFilterEmail] = useState("");
	const [filterOrgId, setFilterOrgId] = useState("");
	const [filterRole, setFilterRole] = useState("");
	const [filterActive, setFilterActive] = useState<"" | "yes" | "no">("");
	const [filterAdmin, setFilterAdmin] = useState<"" | "yes" | "no">("");
	const [editUser, setEditUser] = useState<User | null>(null);
	const [saving, setSaving] = useState(false);
	const [deletingId, setDeletingId] = useState<string | null>(null);

	const addForm = useForm<UserAddFormValuesStrict>({
		resolver: effectSchemaResolver(
			userAddFormSchemaStrict as Schema.Schema<
				UserAddFormValuesStrict,
				unknown,
				never
			>,
		),
		defaultValues: { email: "", password: "", orgId: "", roleIds: [] },
		mode: "onChange",
	});

	const editForm = useForm<UserEditFormValues>({
		resolver: effectSchemaResolver(
			userEditFormSchema as Schema.Schema<UserEditFormValues, unknown, never>,
		),
		defaultValues: { email: "", active: true },
		mode: "onChange",
	});

	const addFormOrgId = addForm.watch("orgId");

	const fetchOrgs = useCallback(async () => {
		if (!canRead) return;
		try {
			const res = await api("/api/dashboard/organizations", {});
			if (res.ok) {
				const data = await res.json();
				setOrganizations(data.organizations ?? []);
			}
		} catch {
			// optional for filters and add form
		}
	}, [canRead]);

	useEffect(() => {
		fetchUsers();
	}, [fetchUsers, liveRefreshTrigger]);

	useEffect(() => {
		fetchOrgs();
	}, [fetchOrgs]);

	const fetchOrgRoles = useCallback(async (orgId: string) => {
		if (!orgId) {
			setOrgRoles([]);
			return;
		}
		try {
			const res = await api(
				`/api/dashboard/roles?org_id=${encodeURIComponent(orgId)}`,
				{},
			);
			if (res.ok) {
				const data = await res.json();
				setOrgRoles(data.roles ?? []);
			} else {
				setOrgRoles([]);
			}
		} catch {
			setOrgRoles([]);
		}
	}, []);

	useEffect(() => {
		fetchOrgRoles(addFormOrgId);
	}, [addFormOrgId, fetchOrgRoles]);

	useEffect(() => {
		const current = addForm.getValues("roleIds");
		const valid = current.filter((id) => orgRoles.some((r) => r.id === id));
		if (valid.length !== current.length) {
			addForm.setValue("roleIds", valid);
		}
	}, [orgRoles, addForm]);

	const handleAdd = async (data: UserAddFormValuesStrict) => {
		setAdding(true);
		setError(null);
		try {
			const res = await api("/api/dashboard/users", {
				method: "POST",
				headers: { "Content-Type": "application/json" },
				body: JSON.stringify({
					email: data.email,
					password: data.password,
					org_id: data.orgId,
					role_ids: data.roleIds,
				}),
			});
			if (res.status === 403) {
				setError("You do not have permission to create users.");
				return;
			}
			if (!res.ok) {
				const resData = await res.json().catch(() => ({}));
				setError(resData.error ?? "Failed to create user.");
				return;
			}
			addForm.reset({ email: "", password: "", orgId: "", roleIds: [] });
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
		editForm.reset({ email: user.email, active: user.is_active });
	};

	const handleSaveEdit = async (data: UserEditFormValues) => {
		if (!editUser) return;
		setSaving(true);
		setError(null);
		try {
			const res = await api("/api/dashboard/users", {
				method: "PATCH",
				headers: { "Content-Type": "application/json" },
				body: JSON.stringify({
					id: editUser.id,
					email: data.email.trim() || undefined,
					is_active: data.active,
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
			const res = await api("/api/dashboard/users", {
				method: "DELETE",
				headers: { "Content-Type": "application/json" },
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
			!filterOrgId || user.memberships.some((m) => m.org_id === filterOrgId);
		const roleMatch =
			!filterRole ||
			user.memberships.some((m) =>
				(m.roles ?? []).some(
					(r) => r.toLowerCase() === filterRole.toLowerCase(),
				),
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
			<Box sx={{ display: "flex", alignItems: "center", gap: 1, mb: 0 }}>
				<Typography variant="h4" component="h1" gutterBottom sx={{ mb: 0 }}>
					Users
				</Typography>
				{wsConnected && <Chip label="Live" color="success" size="small" />}
			</Box>
			<Typography variant="body2" color="text.secondary" sx={{ mb: 2 }}>
				View and manage users. Write actions require{" "}
				<code>dashboard.users.write</code>.
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
				<Box
					sx={{
						display: "flex",
						flexWrap: "wrap",
						gap: 2,
						alignItems: "flex-end",
					}}
				>
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
							{FILTER_ROLES.map((r) => (
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
						onClick={() => {
							addForm.reset({
								email: "",
								password: "",
								orgId: organizations[0]?.id ?? "",
								roleIds: [],
							});
							setAddDialogOpen(true);
						}}
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
									<TableCell colSpan={canWrite ? 6 : 5} align="center">
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

			<FormDialog
				open={addDialogOpen}
				onClose={() => setAddDialogOpen(false)}
				title="Add user"
				submitLabel="Add"
				submittingLabel="Adding…"
				onSubmit={() => addForm.handleSubmit(handleAdd)()}
				submitDisabled={!addForm.formState.isValid}
				submitting={adding}
			>
				<Box
					sx={{
						display: "flex",
						flexDirection: "column",
						gap: 2,
						pt: 1,
						minWidth: 320,
					}}
				>
					<Controller
						control={addForm.control}
						name="email"
						render={({ field, fieldState }) => (
							<TextField
								{...field}
								label="Email"
								type="email"
								required
								disabled={adding}
								error={Boolean(fieldState.error)}
								helperText={fieldState.error?.message}
							/>
						)}
					/>
					<Controller
						control={addForm.control}
						name="password"
						render={({ field, fieldState }) => (
							<TextField
								{...field}
								label="Password"
								type="password"
								placeholder="Min 8 characters"
								disabled={adding}
								error={Boolean(fieldState.error)}
								helperText={fieldState.error?.message}
							/>
						)}
					/>
					<Controller
						control={addForm.control}
						name="orgId"
						render={({ field, fieldState }) => (
							<FormControl
								fullWidth
								size="small"
								disabled={adding}
								error={Boolean(fieldState.error)}
							>
								<InputLabel>Organization</InputLabel>
								<Select
									{...field}
									label="Organization"
									onChange={(e) => field.onChange(e.target.value)}
								>
									{organizations.map((org) => (
										<MenuItem key={org.id} value={org.id}>
											{org.name}
										</MenuItem>
									))}
								</Select>
								{fieldState.error?.message && (
									<Box
										component="span"
										sx={{
											color: "error.main",
											fontSize: "0.75rem",
											mt: 0.5,
											display: "block",
										}}
									>
										{fieldState.error.message}
									</Box>
								)}
							</FormControl>
						)}
					/>
					<Controller
						control={addForm.control}
						name="roleIds"
						render={({ field, fieldState }) => (
							<FormControl
								fullWidth
								size="small"
								disabled={adding}
								error={Boolean(fieldState.error)}
							>
								<InputLabel>Roles</InputLabel>
								<Select
									{...field}
									label="Roles"
									multiple
									onChange={(e) =>
										field.onChange([].slice.call(e.target.value))
									}
									renderValue={(selected) =>
										(selected as string[])
											.map(
												(id) => orgRoles.find((r) => r.id === id)?.name ?? id,
											)
											.join(", ")
									}
								>
									{orgRoles.map((r) => (
										<MenuItem key={r.id} value={r.id}>
											{r.display_name || r.name}
										</MenuItem>
									))}
								</Select>
								{fieldState.error?.message && (
									<Box
										component="span"
										sx={{
											color: "error.main",
											fontSize: "0.75rem",
											mt: 0.5,
											display: "block",
										}}
									>
										{fieldState.error.message}
									</Box>
								)}
							</FormControl>
						)}
					/>
				</Box>
			</FormDialog>

			<FormDialog
				open={Boolean(editUser)}
				onClose={() => setEditUser(null)}
				title="Edit user"
				submitLabel="Save"
				submittingLabel="Saving…"
				onSubmit={() => editForm.handleSubmit(handleSaveEdit)()}
				submitDisabled={!editForm.formState.isValid}
				submitting={saving}
			>
				<Box
					sx={{
						display: "flex",
						flexDirection: "column",
						gap: 2,
						pt: 1,
						minWidth: 320,
					}}
				>
					<Controller
						control={editForm.control}
						name="email"
						render={({ field, fieldState }) => (
							<TextField
								{...field}
								label="Email"
								type="email"
								disabled={saving}
								error={Boolean(fieldState.error)}
								helperText={fieldState.error?.message}
							/>
						)}
					/>
					<Controller
						control={editForm.control}
						name="active"
						render={({ field }) => (
							<FormControlLabel
								control={
									<Checkbox
										checked={field.value}
										onChange={(e) => field.onChange(e.target.checked)}
										disabled={saving}
									/>
								}
								label="Active"
							/>
						)}
					/>
				</Box>
			</FormDialog>
		</>
	);
}
