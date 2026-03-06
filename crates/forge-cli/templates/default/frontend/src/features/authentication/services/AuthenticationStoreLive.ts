import { HttpClient, HttpClientRequest } from '@effect/platform'
import { Effect, Layer, Ref } from 'effect'
import { AuthStateRef } from '../../../lib/authStateRef'
import type { AuthenticationStoreService } from './AuthenticationStore'
import { AuthenticationStore } from './AuthenticationStore'
import { AuthenticationError } from '../domain/AuthenticationError'
import { AuthenticationStateSnapshot } from '../domain/AuthenticationStateSnapshot'
import { AuthenticationUser } from '../domain/AuthenticationUser'
import { Flash } from '../domain/Flash'
import { Scope } from '../domain/Scope'

/**
 * Client-only scope persistence (no server-side scope session).
 * Scope and token are sent on every request via headers; backend derives scope from
 * X-Organization-Id / X-Role-Id only. See docs/technical-choices/01-scoped-session-and-profile-selection.md §1.
 */
const STORAGE_KEYS = {
  token: 'token',
  currentOrgId: 'currentOrgId',
  currentRoleId: 'currentRoleId',
  currentRoleName: 'currentRoleName',
} as const

function readStorage(key: string): string | null {
  if (typeof window === 'undefined') return null
  try {
    return localStorage.getItem(key)
  } catch {
    return null
  }
}

function writeStorage(key: string, value: string | null): void {
  if (typeof window === 'undefined') return
  try {
    if (value === null) localStorage.removeItem(key)
    else localStorage.setItem(key, value)
  } catch {
    // ignore
  }
}

/** API returns "profiles" and "needs_profile_select"; we map to scope terminology. */
function parseMeResponse(data: unknown): {
  user: AuthenticationUser | null
  scopes: Scope[]
  permissions: string[]
  flash: Flash | null
  needs_scope_select: boolean
} {
  if (!data || typeof data !== 'object') {
    return {
      user: null,
      scopes: [],
      permissions: [],
      flash: null,
      needs_scope_select: false,
    }
  }
  const d = data as Record<string, unknown>
  const user =
    d.user && typeof d.user === 'object' && d.user !== null
      ? new AuthenticationUser({
          id: String((d.user as Record<string, unknown>).id ?? ''),
          email: String((d.user as Record<string, unknown>).email ?? ''),
          token: '', // caller (fetchMe) sets token from request
        })
      : null
  const scopes: Scope[] = Array.isArray(d.profiles)
    ? (d.profiles as Record<string, unknown>[]).map(
        (p) =>
          new Scope({
            org_id: String(p.org_id ?? ''),
            org_name: String(p.org_name ?? ''),
            role_id: p.role_id != null ? String(p.role_id) : undefined,
            role: String(p.role ?? ''),
          }),
      )
    : []
  const permissions = Array.isArray(d.permissions)
    ? (d.permissions as string[])
    : []
  const flash =
    d.flash &&
    typeof d.flash === 'object' &&
    d.flash !== null &&
    ((d.flash as Record<string, unknown>).message != null ||
      (d.flash as Record<string, unknown>).error != null)
      ? new Flash(d.flash as { message?: string; error?: string })
      : null
  const needs_scope_select = Boolean(d.needs_profile_select)
  return {
    user,
    scopes,
    permissions,
    flash,
    needs_scope_select,
  }
}

function initialSnapshot(): AuthenticationStateSnapshot {
  const token = readStorage(STORAGE_KEYS.token)
  return new AuthenticationStateSnapshot({
    user: null,
    scopes: [],
    permissions: [],
    flash: null,
    token,
    currentOrgId: readStorage(STORAGE_KEYS.currentOrgId),
    currentRoleId: readStorage(STORAGE_KEYS.currentRoleId),
    currentRoleName: readStorage(STORAGE_KEYS.currentRoleName),
    needs_scope_select: false,
  })
}

/**
 * Live implementation of AuthenticationStore: state in a Ref, persist token/scope to localStorage,
 * fetchMe via HttpClient GET /api/auth/me. Syncs token/scope to AuthStateRef so the HTTP layer
 * can inject them at request time (single runtime, no rebuild on scope switch).
 * Layer requires HttpClient and AuthStateRef.
 */
export const AuthenticationStoreLive = Layer.effect(
  AuthenticationStore,
  Effect.gen(function* () {
    const client = yield* HttpClient.HttpClient
    const authStateRef = yield* AuthStateRef
    const ref = yield* Ref.make(initialSnapshot())

    const syncToAuthStateRef = (s: AuthenticationStateSnapshot) => {
      authStateRef.current.token = s.token
      authStateRef.current.organizationId = s.currentOrgId
      authStateRef.current.roleId = s.currentRoleId
      authStateRef.current.needs_scope_select = s.needs_scope_select
    }
    syncToAuthStateRef(initialSnapshot())

    const persistToken = (token: string | null) =>
      Effect.sync(() => writeStorage(STORAGE_KEYS.token, token))

    const persistScope = (
      orgId: string | null,
      roleId: string | null,
      roleName: string | null,
    ) =>
      Effect.sync(() => {
        writeStorage(STORAGE_KEYS.currentOrgId, orgId)
        writeStorage(STORAGE_KEYS.currentRoleId, roleId)
        writeStorage(STORAGE_KEYS.currentRoleName, roleName)
      })

    const store: AuthenticationStoreService = {
      getToken: () =>
        Effect.gen(function* () {
          yield* Effect.logTrace('AuthenticationStoreLive.getToken')
          const s = yield* Ref.get(ref)
          yield* Effect.logDebug(`getToken result: hasToken=${s.token != null}`)
          return s.token
        }),

      setToken: (token) =>
        Effect.gen(function* () {
          yield* Effect.logTrace('AuthenticationStoreLive.setToken')
          yield* Effect.logDebug(`setToken: hasToken=${token != null}`)
          yield* Ref.update(ref, (s) => {
            const next = new AuthenticationStateSnapshot({ ...s, token })
            syncToAuthStateRef(next)
            return next
          })
          yield* persistToken(token)
        }),

      getState: () =>
        Effect.gen(function* () {
          yield* Effect.logTrace('AuthenticationStoreLive.getState')
          const s = yield* Ref.get(ref)
          yield* Effect.logDebug(
            `getState: user=${s.user?.email ?? 'null'}, needs_scope_select=${s.needs_scope_select}`,
          )
          return s
        }),

      fetchMe: (tokenOverride) =>
        Effect.gen(function* () {
          yield* Effect.logTrace('AuthenticationStoreLive.fetchMe')
          yield* Effect.logDebug(
            `fetchMe: tokenOverride=${tokenOverride != null ? 'provided' : 'none'}`,
          )
          let token: string | null
          if (tokenOverride !== undefined && tokenOverride !== null) {
            token = tokenOverride
            yield* Ref.update(ref, (s) => {
              const next = new AuthenticationStateSnapshot({ ...s, token })
              syncToAuthStateRef(next)
              return next
            })
            yield* persistToken(token)
          } else {
            const s = yield* Ref.get(ref)
            token = s.token
          }
          if (!token) {
            yield* Effect.logDebug('fetchMe: no token, resetting state')
            const empty = initialSnapshot()
            syncToAuthStateRef(empty)
            yield* Ref.set(ref, empty)
            return
          }
          const req = HttpClientRequest.get('/api/auth/me').pipe(
            HttpClientRequest.setHeader('Authorization', `Bearer ${token}`),
          )
          const response = yield* client.execute(req)
          const ok = response.status >= 200 && response.status < 300
          if (!ok) {
            yield* Effect.logDebug(
              `fetchMe: non-ok status=${response.status}, resetting state`,
            )
            yield* persistToken(null)
            const empty = initialSnapshot()
            syncToAuthStateRef(empty)
            yield* Ref.set(ref, empty)
            return
          }
          const body = yield* response.json
          const parsed = parseMeResponse(body)
          const user: AuthenticationUser | null = parsed.user
            ? new AuthenticationUser({ ...parsed.user, token })
            : null
          const current = yield* Ref.get(ref)
          // When we have profiles but no scope selected, default to first so RPC/entity requests get X-Organization-Id and X-Role-Id
          const defaultScope =
            parsed.scopes.length > 0 &&
            (current.currentOrgId == null || current.currentRoleId == null)
              ? parsed.scopes[0]
              : null
          const next = new AuthenticationStateSnapshot({
            ...current,
            user,
            scopes: parsed.scopes,
            permissions: parsed.permissions,
            flash: parsed.flash,
            needs_scope_select: parsed.needs_scope_select,
            ...(defaultScope
              ? {
                  currentOrgId: defaultScope.org_id,
                  currentRoleId: defaultScope.role_id ?? null,
                  currentRoleName: defaultScope.role ?? null,
                }
              : {}),
          })
          syncToAuthStateRef(next)
          yield* Ref.set(ref, next)
          if (defaultScope != null) {
            yield* persistScope(
              next.currentOrgId,
              next.currentRoleId,
              next.currentRoleName,
            )
          }
          yield* Effect.logDebug(
            `fetchMe: success user=${user?.email ?? 'null'}, needs_scope_select=${parsed.needs_scope_select}`,
          )
        }).pipe(
          Effect.catchAll((e) =>
            Effect.fail(
              new AuthenticationError({
                message: e instanceof Error ? e.message : 'fetchMe failed',
                cause: e,
              }),
            ),
          ),
        ),

      logout: () =>
        Effect.gen(function* () {
          yield* Effect.logTrace('AuthenticationStoreLive.logout')
          const empty = initialSnapshot()
          syncToAuthStateRef(empty)
          yield* Ref.set(ref, empty)
          yield* persistToken(null)
          yield* persistScope(null, null, null)
        }),

      setScope: (orgId, roleId, roleName) =>
        Effect.gen(function* () {
          yield* Effect.logTrace('AuthenticationStoreLive.setScope')
          yield* Effect.logDebug(
            `setScope: orgId=${orgId}, roleId=${roleId}, roleName=${roleName}`,
          )
          yield* Ref.update(ref, (s) => {
            const next = new AuthenticationStateSnapshot({
              ...s,
              currentOrgId: orgId,
              currentRoleId: roleId,
              currentRoleName: roleName,
            })
            syncToAuthStateRef(next)
            return next
          })
          yield* persistScope(orgId, roleId, roleName)
        }),
    }

    return store
  }),
)
