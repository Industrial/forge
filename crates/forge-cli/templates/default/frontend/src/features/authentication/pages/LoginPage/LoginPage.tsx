import { useState } from "react";
import { useForm, Controller } from "react-hook-form";
import { Link as RouterLink, useNavigate, useLocation } from "react-router-dom";
import { Schema } from "effect";
import Alert from "@mui/material/Alert";
import Box from "@mui/material/Box";
import Button from "@mui/material/Button";
import Link from "@mui/material/Link";
import TextField from "@mui/material/TextField";
import Typography from "@mui/material/Typography";
import { useSession } from "../../../../context/Session";
import { effectSchemaResolver } from "../../../../lib/effectSchemaResolver";
import {
	loginFormSchema,
	type LoginFormValues,
} from "../../../../schemas/userFormSchemas";

export default function LoginPage() {
	const navigate = useNavigate();
	const location = useLocation();
	const { refresh } = useSession();
	const [error, setError] = useState<string | null>(null);
	const [submitting, setSubmitting] = useState(false);

	const from =
		(location.state as { from?: { pathname: string } } | null)?.from
			?.pathname ?? "/dashboard";

	const form = useForm<LoginFormValues>({
		resolver: effectSchemaResolver(
			loginFormSchema as Schema.Schema<LoginFormValues, unknown, never>,
		),
		defaultValues: { email: "", password: "" },
		mode: "onChange",
	});

	async function handleSubmit(data: LoginFormValues) {
		setSubmitting(true);
		setError(null);
		try {
			const res = await fetch("/api/auth/login", {
				method: "POST",
				headers: { "Content-Type": "application/json" },
				credentials: "include",
				body: JSON.stringify({ email: data.email, password: data.password }),
			});
			const resData = await res.json().catch(() => ({}));
			if (res.ok && typeof resData.token === "string") {
				await refresh(resData.token);
				if (resData.needs_profile_select === true) {
					navigate("/select-profile", { replace: true });
				} else {
					navigate(from, { replace: true });
				}
				return;
			}
			setError(
				typeof resData?.error === "string"
					? resData.error
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
				<form
					onSubmit={form.handleSubmit(handleSubmit)}
					data-testid="login-form"
				>
					<Box sx={{ display: "flex", flexDirection: "column", gap: 2 }}>
						{error != null && (
							<Alert severity="error" data-testid="login-error">
								<Typography variant="subtitle2">Login failed</Typography>
								{error}
							</Alert>
						)}
						<Controller
							control={form.control}
							name="email"
							render={({ field, fieldState }) => (
								<TextField
									{...field}
									name="email"
									type="email"
									label="Email"
									placeholder="you@example.com"
									required
									disabled={submitting}
									data-testid="login-email"
									fullWidth
									error={Boolean(fieldState.error)}
									helperText={fieldState.error?.message}
								/>
							)}
						/>
						<Controller
							control={form.control}
							name="password"
							render={({ field, fieldState }) => (
								<TextField
									{...field}
									name="password"
									type="password"
									label="Password"
									placeholder="••••••••"
									required
									disabled={submitting}
									data-testid="login-password"
									fullWidth
									error={Boolean(fieldState.error)}
									helperText={fieldState.error?.message}
								/>
							)}
						/>
						<Box sx={{ display: "flex", justifyContent: "flex-end" }}>
							<Button
								type="submit"
								variant="contained"
								disabled={submitting || !form.formState.isValid}
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
