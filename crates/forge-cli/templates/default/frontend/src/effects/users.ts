import { HttpClient, HttpClientRequest } from "@effect/platform";
import { Effect } from "effect";

type UserMembership = {
	org_id: string;
	org_name: string;
	roles: string[];
};

export type User = {
	id: string;
	email: string;
	is_active: boolean;
	is_admin: boolean;
	created_at: string;
	memberships: UserMembership[];
};

/**
 * Effect that fetches the users list from the dashboard API using HttpClient.
 * Depends on HttpClient (from @effect/platform); auth and baseUrl come from the layer at runtime.
 * Fails with Error on non-OK or permission/unauthorized responses.
 */
export const fetchUsersEffect = Effect.gen(function* () {
	const client = yield* HttpClient.HttpClient;
	const response = yield* client.execute(HttpClientRequest.get("/api/dashboard/users"));
	if (response.status === 401) {
		return yield* Effect.fail(new Error("Session expired or not logged in. Please log in again."));
	}
	if (response.status === 403) {
		return yield* Effect.fail(new Error("You do not have permission to view users."));
	}
	const body = yield* response.json;
	const ok = response.status >= 200 && response.status < 300;
	if (!ok) {
		const msg =
			typeof body === "object" && body !== null && "error" in body
				? String((body as { error: unknown }).error)
				: `HTTP ${response.status}`;
		return yield* Effect.fail(new Error(msg));
	}
	const data = body as { users?: User[] };
	return (data.users ?? []) as User[];
}).pipe(
	Effect.withSpan("fetchUsers", { attributes: { endpoint: "/api/dashboard/users" } })
);

/**
 * Legacy helper for call sites that still pass the imperative api (e.g. from useApi).
 * Prefer using fetchUsersEffect with a runtime that provides HttpClient.
 * @deprecated Use fetchUsersEffect with AppRuntimeProvider instead.
 */
export function fetchUsersEffectWithLegacyApi(
	api: (url: string, options?: RequestInit) => Promise<Response>
): Effect.Effect<User[], Error> {
	return Effect.tryPromise({
		try: async () => {
			const res = await api("/api/dashboard/users");
			if (res.status === 403) throw new Error("You do not have permission to view users.");
			if (res.status === 401) throw new Error("Session expired or not logged in. Please log in again.");
			if (!res.ok) {
				const text = await res.text();
				let msg: string;
				try {
					const json = JSON.parse(text) as { error?: string };
					msg = (json.error ?? text) || "Failed to load users.";
				} catch {
					msg = text || "Failed to load users.";
				}
				throw new Error(msg);
			}
			const data = (await res.json()) as { users?: User[] };
			return (data.users ?? []) as User[];
		},
		catch: (e) => (e instanceof Error ? e : new Error("Failed to load users")),
	});
}
