import { useState } from "react";
import { useNavigate, useLocation } from "react-router-dom";
import {
	Button,
	Content,
	Form,
	Heading,
	InlineAlert,
	Link,
	TextField,
} from "@react-spectrum/s2";
import { style } from "@react-spectrum/s2/style" with { type: "macro" };
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
			<Heading level={1} styles={style({ font: "heading-xl" })}>
				Log in
			</Heading>
			<div
				className={style({ display: "flex", flexDirection: "column", gap: 12 })}
			>
				<Form onSubmit={handleSubmit} data-testid="login-form">
					{error != null && (
						<InlineAlert
							variant="negative"
							fillStyle="border"
							data-testid="login-error"
						>
							<Heading>Login failed</Heading>
							<Content>{error}</Content>
						</InlineAlert>
					)}
					<TextField
						name="email"
						type="email"
						label="Email"
						placeholder="you@example.com"
						value={email}
						onChange={setEmail}
						isRequired
						isDisabled={submitting}
						data-testid="login-email"
					/>
					<TextField
						name="password"
						type="password"
						label="Password"
						placeholder="••••••••"
						value={password}
						onChange={setPassword}
						isRequired
						isDisabled={submitting}
						data-testid="login-password"
					/>
					<div
						className={style({
							display: "flex",
							flexDirection: "row",
							justifyContent: "end",
						})}
					>
						<Button
							type="submit"
							variant="accent"
							isDisabled={submitting}
							data-testid="login-submit"
						>
							Log in
						</Button>
					</div>
				</Form>
				<div className={style({ textAlign: "center", font: "body" })}>
					<Link href="/register">
						Don&apos;t have an account? Create an Account
					</Link>
				</div>
			</div>
		</>
	);
}
