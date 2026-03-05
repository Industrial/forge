import { Context, Layer } from 'effect'

/**
 * Mutable ref holding current auth state for request-time header injection.
 * The HTTP layer reads this synchronously in mapRequest; AuthenticationStore
 * updates it whenever token or scope changes. Single source of truth so
 * no runtime rebuild when user switches scope. See technical-choices §2.
 */
export interface AuthStateRef {
  readonly current: {
    token: string | null
    organizationId: string | null
    roleId: string | null
    /** When true, scoped API calls must be blocked until user selects scope (Epic 1). */
    needs_scope_select: boolean
  }
}

export const AuthStateRef = Context.GenericTag<AuthStateRef>(
  '@forge/AuthStateRef',
)

export const AuthStateRefLayer = Layer.sync(AuthStateRef, () => ({
  current: {
    token: null,
    organizationId: null,
    roleId: null,
    needs_scope_select: false,
  },
}))

/**
 * Callback run when any API response is 401 (e.g. token expired). Set by app
 * to clear session and redirect to login. See technical-choices §7.2.
 */
export const on401HandlerRef: { current: () => void } = { current: () => {} }

/**
 * Callback run when a scoped API is called but scope is required and not set
 * (e.g. needs_scope_select and no org/role). Set by app to redirect to
 * scope selection. Epic 1: explicit scope guard before API calls.
 */
export const onScopeRequiredRef: { current: () => void } = { current: () => {} }
