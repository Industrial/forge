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
  },
}))

/**
 * Callback run when any API response is 401 (e.g. token expired). Set by app
 * to clear session and redirect to login. See technical-choices §7.2.
 */
export const on401HandlerRef: { current: () => void } = { current: () => {} }
