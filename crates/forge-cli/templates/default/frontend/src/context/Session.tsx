import React, {
	createContext,
	useCallback,
	useContext,
	useEffect,
	useState,
} from "react";

export type SessionUser = { id: string; email: string };

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
}> {
	const res = await fetch("/api/auth/session", { credentials: "include" });
	if (!res.ok) return { user: null, profiles: [], permissions: [], flash: null };
	const data = await res.json();
	return {
		user: data.user ?? null,
		profiles: Array.isArray(data.profiles) ? data.profiles : [],
		permissions: Array.isArray(data.permissions) ? data.permissions : [],
		flash:
			data.flash && (data.flash.message != null || data.flash.error != null)
				? data.flash
				: null,
	};
}

export function SessionProvider({ children }: { children: React.ReactNode }) {
	const [user, setUser] = useState<SessionUser | null>(null);
	const [profiles, setProfiles] = useState<Profile[]>([]);
	const [permissions, setPermissions] = useState<string[]>([]);
	const [flash, setFlash] = useState<Flash | null>(null);
	const [loading, setLoading] = useState(true);

	const refresh = useCallback(async () => {
		const { user: u, profiles: p, permissions: perm, flash: f } =
			await fetchSession();
		setUser(u);
		setProfiles(p);
		setPermissions(perm);
		setFlash(f);
	}, []);

	useEffect(() => {
		let cancelled = false;
		fetchSession()
			.then(({ user: u, profiles: p, permissions: perm, flash: f }) => {
				if (!cancelled) {
					setUser(u);
					setProfiles(p);
					setPermissions(perm);
					setFlash(f);
					setLoading(false);
				}
			})
			.catch(() => {
				if (!cancelled) {
					setUser(null);
					setProfiles([]);
					setPermissions([]);
					setFlash(null);
					setLoading(false);
				}
			});
		return () => {
			cancelled = true;
		};
	}, []);

	return (
		<SessionContext.Provider
			value={{ user, profiles, permissions, flash, loading, refresh }}
		>
			{children}
		</SessionContext.Provider>
	);
}
