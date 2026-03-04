/**
 * AuthenticationStore service.
 *
 * Provides the Effect service interface for authentication state: token, user,
 * profiles, permissions, and current org/role scope. All methods return
 * `Effect<A, AuthError, never>` (no requirement leakage); the Live
 * implementation uses HttpClient internally for `fetchMe`.
 *
 * @see AuthenticationStoreLive – browser implementation with Ref + localStorage
 * @see AuthenticationStoreMock – test double
 */

import { Context, Effect } from "effect";
import {
	AuthError,
	AuthStateSnapshot,
	AuthUser,
	Flash,
	Profile,
} from "../domain";

export { AuthError, AuthStateSnapshot, AuthUser, Flash, Profile };

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
