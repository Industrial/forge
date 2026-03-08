/**
 * Authentication service: single capability for auth in the app.
 * Truth lives only in Effect; React runs effects at boundaries and reads from the reactive store
 * that this service updates (getCurrentUser, login, logout, selectScope).
 *
 * One service (Tag) per file; Live impl in AuthenticationLive.ts, test impl in AuthenticationMock.ts.
 */
import { Context, Effect, Option } from 'effect'

import type { AuthenticationUser } from '@/features/authentication/domain/AuthenticationUser'
import type {
  InvalidTokenError,
  LoginFailedError,
  RegistrationFailedError,
  ScopeError,
  TokenMissingError,
  UserNotFoundError,
} from '@/features/authentication/errors'

export interface Authentication {
  /**
   * Restore session from persisted token: read token from storage; if present,
   * fetch current user and update reactive store. Call once before mounting the app.
   */
  readonly restoreSession: () => Effect.Effect<void, never, never>

  /** Load current user from storage/API; updates reactive store. Returns none if not logged in or token invalid. */
  readonly getCurrentUser: () => Effect.Effect<
    Option.Option<AuthenticationUser>,
    InvalidTokenError | UserNotFoundError,
    never
  >

  /** Login with email/password; updates store and ref on success. */
  readonly login: (
    email: string,
    password: string,
  ) => Effect.Effect<
    AuthenticationUser,
    LoginFailedError | TokenMissingError | UserNotFoundError,
    never
  >

  /** Register with email/password. Does not log in; use login after. */
  readonly register: (
    email: string,
    password: string,
  ) => Effect.Effect<void, RegistrationFailedError, never>

  /** Clear session (storage, ref, reactive store). */
  readonly logout: () => Effect.Effect<void, never, never>

  /** Select ReBAC scope (organization + role); updates ref and store. */
  readonly selectScope: (
    organizationId: string,
    roleId: string,
  ) => Effect.Effect<void, ScopeError, never>
}

export const Authentication = Context.GenericTag<Authentication>(
  '@forge/Authentication',
)

export type AuthenticationService = Authentication
