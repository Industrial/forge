/**
 * AuthenticationStore service and auth-related data types.
 *
 * Provides the Effect service interface for authentication state: token, user,
 * profiles, permissions, and current org/role scope. All methods return
 * `Effect<A, AuthError, never>` (no requirement leakage); the Live
 * implementation uses HttpClient internally for `fetchMe`.
 *
 * @see AuthenticationStoreLive – browser implementation with Ref + localStorage
 * @see AuthenticationStoreMock – test double
 */

import { Context, Data, Effect } from "effect";

/**
 * User identity returned from `/api/auth/me`.
 *
 * @remarks
 * Tagged Data class for structural equality. The token is the session bearer
 * token used for API and WebSocket auth.
 */
export class AuthUser extends Data.TaggedClass("AuthUser")<{
	readonly id: string;
	readonly email: string;
	readonly token: string;
}> {}

/**
 * Flash message from the auth API (e.g. success or error after login).
 *
 * @remarks
 * Optional `message` (success) or `error` (failure). Used to display one-off
 * feedback in the auth UI.
 */
export class Flash extends Data.TaggedClass("Flash")<{
	readonly message?: string;
	readonly error?: string;
}> {}

/**
 * Profile: one org + role pair the user can switch to.
 *
 * @remarks
 * Used for profile-select and as the current scope. `role_id` is optional
 * for backwards compatibility; `role` is the role name (e.g. "owner", "viewer").
 */
export class Profile extends Data.TaggedClass("Profile")<{
	readonly org_id: string;
	readonly org_name: string;
	readonly role_id?: string;
	readonly role: string;
}> {}

/**
 * Read-only snapshot of auth state for consumers (e.g. React useAuth hook).
 *
 * @remarks
 * No setters; all mutations go through AuthenticationStore methods. Use
 * `getState()` to read the current snapshot. When `needs_profile_select` is
 * true, the app should redirect to profile-select before the dashboard.
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

/**
 * Error produced by AuthenticationStore operations.
 *
 * @remarks
 * Tagged Data error for pattern matching. Typically used when fetchMe fails
 * (e.g. network or invalid token) or when a method is used in an invalid state.
 */
export class AuthError extends Data.TaggedError("AuthError")<{
	readonly message: string;
	readonly cause?: unknown;
}> {}

/**
 * AuthenticationStore service interface.
 *
 * Manages auth state (token, user, profiles, permissions, scope) and
 * persistence (e.g. localStorage in the Live impl). All methods return
 * `Effect<A, AuthError, never>` so callers do not need to provide any
 * requirements; the Live layer supplies HttpClient for `fetchMe` internally.
 *
 * @remarks
 * - **getToken / setToken**: Read or write the bearer token (and persist in Live).
 * - **getState**: Current snapshot for UI; re-run after login, logout, or setScope.
 * - **fetchMe**: Call `/api/auth/me`, update internal state; use `tokenOverride` to set token then fetch (e.g. after login).
 * - **logout**: Clear token and scope; no API call.
 * - **setScope**: Set current org/role for the session (used for X-Organization-Id / X-Role-Id); does not call the API.
 */
export interface AuthenticationStoreService {
	/** Returns the current bearer token or null if logged out. */
	readonly getToken: () => Effect.Effect<string | null, AuthError, never>;
	/** Sets the bearer token (and persists in Live); pass null to clear. */
	readonly setToken: (token: string | null) => Effect.Effect<void, AuthError, never>;
	/** Returns a read-only snapshot of the current auth state. */
	readonly getState: () => Effect.Effect<AuthStateSnapshot, AuthError, never>;
	/**
	 * Fetches `/api/auth/me` and updates internal state (user, profiles, permissions, needs_profile_select).
	 * If `tokenOverride` is provided, sets the token first then fetches (e.g. after login).
	 */
	readonly fetchMe: (tokenOverride?: string | null) => Effect.Effect<void, AuthError, never>;
	/** Clears token and scope; no API call. */
	readonly logout: () => Effect.Effect<void, AuthError, never>;
	/**
	 * Sets the current scope (org + role) for the session.
	 * Does not call the API; used for request headers and UI.
	 */
	readonly setScope: (
		orgId: string,
		roleId: string,
		roleName: string,
	) => Effect.Effect<void, AuthError, never>;
}

/**
 * Tag for the AuthenticationStore service.
 *
 * Use this to access the service in Effects (e.g. `yield* AuthenticationStore`
 * or `Effect.flatMap(AuthenticationStore, store => store.getState())`). Provide
 * the service with `AuthenticationStoreLive` or `AuthenticationStoreMock` in your
 * Layer composition.
 *
 * @example
 * ```ts
 * const program = Effect.gen(function* () {
 *   const store = yield* AuthenticationStore;
 *   const state = yield* store.getState();
 *   return state.user?.email;
 * });
 * ```
 */
export const AuthenticationStore = Context.GenericTag<AuthenticationStoreService>(
	"@forge/AuthenticationStore",
);
