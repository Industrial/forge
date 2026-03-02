import { useState } from "react";
import { useNavigate } from "react-router-dom";
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
			<Heading level={1} styles={style({ font: "heading-xl" })}>
				Create an account
			</Heading>
			<div
				className={style({ display: "flex", flexDirection: "column", gap: 12 })}
			>
				<Form onSubmit={handleSubmit} data-testid="register-form">
					{error != null && (
						<InlineAlert
							variant="negative"
							fillStyle="border"
							data-testid="register-error"
						>
							<Heading>Registration failed</Heading>
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
						data-testid="register-email"
					/>
					<TextField
						name="password"
						type="password"
						label="Password"
						placeholder="••••••••"
						value={password}
						onChange={setPassword}
						isRequired
						minLength={8}
						isDisabled={submitting}
						data-testid="register-password"
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
							data-testid="register-submit"
						>
							Register
						</Button>
					</div>
				</Form>
				<div className={style({ textAlign: "center", font: "body" })}>
					<Link href="/login">Already have an account? Log in</Link>
				</div>
			</div>
		</>
	);
}
