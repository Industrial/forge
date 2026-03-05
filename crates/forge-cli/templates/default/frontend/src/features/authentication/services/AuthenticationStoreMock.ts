import { Effect, Layer } from 'effect'
import type { AuthenticationStoreService } from './AuthenticationStore'
import { AuthenticationStore } from './AuthenticationStore'
import { AuthenticationStateSnapshot } from '../domain/AuthenticationStateSnapshot'
import { AuthenticationUser } from '../domain/AuthenticationUser'
import { Flash } from '../domain/Flash'
import { Profile } from '../domain/Profile'

/**
 * Options for creating a mock AuthenticationStore (e.g. for tests).
 * Provide an initial state snapshot; all methods are no-op or return the state.
 */
export interface AuthenticationStoreMockOptions {
  /** Initial state returned by getState and used by getToken. Defaults to empty/logged-out. */
  readonly initialState?: Partial<AuthenticationStateSnapshot>
}

type MutableSnapshot = {
  user: AuthenticationUser | null
  profiles: Profile[]
  permissions: string[]
  flash: Flash | null
  token: string | null
  currentOrgId: string | null
  currentRoleId: string | null
  currentRoleName: string | null
  needs_profile_select: boolean
}

function toSnapshot(s: MutableSnapshot): AuthenticationStateSnapshot {
  return new AuthenticationStateSnapshot({
    ...s,
    user: s.user,
    profiles: s.profiles.map((p) =>
      p instanceof Profile ? p : new Profile(p),
    ),
    flash:
      s.flash != null && !(s.flash instanceof Flash)
        ? new Flash(s.flash)
        : s.flash,
  })
}

/**
 * Creates a mock AuthenticationStore service for tests.
 * State is mutable in memory; setToken/setScope/ logout update it (no localStorage).
 * fetchMe is a no-op that succeeds.
 */
function makeAuthenticationStoreMock(
  options: AuthenticationStoreMockOptions = {},
): AuthenticationStoreService {
  const state: MutableSnapshot = {
    user: null,
    profiles: [],
    permissions: [],
    flash: null,
    token: null,
    currentOrgId: null,
    currentRoleId: null,
    currentRoleName: null,
    needs_profile_select: false,
  }
  if (options.initialState) {
    const i = options.initialState
    if (i.user !== undefined)
      state.user =
        i.user === null
          ? null
          : i.user instanceof AuthenticationUser
            ? i.user
            : new AuthenticationUser(i.user)
    if (i.profiles !== undefined) state.profiles = [...i.profiles]
    if (i.permissions !== undefined) state.permissions = [...i.permissions]
    if (i.flash !== undefined)
      state.flash =
        i.flash === null
          ? null
          : i.flash instanceof Flash
            ? i.flash
            : new Flash(i.flash)
    if (i.token !== undefined) state.token = i.token
    if (i.currentOrgId !== undefined) state.currentOrgId = i.currentOrgId
    if (i.currentRoleId !== undefined) state.currentRoleId = i.currentRoleId
    if (i.currentRoleName !== undefined)
      state.currentRoleName = i.currentRoleName
    if (i.needs_profile_select !== undefined)
      state.needs_profile_select = i.needs_profile_select
  }

  return {
    getToken: () => Effect.succeed(state.token),
    setToken: (token) =>
      Effect.sync(() => {
        state.token = token
      }),
    getState: () => Effect.succeed(toSnapshot(state)),
    fetchMe: () => Effect.void,
    logout: () =>
      Effect.sync(() => {
        state.user = null
        state.profiles = []
        state.permissions = []
        state.flash = null
        state.token = null
        state.currentOrgId = null
        state.currentRoleId = null
        state.currentRoleName = null
        state.needs_profile_select = false
      }),
    setScope: (orgId, roleId, roleName) =>
      Effect.sync(() => {
        state.currentOrgId = orgId
        state.currentRoleId = roleId
        state.currentRoleName = roleName
      }),
  }
}

/**
 * Layer that provides a mock AuthenticationStore for tests.
 * No dependencies (no HttpClient, no localStorage).
 */
export const AuthenticationStoreMock = (
  options?: AuthenticationStoreMockOptions,
) => Layer.succeed(AuthenticationStore, makeAuthenticationStoreMock(options))
