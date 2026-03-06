import React, {
  createContext,
  useCallback,
  useContext,
  useEffect,
  useMemo,
} from 'react'
import { useNavigate } from 'react-router-dom'
import { Effect } from 'effect'
import { runApp } from '../lib/appRuntime'
import { on401HandlerRef, onScopeRequiredRef } from '../lib/authStateRef'
import type { AuthenticationUser } from '../features/authentication/domain/AuthenticationUser'
import type { Flash } from '../features/authentication/domain/Flash'
import type { Scope } from '../features/authentication/domain/Scope'
import { AuthenticationError } from '../features/authentication/domain/AuthenticationError'
import { AuthenticationStore } from '../features/authentication/services/AuthenticationStore'
import { useAuthenticationState } from '../features/authentication/hooks/useAuthentication'
import type { AppServices } from '../lib/appLayer'

/**
 * Context value: snapshot data + loading + imperative methods.
 * Data shape matches AuthenticationStateSnapshot from domain; methods and loading are context-specific.
 */
export type AuthenticationState = {
  user: AuthenticationUser | null
  scopes: Scope[]
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
  /** When true, redirect to scope selection before dashboard. */
  needs_scope_select: boolean
  loading: boolean
  /** Re-fetch user/scopes/permissions. Pass a token (e.g. from login) to set it and then fetch. */
  fetchMe: (tokenOverride?: string | null) => Promise<void>
  /** Re-read auth state from store (e.g. after auto-select scope). */
  refresh: () => Promise<void>
  /** Effect that runs fetchMe then refreshes React state; use in Effect.gen for composition. */
  fetchMeEffect: (
    tokenOverride?: string | null,
  ) => Effect.Effect<void, AuthenticationError, AppServices>
  logout: () => Promise<void>
  setCurrentScope: (
    orgId: string,
    roleId: string,
    roleName: string,
  ) => Promise<void>
  /** Set scope (org/role) client-side and refetch /api/auth/me. Use from navbar to switch scope. */
  switchScope: (
    orgId: string,
    roleId: string,
    roleName: string,
  ) => Promise<void>
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

const switchScopeEffect = (orgId: string, roleId: string, roleName: string) =>
  Effect.gen(function* () {
    const store = yield* AuthenticationStore
    const prev = yield* store.getState()
    yield* store.setScope(orgId, roleId, roleName)
    yield* store.fetchMe().pipe(
      Effect.catchAll((e) =>
        Effect.gen(function* () {
          yield* store.setScope(
            prev.currentOrgId ?? '',
            prev.currentRoleId ?? '',
            prev.currentRoleName ?? '',
          )
          return yield* Effect.fail(e)
        }),
      ),
    )
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
  console.log('AuthenticationProvider')

  useEffect(() => {
    Effect.gen(function* () {
      const store = yield* AuthenticationStore
      yield* store.fetchMe()
    })
  }, [])

  // const navigate = useNavigate()
  const { state, refresh, isPending } = useAuthenticationState()

  // useEffect(() => {
  //   on401HandlerRef.current = () => {
  //     runApp(logoutEffect)
  //       .then(refresh)
  //       .then(() => navigate('/authentication/login', { replace: true }))
  //   }
  //   onScopeRequiredRef.current = () => {
  //     navigate('/authentication/select-scope', { replace: true })
  //   }
  //   return () => {
  //     on401HandlerRef.current = () => {}
  //     onScopeRequiredRef.current = () => {}
  //   }
  // }, [refresh, navigate])

  const runThenRefresh = useCallback(
    <R extends AppServices>(
      effect: Effect.Effect<void, AuthenticationError, R>,
    ) => {
      return runApp(effect).then(refresh)
    },
    [refresh],
  )

  const fetchMe = useCallback(
    async (tokenOverride?: string | null) => {
      await runThenRefresh(fetchMeEffect(tokenOverride))
    },
    [runThenRefresh],
  )

  const fetchMeEffectForContext = useCallback(
    (tokenOverride?: string | null) =>
      fetchMeEffect(tokenOverride).pipe(
        Effect.andThen(() => Effect.promise(() => refresh())),
      ),
    [refresh],
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

  const switchScope = useCallback(
    async (orgId: string, roleId: string, roleName: string) => {
      await runApp(switchScopeEffect(orgId, roleId, roleName))
      refresh()
    },
    [refresh],
  )

  const value = useMemo<AuthenticationState>(
    () => ({
      user: state?.user ?? null,
      scopes: state?.scopes != null ? [...state.scopes] : [],
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
      needs_scope_select: state?.needs_scope_select ?? false,
      loading: isPending,
      fetchMe,
      refresh,
      fetchMeEffect: fetchMeEffectForContext,
      logout,
      setCurrentScope,
      switchScope,
    }),
    [
      state,
      isPending,
      setToken,
      fetchMe,
      refresh,
      fetchMeEffectForContext,
      logout,
      setCurrentScope,
      switchScope,
    ],
  )

  return (
    <AuthenticationContext.Provider value={value}>
      {children}
    </AuthenticationContext.Provider>
  )
}
