import React, {
	createContext,
	useCallback,
	useContext,
	useEffect,
	useState,
} from "react";

export type SessionUser = {
	id: string;
	email: string;
	/** Current organization context from session (source of truth for dashboard scope). */
	current_org_id: string | null;
	/** Current role name from session. */
	current_role_name?: string | null;
};

export type Flash = { message?: string; error?: string };

export type Profile = {
	org_id: string;
	org_name: string;
	role_id?: string;
	role: string;
};

export type SessionState = {
	user: SessionUser | null;
	profiles: Profile[];
	permissions: string[];
	flash: Flash | null;
	/** When true, redirect to /select-profile before dashboard. */
	needs_profile_select: boolean;
	loading: boolean;
	refresh: () => Promise<void>;
};

const SessionContext = createContext<SessionState | null>(null);

export function useSession(): SessionState {
	const ctx = useContext(SessionContext);
	if (!ctx) throw new Error("useSession must be used within SessionProvider");
	return ctx;
}

async function fetchSession(): Promise<{
	user: SessionUser | null;
	profiles: Profile[];
	permissions: string[];
	flash: Flash | null;
	needs_profile_select: boolean;
}> {
	const res = await fetch("/api/auth/session", { credentials: "include" });
	if (!res.ok) {
		return {
			user: null,
			profiles: [],
			permissions: [],
			flash: null,
			needs_profile_select: false,
		};
	}
	const data = await res.json();
	const user = data.user
		? {
				id: data.user.id,
				email: data.user.email,
				current_org_id: data.user.current_org_id ?? null,
				current_role_name: data.user.current_role_name ?? null,
			}
		: null;
	const needs_profile_select = Boolean(data.needs_profile_select);
	return {
		user,
		profiles: Array.isArray(data.profiles) ? data.profiles : [],
		permissions: Array.isArray(data.permissions) ? data.permissions : [],
		flash:
			data.flash && (data.flash.message != null || data.flash.error != null)
				? data.flash
				: null,
		needs_profile_select,
	};
}

export function SessionProvider({ children }: { children: React.ReactNode }) {
	const [user, setUser] = useState<SessionUser | null>(null);
	const [profiles, setProfiles] = useState<Profile[]>([]);
	const [permissions, setPermissions] = useState<string[]>([]);
	const [flash, setFlash] = useState<Flash | null>(null);
	const [needs_profile_select, setNeedsProfileSelect] = useState(false);
	const [loading, setLoading] = useState(true);

	const refresh = useCallback(async () => {
		const { user: u, profiles: p, permissions: perm, flash: f, needs_profile_select: need } =
			await fetchSession();
		setUser(u);
		setProfiles(p);
		setPermissions(perm);
		setFlash(f);
		setNeedsProfileSelect(need);
	}, []);

	useEffect(() => {
		let cancelled = false;
		fetchSession()
			.then(({ user: u, profiles: p, permissions: perm, flash: f, needs_profile_select: need }) => {
				if (!cancelled) {
					setUser(u);
					setProfiles(p);
					setPermissions(perm);
					setFlash(f);
					setNeedsProfileSelect(need);
					setLoading(false);
				}
			})
			.catch(() => {
				if (!cancelled) {
					setUser(null);
					setProfiles([]);
					setPermissions([]);
					setFlash(null);
					setNeedsProfileSelect(false);
					setLoading(false);
				}
			});
		return () => {
			cancelled = true;
		};
	}, []);

	return (
		<SessionContext.Provider
			value={{ user, profiles, permissions, flash, needs_profile_select, loading, refresh }}
		>
			{children}
		</SessionContext.Provider>
	);
}
