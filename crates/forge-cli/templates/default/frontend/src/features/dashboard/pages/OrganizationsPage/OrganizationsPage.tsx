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
import useTheme from "@mui/material/styles/useTheme";
import FormDialog from "../../../../components/FormDialog";
import useMediaQuery from "@mui/material/useMediaQuery";
import { useSession } from "../../../../context/Session";

type Organization = {
	id: string;
	name: string;
	slug: string;
	created_at: string;
	updated_at: string;
};

const ORG_WRITE = "dashboard.organizations.write";

type OrganizationFormProps = {
	mode: "add" | "edit";
	name: string;
	slug: string;
	onNameChange: (value: string) => void;
	onSlugChange: (value: string) => void;
	disabled?: boolean;
};

function OrganizationForm({
	mode,
	name,
	slug,
	onNameChange,
	onSlugChange,
	disabled = false,
}: OrganizationFormProps) {
	const slugLabel = mode === "add" ? "Slug (optional)" : "Slug";
	const slugPlaceholder = mode === "add" ? "Auto from name if blank" : undefined;
	return (
		<Box sx={{ display: "flex", flexDirection: "column", gap: 2, pt: 1, minWidth: 0 }}>
			<TextField
				label="Name"
				size="small"
				fullWidth
				value={name}
				onChange={(e) => onNameChange(e.target.value)}
				required
				disabled={disabled}
			/>
			<TextField
				label={slugLabel}
				size="small"
				fullWidth
				value={slug}
				onChange={(e) => onSlugChange(e.target.value)}
				placeholder={slugPlaceholder}
				disabled={disabled}
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

export default function OrganizationsPage() {
	const theme = useTheme();
	const isMobile = useMediaQuery(theme.breakpoints.down("md"));
	const { permissions } = useSession();
	const canWrite = permissions.includes(ORG_WRITE);

	const [organizations, setOrganizations] = useState<Organization[]>([]);
	const [loading, setLoading] = useState(true);
	const [error, setError] = useState<string | null>(null);
	const [addDialogOpen, setAddDialogOpen] = useState(false);
	const [addName, setAddName] = useState("");
	const [addSlug, setAddSlug] = useState("");
	const [adding, setAdding] = useState(false);
	const [filterName, setFilterName] = useState("");
	const [filterSlug, setFilterSlug] = useState("");
	const [editOrg, setEditOrg] = useState<Organization | null>(null);
	const [editName, setEditName] = useState("");
	const [editSlug, setEditSlug] = useState("");
	const [saving, setSaving] = useState(false);
	const [deletingId, setDeletingId] = useState<string | null>(null);

	const fetchData = useCallback(async () => {
		setError(null);
		try {
			const res = await fetch("/api/dashboard/organizations", {
				credentials: "include",
			});
			if (res.status === 403) {
				setError("You do not have permission to view organizations.");
				setOrganizations([]);
				return;
			}
			if (!res.ok) {
				setError("Failed to load organizations.");
				setOrganizations([]);
				return;
			}
			const data = await res.json();
			setOrganizations(data.organizations ?? []);
		} catch {
			setError("Failed to load organizations.");
			setOrganizations([]);
		} finally {
			setLoading(false);
		}
	}, []);

	useEffect(() => {
		fetchData();
	}, [fetchData]);

	const handleAdd = async () => {
		const name = addName.trim();
		if (!name) return;
		setAdding(true);
		setError(null);
		try {
			const res = await fetch("/api/dashboard/organizations", {
				method: "POST",
				headers: { "Content-Type": "application/json" },
				credentials: "include",
				body: JSON.stringify({
					name,
					slug: addSlug.trim() || undefined,
				}),
			});
			if (res.status === 403) {
				setError("You do not have permission to create organizations.");
				return;
			}
			if (!res.ok) {
				const data = await res.json().catch(() => ({}));
				setError(data.error ?? "Failed to create organization.");
				return;
			}
			setAddName("");
			setAddSlug("");
			setAddDialogOpen(false);
			await fetchData();
		} catch {
			setError("Failed to create organization.");
		} finally {
			setAdding(false);
		}
	};

	const openEdit = (org: Organization) => {
		setEditOrg(org);
		setEditName(org.name);
		setEditSlug(org.slug);
	};

	const handleSaveEdit = async () => {
		if (!editOrg) return;
		setSaving(true);
		setError(null);
		try {
			const res = await fetch("/api/dashboard/organizations", {
				method: "PATCH",
				headers: { "Content-Type": "application/json" },
				credentials: "include",
				body: JSON.stringify({
					id: editOrg.id,
					name: editName.trim() || undefined,
					slug: editSlug.trim() || undefined,
				}),
			});
			if (res.status === 403) {
				setError("You do not have permission to update organizations.");
				return;
			}
			if (!res.ok) {
				const data = await res.json().catch(() => ({}));
				setError(data.error ?? "Failed to update organization.");
				return;
			}
			setEditOrg(null);
			await fetchData();
		} catch {
			setError("Failed to update organization.");
		} finally {
			setSaving(false);
		}
	};

	const handleDelete = async (id: string) => {
		setDeletingId(id);
		setError(null);
		try {
			const res = await fetch("/api/dashboard/organizations", {
				method: "DELETE",
				headers: { "Content-Type": "application/json" },
				credentials: "include",
				body: JSON.stringify({ id }),
			});
			if (res.status === 403) {
				setError("You do not have permission to delete organizations.");
				return;
			}
			if (res.ok) {
				await fetchData();
			} else {
				const data = await res.json().catch(() => ({}));
				setError(data.error ?? "Failed to delete organization.");
			}
		} catch {
			setError("Failed to delete organization.");
		} finally {
			setDeletingId(null);
		}
	};

	const filteredOrganizations = organizations.filter((org) => {
		const nameMatch =
			!filterName.trim() ||
			org.name.toLowerCase().includes(filterName.trim().toLowerCase());
		const slugMatch =
			!filterSlug.trim() ||
			org.slug.toLowerCase().includes(filterSlug.trim().toLowerCase());
		return nameMatch && slugMatch;
	});

	return (
		<>
			<Typography variant="h4" component="h1" gutterBottom>
				Organizations
			</Typography>
			<Typography variant="body2" color="text.secondary" sx={{ mb: 2 }}>
				View and manage organizations. Write actions require{" "}
				<code>dashboard.organizations.write</code>.
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
						label="Name"
						size="small"
						value={filterName}
						onChange={(e) => setFilterName(e.target.value)}
						placeholder="Search by name"
						sx={{ minWidth: 200 }}
					/>
					<TextField
						label="Slug"
						size="small"
						value={filterSlug}
						onChange={(e) => setFilterSlug(e.target.value)}
						placeholder="Search by slug"
						sx={{ minWidth: 160 }}
					/>
				</Box>
			</Paper>

			{canWrite && (
				<Box sx={{ mb: 2 }}>
					<Button
						variant="contained"
						startIcon={<AddIcon />}
						onClick={() => setAddDialogOpen(true)}
					>
						Add organization
					</Button>
				</Box>
			)}

			{loading ? (
				<Box sx={{ display: "flex", justifyContent: "center", py: 4 }}>
					<CircularProgress />
				</Box>
			) : filteredOrganizations.length === 0 ? (
				<Paper sx={{ p: 3, textAlign: "center" }}>
					<Typography color="text.secondary">
						{organizations.length === 0
							? "No organizations."
							: "No organizations match the filters."}
					</Typography>
				</Paper>
			) : isMobile ? (
				<Box component="ul" sx={{ listStyle: "none", m: 0, p: 0, display: "flex", flexDirection: "column", gap: 1.5 }}>
					{filteredOrganizations.map((org) => (
						<Box key={org.id} component="li">
							<Paper sx={{ p: 2 }}>
								<Typography variant="subtitle1" fontWeight={600}>
									{org.name}
								</Typography>
								<Typography variant="body2" color="text.secondary">
									{org.slug}
								</Typography>
								<Typography variant="caption" color="text.secondary" display="block" sx={{ mt: 0.5 }}>
									Created {formatDate(org.created_at)} · Updated {formatDate(org.updated_at)}
								</Typography>
								{canWrite && (
									<Box sx={{ mt: 2, display: "flex", gap: 0.5, justifyContent: "flex-end" }}>
										<Button size="small" startIcon={<EditIcon />} onClick={() => openEdit(org)}>
											Edit
										</Button>
										<Button
											size="small"
											color="error"
											startIcon={<DeleteIcon />}
											onClick={() => handleDelete(org.id)}
											disabled={deletingId === org.id}
										>
											Delete
										</Button>
									</Box>
								)}
							</Paper>
						</Box>
					))}
				</Box>
			) : (
				<TableContainer component={Paper}>
					<Table size="small" aria-label="Organizations">
						<TableHead>
							<TableRow>
								<TableCell>Name</TableCell>
								<TableCell>Slug</TableCell>
								<TableCell>Created</TableCell>
								<TableCell>Updated</TableCell>
								{canWrite && <TableCell align="right">Actions</TableCell>}
							</TableRow>
						</TableHead>
						<TableBody>
							{filteredOrganizations.map((org) => (
								<TableRow key={org.id}>
									<TableCell sx={{ fontWeight: 500 }}>{org.name}</TableCell>
									<TableCell>{org.slug}</TableCell>
									<TableCell sx={{ whiteSpace: "nowrap" }}>{formatDate(org.created_at)}</TableCell>
									<TableCell sx={{ whiteSpace: "nowrap" }}>{formatDate(org.updated_at)}</TableCell>
									{canWrite && (
										<TableCell align="right">
											<IconButton size="small" aria-label="Edit" onClick={() => openEdit(org)}>
												<EditIcon />
											</IconButton>
											<IconButton
												size="small"
												aria-label="Delete"
												onClick={() => handleDelete(org.id)}
												disabled={deletingId === org.id}
											>
												<DeleteIcon />
											</IconButton>
										</TableCell>
									)}
								</TableRow>
							))}
						</TableBody>
					</Table>
				</TableContainer>
			)}

			<FormDialog
				open={addDialogOpen}
				onClose={() => setAddDialogOpen(false)}
				title="Add organization"
				submitLabel="Add"
				submittingLabel="Adding…"
				onSubmit={handleAdd}
				submitDisabled={!addName.trim()}
				submitting={adding}
				contentSx={{ minWidth: 0 }}
			>
				<OrganizationForm
					mode="add"
					name={addName}
					slug={addSlug}
					onNameChange={setAddName}
					onSlugChange={setAddSlug}
					disabled={adding}
				/>
			</FormDialog>

			<FormDialog
				open={Boolean(editOrg)}
				onClose={() => setEditOrg(null)}
				title="Edit organization"
				submitLabel="Save"
				submittingLabel="Saving…"
				onSubmit={handleSaveEdit}
				submitDisabled={false}
				submitting={saving}
				contentSx={{ minWidth: 0 }}
			>
				<OrganizationForm
					mode="edit"
					name={editName}
					slug={editSlug}
					onNameChange={setEditName}
					onSlugChange={setEditSlug}
					disabled={saving}
				/>
			</FormDialog>
		</>
	);
}
