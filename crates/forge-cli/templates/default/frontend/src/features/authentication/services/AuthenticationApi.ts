/**
 * AuthenticationApi service: HTTP calls for login and register.
 * Scope is set client-side only (AuthenticationStore.setScope + fetchMe); no set-scope endpoint.
 * Used by LoginPage, RegisterPage. No auth header required for login/register.
 */

import { Context, Effect } from 'effect'

export interface LoginResult {
  readonly token: string
  readonly needs_profile_select?: boolean
}

export interface AuthenticationApiService {
  readonly login: (
    email: string,
    password: string,
  ) => Effect.Effect<LoginResult, Error, never>
  readonly register: (
    email: string,
    password: string,
  ) => Effect.Effect<void, Error, never>
}

export const AuthenticationApi =
  Context.GenericTag<AuthenticationApiService>('@forge/AuthenticationApi')
