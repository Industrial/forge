/**
 * Authentication feature layer.
 *
 * Provides AuthStateRef (request-time auth/scope), {@link HttpClient} (with auth),
 * and {@link AuthenticationStore}. Scope and token are read at request time from AuthStateRef.
 */
import { Layer } from 'effect'
import { AuthStateRefLayer } from '../../lib/authStateRef'
import {
  httpClientWithAuthLayer,
  type HttpClientWithAuthConfig,
} from '../../lib/httpClientWithAuth'
import { AuthenticationApiLive } from './services/AuthenticationApiLive'
import { AuthenticationStoreLive } from './services/AuthenticationStoreLive'

/**
 * Builds the authentication feature layer: AuthStateRef, HttpClient (request-time auth/scope), AuthenticationStore, AuthenticationApi.
 * Supply config when composing at app root (baseUrl; token/scope are read at request time).
 */
export const AuthenticationFeatureLayer = (
  config: HttpClientWithAuthConfig,
) => {
  const authRefLayer = AuthStateRefLayer
  const httpLayer = httpClientWithAuthLayer(config.baseUrl).pipe(
    Layer.provide(authRefLayer),
  )
  const withHttp = Layer.merge(
    httpLayer,
    AuthenticationStoreLive.pipe(
      Layer.provide(httpLayer),
      Layer.provide(authRefLayer),
    ),
  )
  return Layer.mergeAll(
    authRefLayer,
    withHttp,
    AuthenticationApiLive.pipe(Layer.provide(httpLayer)),
  )
}
