import { Context, Data, Effect } from "effect";

/** User identity returned from /api/auth/me */
export class AuthUser extends Data.TaggedClass("AuthUser")<{
	readonly id: string;
	readonly email: string;
	readonly token: string;
}> {}

/** Flash message from auth API */
export class Flash extends Data.TaggedClass("Flash")<{
	readonly message?: string;
	readonly error?: string;
}> {}

/** Profile (org + role) for profile-select and scope */
export class Profile extends Data.TaggedClass("Profile")<{
	readonly org_id: string;
	readonly org_name: string;
	readonly role_id?: string;
	readonly role: string;
}> {}

/**
 * Read-only snapshot of auth state for React (e.g. useAuth hook).
 * No setters; all mutations go through AuthenticationStore methods.
 */
export class AuthStateSnapshot extends Data.TaggedClass("AuthStateSnapshot")<{
	readonly user: AuthUser | null;
	readonly profiles: readonly Profile[];
	readonly permissions: readonly string[];
	readonly flash: Flash | null;
	readonly token: string | null;
	readonly currentOrgId: string | null;
	readonly currentRoleId: string | null;
	readonly currentRoleName: string | null;
	/** When true, redirect to /select-profile before dashboard. */
	readonly needs_profile_select: boolean;
}> {}

export class AuthError extends Data.TaggedError("AuthError")<{
	readonly message: string;
	readonly cause?: unknown;
}> {}

/**
 * AuthenticationStore service interface.
 * All methods return Effect<A, AuthError, never> — no requirement leakage;
 * the Live implementation may require HttpClient internally for fetchMe.
 */
export interface AuthenticationStoreService {
	readonly getToken: () => Effect.Effect<string | null, AuthError, never>;
	readonly setToken: (token: string | null) => Effect.Effect<void, AuthError, never>;
	readonly getState: () => Effect.Effect<AuthStateSnapshot, AuthError, never>;
	/**
	 * Fetch /api/auth/me and update internal state.
	 * Optional tokenOverride: if provided, set token first then fetch.
	 */
	readonly fetchMe: (tokenOverride?: string | null) => Effect.Effect<void, AuthError, never>;
	readonly logout: () => Effect.Effect<void, AuthError, never>;
	/**
	 * Set current scope (org + role). Does not call API; updates session state only.
	 */
	readonly setScope: (
		orgId: string,
		roleId: string,
		roleName: string,
	) => Effect.Effect<void, AuthError, never>;
}

/**
 * Tag for the AuthenticationStore service.
 * Use this to access auth state and actions in Effects.
 * No R in the interface — Live layer will provide HttpClient for fetchMe.
 */
export const AuthenticationStore = Context.GenericTag<AuthenticationStoreService>(
	"@forge/AuthenticationStore",
);
