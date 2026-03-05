import { HttpClient, HttpClientRequest } from '@effect/platform'
import { Effect, Layer } from 'effect'
import {
  AuthenticationApi,
  type AuthenticationApiService,
  type LoginResult,
} from './AuthenticationApi'

function parseError(body: unknown): string {
  if (typeof body === 'object' && body !== null && 'error' in body) {
    return String((body as { error: unknown }).error)
  }
  return 'Request failed.'
}

export const AuthenticationApiLive = Layer.effect(
  AuthenticationApi,
  Effect.gen(function* () {
    const client = yield* HttpClient.HttpClient

    const login: AuthenticationApiService['login'] = (email, password) =>
      Effect.gen(function* () {
        yield* Effect.logTrace('AuthenticationApiLive.login')
        yield* Effect.logDebug(`login: email=${email}`)
        const response = yield* client.execute(
          HttpClientRequest.post('/api/auth/login').pipe(
            HttpClientRequest.bodyUnsafeJson({ email, password }),
          ),
        )
        const body = yield* response.json
        if (response.status < 200 || response.status >= 300) {
          return yield* Effect.fail(
            new Error(
              typeof body === 'object' && body !== null && 'error' in body
                ? String((body as { error: unknown }).error)
                : 'Invalid email or password',
            ),
          )
        }
        const data = body as { token?: string; needs_profile_select?: boolean }
        if (typeof data.token !== 'string') {
          return yield* Effect.fail(new Error('Invalid response from server.'))
        }
        const result = {
          token: data.token,
          needs_profile_select: data.needs_profile_select,
        } as LoginResult
        yield* Effect.logDebug(`login result: needs_profile_select=${result.needs_profile_select}`)
        return result
      })

    const register: AuthenticationApiService['register'] = (email, password) =>
      Effect.gen(function* () {
        yield* Effect.logTrace('AuthenticationApiLive.register')
        yield* Effect.logDebug(`register: email=${email}`)
        const response = yield* client.execute(
          HttpClientRequest.post('/api/auth/register').pipe(
            HttpClientRequest.bodyUnsafeJson({ email, password }),
          ),
        )
        const body = yield* response.json
        if (response.status < 200 || response.status >= 300) {
          return yield* Effect.fail(new Error(parseError(body)))
        }
        yield* Effect.logDebug('register: success')
      })

    return { login, register }
  }),
)

