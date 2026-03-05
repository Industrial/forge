/**
 * Dashboard feature layer.
 *
 * Provides {@link HttpClient} (with auth: baseUrl and request-time auth/scope) and
 * {@link AuthenticationStore}. Call with config at app root (baseUrl).
 */
import { Layer } from 'effect'
import { AuthStateRefLayer } from '../../lib/authStateRef'
import {
  httpClientWithAuthLayer,
  type HttpClientWithAuthConfig,
} from '../../lib/httpClientWithAuth'
import { AuthenticationStoreLive } from '../authentication/services/AuthenticationStoreLive'

/**
 * Builds the dashboard feature layer: HttpClient (with auth) + AuthenticationStore.
 * Supply config when composing at app root; token/scope are read at request time from AuthStateRef.
 */
export const DashboardFeatureLayer = (config: HttpClientWithAuthConfig) => {
  const authRefLayer = AuthStateRefLayer
  const httpLayer = httpClientWithAuthLayer(config.baseUrl).pipe(
    Layer.provide(authRefLayer),
  )
  return Layer.merge(
    httpLayer,
    AuthenticationStoreLive.pipe(
      Layer.provide(httpLayer),
      Layer.provide(authRefLayer),
    ),
  )
}
