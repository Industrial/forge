import { useState } from "react";
import { Link as RouterLink, useNavigate } from "react-router-dom";
import Alert from "@mui/material/Alert";
import Box from "@mui/material/Box";
import Button from "@mui/material/Button";
import Link from "@mui/material/Link";
import TextField from "@mui/material/TextField";
import Typography from "@mui/material/Typography";

export default function RegisterPage() {
	const navigate = useNavigate();
	const [email, setEmail] = useState("");
	const [password, setPassword] = useState("");
	const [error, setError] = useState<string | null>(null);
	const [submitting, setSubmitting] = useState(false);

	async function handleSubmit(e: React.FormEvent) {
		e.preventDefault();
		setSubmitting(true);
		try {
			const res = await fetch("/api/auth/register", {
				method: "POST",
				headers: { "Content-Type": "application/json" },
				credentials: "include",
				body: JSON.stringify({ email, password }),
			});
			const data = await res.json().catch(() => ({}));
			if (res.ok) {
				navigate("/login", { replace: true });
				return;
			}
			setError(
				typeof data?.error === "string"
					? data.error
					: "Registration failed. Please try again.",
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
				Create an account
			</Typography>
			<Box sx={{ display: "flex", flexDirection: "column", gap: 2 }}>
				<form onSubmit={handleSubmit} data-testid="register-form">
					<Box sx={{ display: "flex", flexDirection: "column", gap: 2 }}>
						{error != null && (
							<Alert severity="error" data-testid="register-error">
								<Typography variant="subtitle2">Registration failed</Typography>
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
							data-testid="register-email"
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
							inputProps={{ minLength: 8 }}
							disabled={submitting}
							data-testid="register-password"
							fullWidth
						/>
						<Box sx={{ display: "flex", justifyContent: "flex-end" }}>
							<Button
								type="submit"
								variant="contained"
								disabled={submitting}
								data-testid="register-submit"
							>
								Register
							</Button>
						</Box>
					</Box>
				</form>
				<Typography variant="body2" textAlign="center">
					<Link component={RouterLink} to="/login" variant="body2">
						Already have an account? Log in
					</Link>
				</Typography>
			</Box>
		</>
	);
}
