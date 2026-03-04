import { useState } from "react";
import { Navigate, useNavigate } from "react-router-dom";
import Box from "@mui/material/Box";
import Card from "@mui/material/Card";
import CardActionArea from "@mui/material/CardActionArea";
import CardContent from "@mui/material/CardContent";
import Typography from "@mui/material/Typography";
import { useSession } from "../../../../context/Session";

export default function SelectProfilePage() {
	const navigate = useNavigate();
	const { user, profiles, needs_profile_select, loading, refresh } =
		useSession();
	const [submitting, setSubmitting] = useState<string | null>(null);
	const [error, setError] = useState<string | null>(null);

	// Must be logged in
	if (!loading && user == null) {
		return (
			<Navigate
				to="/login"
				replace
				state={{ from: { pathname: "/dashboard" } }}
			/>
		);
	}
	// Already have a profile, go to dashboard
	if (!loading && user != null && !needs_profile_select) {
		return <Navigate to="/dashboard" replace />;
	}

	async function handleSelectProfile(orgId: string, roleId?: string) {
		setError(null);
		setSubmitting(orgId);
		try {
			const res = await fetch("/api/auth/set-profile", {
				method: "POST",
				headers: { "Content-Type": "application/json" },
				credentials: "include",
				body: JSON.stringify({ org_id: orgId, role_id: roleId ?? undefined }),
			});
			if (!res.ok) {
				const data = await res.json().catch(() => ({}));
				setError(
					typeof data?.error === "string"
						? data.error
						: "Failed to set profile",
				);
				return;
			}
			await refresh();
			navigate("/dashboard", { replace: true });
		} catch {
			setError("Something went wrong. Please try again.");
		} finally {
			setSubmitting(null);
		}
	}

	if (loading || profiles.length === 0) {
		return (
			<Box sx={{ p: 3 }}>
				<Typography variant="h5" gutterBottom>
					Select profile
				</Typography>
				<Typography color="text.secondary">
					{loading ? "Loading…" : "No profiles available."}
				</Typography>
			</Box>
		);
	}

	return (
		<>
			<Typography variant="h4" component="h1" gutterBottom>
				Select profile
			</Typography>
			<Typography variant="body1" color="text.secondary" sx={{ mb: 2 }}>
				Choose the organization and role to use for this session.
			</Typography>
			{error != null && (
				<Typography
					color="error"
					sx={{ mb: 2 }}
					data-testid="profile-select-error"
				>
					{error}
				</Typography>
			)}
			<Box sx={{ display: "flex", flexDirection: "column", gap: 1.5 }}>
				{profiles.map((p) => (
					<Card key={p.org_id + (p.role_id ?? p.role)} variant="outlined">
						<CardActionArea
							onClick={() => handleSelectProfile(p.org_id, p.role_id)}
							disabled={submitting != null}
							data-testid={`profile-${p.org_name}-${p.role}`}
						>
							<CardContent>
								<Typography variant="subtitle1">
									{p.org_name} · {(p.role ?? "").charAt(0).toUpperCase()}
									{(p.role ?? "").slice(1)}
								</Typography>
							</CardContent>
						</CardActionArea>
					</Card>
				))}
			</Box>
		</>
	);
}
