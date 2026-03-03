import React, {
	createContext,
	useCallback,
	useContext,
	useEffect,
	useState,
} from "react";

export type AuthUser = {
	id: string;
	email: string;
	token: string;
};

export type Flash = { message?: string; error?: string };

export type Profile = {
	org_id: string;
	org_name: string;
	role_id?: string;
	role: string;
};

export type AuthState = {
	user: AuthUser | null;
	profiles: Profile[];
	permissions: string[];
	flash: Flash | null;
	token: string | null;
	setToken: (token: string | null) => void;
	currentOrgId: string | null;
	setCurrentOrgId: (orgId: string | null) => void;
	currentRoleName: string | null;
	setCurrentRoleName: (roleName: string | null) => void;
	/** When true, redirect to /select-profile before dashboard. */
	needs_profile_select: boolean;
	loading: boolean;
	fetchMe: () => Promise<void>;
	logout: () => Promise<void>;
	setCurrentScope: (orgId: string, roleName: string) => Promise<void>;
};

const AuthContext = createContext<AuthState | null>(null);

export function useAuth(): AuthState {
	const ctx = useContext(SessionContext);
	if (!ctx) throw new Error("useSession must be used within SessionProvider");
	return ctx;
}

async function fetchMe(token: string): Promise<{
	user: AuthUser | null;
	profiles: Profile[];
	permissions: string[];
	flash: Flash | null;
	needs_profile_select: boolean;
}> {
	const res = await fetch("/api/auth/me", {
		headers: { Authorization: `Bearer ${token}` },
	});
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
				token,
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

export function AuthProvider({ children }: { children: React.ReactNode }) {
	const [user, setUser] = useState<AuthUser | null>(null);
	const [token, setToken] = useState<string | null>(localStorage.getItem("token"));
	const [currentOrgId, setCurrentOrgId] = useState<string | null>(
		localStorage.getItem("currentOrgId"),
	);
	const [currentRoleName, setCurrentRoleName] = useState<string | null>(
		localStorage.getItem("currentRoleName"),
	);
	const [profiles, setProfiles] = useState<Profile[]>([]);
	const [permissions, setPermissions] = useState<string[]>([]);
	const [flash, setFlash] = useState<Flash | null>(null);
	const [needs_profile_select, setNeedsProfileSelect] = useState(false);
	const [loading, setLoading] = useState(true);

	// Store token in local storage
	useEffect(() => {
		if (token) {
			localStorage.setItem("token", token);
		} else {
			localStorage.removeItem("token");
		}
	}, [token]);

	// Store currentOrgId in local storage
	useEffect(() => {
		if (currentOrgId) {
			localStorage.setItem("currentOrgId", currentOrgId);
		} else {
			localStorage.removeItem("currentOrgId");
		}
	}, [currentOrgId]);

	// Store currentRoleName in local storage
	useEffect(() => {
		if (currentRoleName) {
			localStorage.setItem("currentRoleName", currentRoleName);
		} else {
			localStorage.removeItem("currentRoleName");
		}
	}, [currentRoleName]);

	const fetchMeCb = useCallback(async () => {
		setLoading(true);
		if (!token) {
			setUser(null);
			setProfiles([]);
			setPermissions([]);
			setFlash(null);
			setNeedsProfileSelect(false);
			setLoading(false);
			return;
		}

		const { user: u, profiles: p, permissions: perm, flash: f, needs_profile_select: need } =
			await fetchMe(token);

		setUser(u);
		setProfiles(p);
		setPermissions(perm);
		setFlash(f);
		setNeedsProfileSelect(need);
		setLoading(false);
	}, [token]);

	const logout = useCallback(async () => {
		// TODO: Call API to revoke token
		setToken(null);
		setCurrentOrgId(null);
		setCurrentRoleName(null);
		setUser(null);
		setProfiles([]);
		setPermissions([]);
		setFlash(null);
		setNeedsProfileSelect(false);
	}, []);

	const setCurrentScope = useCallback(async (orgId: string, roleName: string) => {
		setCurrentOrgId(orgId);
		setCurrentRoleName(roleName);
		// TODO: Potentially re-fetch permissions based on new scope
	}, []);

	useEffect(() => {
		fetchMeCb();
	}, [fetchMeCb]);

	return (
		<AuthContext.Provider
			value={{
				user,
				profiles,
				permissions,
				flash,
				token,
				setToken,
				currentOrgId,
				setCurrentOrgId,
				currentRoleName,
				setCurrentRoleName,
				needs_profile_select,
				loading,
				fetchMe: fetchMeCb,
				logout,
				setCurrentScope,
			}}
		>
			{children}
		</AuthContext.Provider>
	);
}
