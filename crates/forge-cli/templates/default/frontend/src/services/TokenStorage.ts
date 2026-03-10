/**
 * Persisted auth token and scope (organization + role). Used by Authentication
 * to restore session and persist login/scope. Live uses localStorage; tests use Mock.
 */
import { Context, type Effect, type Option } from 'effect'

export interface TokenStorage {
  readonly getToken: () => Effect.Effect<Option.Option<string>, never, never>
  readonly setToken: (token: string) => Effect.Effect<void, never, never>
  readonly clearToken: () => Effect.Effect<void, never, never>

  readonly getScope: () => Effect.Effect<
    Option.Option<{ readonly organizationId: string; readonly roleId: string }>,
    never,
    never
  >
  readonly setScope: (
    organizationId: string,
    roleId: string,
  ) => Effect.Effect<void, never, never>
  readonly clearScope: () => Effect.Effect<void, never, never>
}

export const TokenStorage = Context.GenericTag<TokenStorage>(
  '@forge/TokenStorage',
)

export type TokenStorageService = TokenStorage
