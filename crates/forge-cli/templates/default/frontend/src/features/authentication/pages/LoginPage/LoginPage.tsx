import { useState } from "react";
import { Link as RouterLink, useNavigate, useLocation } from "react-router-dom";
import Alert from "@mui/material/Alert";
import Box from "@mui/material/Box";
import Button from "@mui/material/Button";
import Link from "@mui/material/Link";
import TextField from "@mui/material/TextField";
import Typography from "@mui/material/Typography";
import { useSession } from "../../../../context/Session";

export default function LoginPage() {
	const navigate = useNavigate();
	const location = useLocation();
	const { refresh } = useSession();
	const [email, setEmail] = useState("");
	const [password, setPassword] = useState("");
	const [error, setError] = useState<string | null>(null);
	const [submitting, setSubmitting] = useState(false);

	const from =
		(location.state as { from?: { pathname: string } } | null)?.from
			?.pathname ?? "/dashboard";

	async function handleSubmit(e: React.FormEvent) {
		e.preventDefault();
		setSubmitting(true);
		try {
			const res = await fetch("/api/auth/login", {
				method: "POST",
				headers: { "Content-Type": "application/json" },
				credentials: "include",
				body: JSON.stringify({ email, password }),
			});
			const data = await res.json().catch(() => ({}));
			if (res.ok) {
				await refresh();
				navigate(from, { replace: true });
				return;
			}
			setError(
				typeof data?.error === "string"
					? data.error
					: "Invalid email or password",
			);
		} catch {
			setError("Something went wrong. Please try again.");
		} finally {
			setSubmitting(false);
		}
	}

	return (
		<>
			<Typography variant="h4" component="h1" gutterBottom>
				Log in
			</Typography>
			<Box sx={{ display: "flex", flexDirection: "column", gap: 2 }}>
				<form onSubmit={handleSubmit} data-testid="login-form">
					<Box sx={{ display: "flex", flexDirection: "column", gap: 2 }}>
						{error != null && (
							<Alert severity="error" data-testid="login-error">
								<Typography variant="subtitle2">Login failed</Typography>
								{error}
							</Alert>
						)}
						<TextField
							name="email"
							type="email"
							label="Email"
							placeholder="you@example.com"
							value={email}
							onChange={(e) => setEmail(e.target.value)}
							required
							disabled={submitting}
							data-testid="login-email"
							fullWidth
						/>
						<TextField
							name="password"
							type="password"
							label="Password"
							placeholder="••••••••"
							value={password}
							onChange={(e) => setPassword(e.target.value)}
							required
							disabled={submitting}
							data-testid="login-password"
							fullWidth
						/>
						<Box sx={{ display: "flex", justifyContent: "flex-end" }}>
							<Button
								type="submit"
								variant="contained"
								disabled={submitting}
								data-testid="login-submit"
							>
								Log in
							</Button>
						</Box>
					</Box>
				</form>
				<Typography variant="body2" textAlign="center">
					<Link component={RouterLink} to="/register" variant="body2">
						Don&apos;t have an account? Create an Account
					</Link>
				</Typography>
			</Box>
		</>
	);
}
