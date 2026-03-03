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

function fetchUsersApi(): Promise<{ users: User[] }> {
	return fetch("/api/dashboard/users", { credentials: "include" }).then(async (res) => {
		if (res.status === 403) {
			throw new Error("You do not have permission to view users.");
		}
		if (res.status === 401) {
			throw new Error("Session expired or not logged in. Please log in again.");
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
			throw new Error(msg);
		}
		return res.json() as Promise<{ users: User[] }>;
	});
}

/**
 * Effect that fetches the users list from the dashboard API.
 * Fails with Error on non-OK or permission/unauthorized responses.
 */
export const fetchUsersEffect = Effect.tryPromise({
	try: () => fetchUsersApi(),
	catch: (e) => (e instanceof Error ? e : new Error("Failed to load users")),
}).pipe(Effect.map((data) => (data.users ?? []) as User[]));
