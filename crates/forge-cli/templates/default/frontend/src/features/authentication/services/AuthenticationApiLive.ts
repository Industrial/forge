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
        return {
          token: data.token,
          needs_profile_select: data.needs_profile_select,
        } as LoginResult
      })

    const register: AuthenticationApiService['register'] = (email, password) =>
      Effect.gen(function* () {
        const response = yield* client.execute(
          HttpClientRequest.post('/api/auth/register').pipe(
            HttpClientRequest.bodyUnsafeJson({ email, password }),
          ),
        )
        const body = yield* response.json
        if (response.status < 200 || response.status >= 300) {
          return yield* Effect.fail(new Error(parseError(body)))
        }
      })

    const setProfile: AuthenticationApiService['setProfile'] = (
      orgId,
      roleId,
    ) =>
      Effect.gen(function* () {
        const response = yield* client.execute(
          HttpClientRequest.post('/api/auth/set-profile').pipe(
            HttpClientRequest.bodyUnsafeJson({
              org_id: orgId,
              role_id: roleId ?? undefined,
            }),
          ),
        )
        if (response.status < 200 || response.status >= 300) {
          const body = yield* response.json
          return yield* Effect.fail(new Error(parseError(body)))
        }
      })

    return { login, register, setProfile }
  }),
)

