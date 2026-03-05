import React, { createContext, useCallback, useContext, useMemo } from 'react'
import { Effect } from 'effect'
import { runWithAppRuntime } from '../lib/appLayer'
import type { AuthenticationUser } from '../features/authentication/domain/AuthenticationUser'
import type { Flash } from '../features/authentication/domain/Flash'
import type { Profile } from '../features/authentication/domain/Profile'
import { AuthenticationError } from '../features/authentication/domain/AuthenticationError'
import { AuthenticationApi } from '../features/authentication/services/AuthenticationApi'
import { AuthenticationStore } from '../features/authentication/services/AuthenticationStore'
import { useAuthenticationState } from '../features/authentication/hooks/useAuthentication'
import { useEffectRuntime } from '../lib/react-effect'
import type { AppServices } from '../lib/appLayer'

/**
 * Context value: snapshot data + loading + imperative methods.
 * Data shape matches AuthenticationStateSnapshot from domain; methods and loading are context-specific.
 */
export type AuthenticationState = {
  user: AuthenticationUser | null
  profiles: Profile[]
  permissions: string[]
  flash: Flash | null
  token: string | null
  setToken: (token: string | null) => void
  currentOrgId: string | null
  setCurrentOrgId: (orgId: string | null) => void
  currentRoleId: string | null
  setCurrentRoleId: (roleId: string | null) => void
  currentRoleName: string | null
  setCurrentRoleName: (roleName: string | null) => void
  /** When true, redirect to /select-profile before dashboard. */
  needs_profile_select: boolean
  loading: boolean
  /** Re-fetch user/profiles/permissions. Pass a token (e.g. from login) to set it and then fetch. */
  fetchMe: (tokenOverride?: string | null) => Promise<void>
  logout: () => Promise<void>
  setCurrentScope: (
    orgId: string,
    roleId: string,
    roleName: string,
  ) => Promise<void>
  /** Call API to set profile (org/role) then refresh state. Use from navbar to switch profile. */
  switchProfile: (orgId: string, roleId?: string) => Promise<void>
}

const AuthenticationContext = createContext<AuthenticationState | null>(null)

export function useAuthentication(): AuthenticationState {
  const ctx = useContext(AuthenticationContext)
  if (!ctx) {
    throw new Error(
      'useAuthentication must be used within AuthenticationProvider',
    )
  }
  return ctx
}

const fetchMeEffect = (tokenOverride?: string | null) =>
  Effect.gen(function* () {
    const store = yield* AuthenticationStore
    yield* store.fetchMe(tokenOverride)
  })

const logoutEffect = Effect.gen(function* () {
  const store = yield* AuthenticationStore
  yield* store.logout()
})

const setScopeEffect = (orgId: string, roleId: string, roleName: string) =>
  Effect.gen(function* () {
    const store = yield* AuthenticationStore
    yield* store.setScope(orgId, roleId, roleName)
  })

const setTokenEffect = (token: string | null) =>
  Effect.gen(function* () {
    const store = yield* AuthenticationStore
    yield* store.setToken(token)
  })

const switchProfileEffect = (orgId: string, roleId?: string) =>
  Effect.gen(function* () {
    const api = yield* AuthenticationApi
    const store = yield* AuthenticationStore
    yield* api.setProfile(orgId, roleId)
    yield* store.fetchMe()
  })

/**
 * Authentication provider backed by AuthenticationStore (Effect). Must be used
 * inside AuthenticationRuntimeProvider so the runtime has AuthenticationStore.
 */
export function AuthenticationProvider({
  children,
}: {
  children: React.ReactNode
}) {
  const { state, refresh, isPending } = useAuthenticationState()
  const { runtime } = useEffectRuntime<AppServices>()

  const runThenRefresh = useCallback(
    <R extends AppServices>(effect: Effect.Effect<void, AuthenticationError, R>) => {
      return runWithAppRuntime(runtime, effect).then(refresh)
    },
    [runtime, refresh],
  )

  const fetchMe = useCallback(
    async (tokenOverride?: string | null) => {
      await runThenRefresh(fetchMeEffect(tokenOverride))
    },
    [runThenRefresh],
  )

  const logout = useCallback(async () => {
    await runThenRefresh(logoutEffect)
  }, [runThenRefresh])

  const setCurrentScope = useCallback(
    async (orgId: string, roleId: string, roleName: string) => {
      await runThenRefresh(setScopeEffect(orgId, roleId, roleName))
    },
    [runThenRefresh],
  )

  const setToken = useCallback(
    async (token: string | null) => {
      await runThenRefresh(setTokenEffect(token))
    },
    [runThenRefresh],
  )

  const switchProfile = useCallback(
    async (orgId: string, roleId?: string) => {
      await runWithAppRuntime(runtime, switchProfileEffect(orgId, roleId))
      refresh()
    },
    [runtime, refresh],
  )

  const value = useMemo<AuthenticationState>(
    () => ({
      user: state?.user ?? null,
      profiles: state?.profiles != null ? [...state.profiles] : [],
      permissions: state?.permissions != null ? [...state.permissions] : [],
      flash: state?.flash ?? null,
      token: state?.token ?? null,
      setToken,
      currentOrgId: state?.currentOrgId ?? null,
      setCurrentOrgId: (orgId) =>
        setCurrentScope(
          orgId ?? '',
          state?.currentRoleId ?? '',
          state?.currentRoleName ?? '',
        ),
      currentRoleId: state?.currentRoleId ?? null,
      setCurrentRoleId: (roleId) =>
        setCurrentScope(
          state?.currentOrgId ?? '',
          roleId ?? '',
          state?.currentRoleName ?? '',
        ),
      currentRoleName: state?.currentRoleName ?? null,
      setCurrentRoleName: (roleName) =>
        setCurrentScope(
          state?.currentOrgId ?? '',
          state?.currentRoleId ?? '',
          roleName ?? '',
        ),
      needs_profile_select: state?.needs_profile_select ?? false,
      loading: isPending,
      fetchMe,
      logout,
      setCurrentScope,
      switchProfile,
    }),
    [state, isPending, setToken, fetchMe, logout, setCurrentScope, switchProfile],
  )

  return (
    <AuthenticationContext.Provider value={value}>
      {children}
    </AuthenticationContext.Provider>
  )
}
