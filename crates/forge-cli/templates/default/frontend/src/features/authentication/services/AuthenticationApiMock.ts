/**
 * Mock AuthenticationApi service for tests.
 *
 * All methods succeed by default. Use options to override login result or force failures.
 */

import { Effect, Layer } from 'effect'
import {
  AuthenticationApi,
  type AuthenticationApiService,
  type LoginResult,
} from './AuthenticationApi'

export interface AuthenticationApiMockOptions {
  /** Result returned by login(). Defaults to { token: 'mock-token', needs_profile_select: false }. */
  readonly loginResult?: LoginResult
  /** If true, login() fails with an error. */
  readonly failLogin?: boolean
  /** If true, register() fails with an error. */
  readonly failRegister?: boolean
}

const defaultLoginResult: LoginResult = {
  token: 'mock-token',
  needs_profile_select: false,
}

/**
 * Creates a mock AuthenticationApi service. Optionally pass options to control behavior.
 */
export function makeAuthenticationApiMock(
  options: AuthenticationApiMockOptions = {},
): AuthenticationApiService {
  return {
    login: (_email: string, _password: string) =>
      options.failLogin
        ? Effect.fail(new Error('Mock login failed.'))
        : Effect.succeed(options.loginResult ?? defaultLoginResult),

    register: (_email: string, _password: string) =>
      options.failRegister
        ? Effect.fail(new Error('Mock register failed.'))
        : Effect.void,
  }
}

/**
 * Layer that provides a mock AuthenticationApi for tests.
 * No dependencies (no HttpClient).
 */
export const AuthenticationApiMock = (options?: AuthenticationApiMockOptions) =>
  Layer.succeed(AuthenticationApi, makeAuthenticationApiMock(options))
