import { HttpClient, HttpClientRequest } from "@effect/platform";
import { Effect, Layer, Ref } from "effect";
import type { AuthenticationStoreService } from "./AuthenticationStore";
import {
	AuthError,
	AuthStateSnapshot,
	AuthUser,
	AuthenticationStore,
	Flash,
	Profile,
} from "./AuthenticationStore";

const STORAGE_KEYS = {
	token: "token",
	currentOrgId: "currentOrgId",
	currentRoleId: "currentRoleId",
	currentRoleName: "currentRoleName",
} as const;

function readStorage(key: string): string | null {
	if (typeof window === "undefined") return null;
	try {
		return localStorage.getItem(key);
	} catch {
		return null;
	}
}

function writeStorage(key: string, value: string | null): void {
	if (typeof window === "undefined") return;
	try {
		if (value === null) localStorage.removeItem(key);
		else localStorage.setItem(key, value);
	} catch {
		// ignore
	}
}

function parseMeResponse(data: unknown): {
	user: AuthUser | null;
	profiles: Profile[];
	permissions: string[];
	flash: Flash | null;
	needs_profile_select: boolean;
} {
	if (!data || typeof data !== "object") {
		return {
			user: null,
			profiles: [],
			permissions: [],
			flash: null,
			needs_profile_select: false,
		};
	}
	const d = data as Record<string, unknown>;
	const user = d.user && typeof d.user === "object" && d.user !== null
		? new AuthUser({
				id: String((d.user as Record<string, unknown>).id ?? ""),
				email: String((d.user as Record<string, unknown>).email ?? ""),
				token: "", // caller (fetchMe) sets token from request
			})
		: null;
	const profiles: Profile[] = Array.isArray(d.profiles)
		? (d.profiles as Record<string, unknown>[]).map(
				(p) =>
					new Profile({
						org_id: String(p.org_id ?? ""),
						org_name: String(p.org_name ?? ""),
						role_id: p.role_id != null ? String(p.role_id) : undefined,
						role: String(p.role ?? ""),
					}),
			)
		: [];
	const permissions = Array.isArray(d.permissions) ? (d.permissions as string[]) : [];
	const flash =
		d.flash &&
		typeof d.flash === "object" &&
		d.flash !== null &&
		((d.flash as Record<string, unknown>).message != null ||
			(d.flash as Record<string, unknown>).error != null)
			? new Flash(d.flash as { message?: string; error?: string })
			: null;
	const needs_profile_select = Boolean(d.needs_profile_select);
	return {
		user,
		profiles,
		permissions,
		flash,
		needs_profile_select,
	};
}

function initialSnapshot(): AuthStateSnapshot {
	const token = readStorage(STORAGE_KEYS.token);
	return new AuthStateSnapshot({
		user: null,
		profiles: [],
		permissions: [],
		flash: null,
		token,
		currentOrgId: readStorage(STORAGE_KEYS.currentOrgId),
		currentRoleId: readStorage(STORAGE_KEYS.currentRoleId),
		currentRoleName: readStorage(STORAGE_KEYS.currentRoleName),
		needs_profile_select: false,
	});
}

/**
 * Live implementation of AuthenticationStore: state in a Ref, persist token/scope to localStorage,
 * fetchMe via HttpClient GET /api/auth/me.
 * Layer requires HttpClient (e.g. from httpClientWithAuthLayer with baseUrl; token/scope can be updated by this service).
 */
export const AuthenticationStoreLive = Layer.effect(
	AuthenticationStore,
	Effect.gen(function* () {
		const client = yield* HttpClient.HttpClient;
		const ref = yield* Ref.make(initialSnapshot());

		const persistToken = (token: string | null) =>
			Effect.sync(() => writeStorage(STORAGE_KEYS.token, token));

		const persistScope = (orgId: string | null, roleId: string | null, roleName: string | null) =>
			Effect.sync(() => {
				writeStorage(STORAGE_KEYS.currentOrgId, orgId);
				writeStorage(STORAGE_KEYS.currentRoleId, roleId);
				writeStorage(STORAGE_KEYS.currentRoleName, roleName);
			});

		const store: AuthenticationStoreService = {
			getToken: () =>
				Effect.gen(function* () {
					const s = yield* Ref.get(ref);
					return s.token;
				}),

			setToken: (token) =>
				Effect.gen(function* () {
					yield* Ref.update(ref, (s) => new AuthStateSnapshot({ ...s, token }));
					yield* persistToken(token);
				}),

			getState: () => Ref.get(ref),

			fetchMe: (tokenOverride) =>
				Effect.gen(function* () {
					let token: string | null;
					if (tokenOverride !== undefined && tokenOverride !== null) {
						token = tokenOverride;
						yield* Ref.update(ref, (s) => new AuthStateSnapshot({ ...s, token }));
						yield* persistToken(token);
					} else {
						const s = yield* Ref.get(ref);
						token = s.token;
					}
					if (!token) {
						yield* Ref.set(
							ref,
							initialSnapshot(),
						);
						return;
					}
					const req = HttpClientRequest.get("/api/auth/me").pipe(
						HttpClientRequest.setHeader("Authorization", `Bearer ${token}`),
					);
					const response = yield* client.execute(req);
					const body = yield* response.json;
					const ok = response.status >= 200 && response.status < 300;
					if (!ok) {
						yield* persistToken(null);
						yield* Ref.set(ref, initialSnapshot());
						return;
					}
					const parsed = parseMeResponse(body);
					const user: AuthUser | null = parsed.user
						? new AuthUser({ ...parsed.user, token })
						: null;
					const current = yield* Ref.get(ref);
					yield* Ref.set(
						ref,
						new AuthStateSnapshot({
							...current,
							user,
							profiles: parsed.profiles,
							permissions: parsed.permissions,
							flash: parsed.flash,
							needs_profile_select: parsed.needs_profile_select,
						}),
					);
				}).pipe(
					Effect.catchAll((e) =>
						Effect.fail(
							new AuthError({
								message: e instanceof Error ? e.message : "fetchMe failed",
								cause: e,
							}),
						),
					),
				),

			logout: () =>
				Effect.gen(function* () {
					yield* Ref.set(ref, initialSnapshot());
					yield* persistToken(null);
					yield* persistScope(null, null, null);
				}),

			setScope: (orgId, roleId, roleName) =>
				Effect.gen(function* () {
					yield* Ref.update(ref, (s) =>
						new AuthStateSnapshot({
							...s,
							currentOrgId: orgId,
							currentRoleId: roleId,
							currentRoleName: roleName,
						}),
					);
					yield* persistScope(orgId, roleId, roleName);
				}),
		};

		return store;
	}),
);
