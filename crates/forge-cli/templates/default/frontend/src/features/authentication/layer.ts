/**
 * Authentication feature layer.
 *
 * Provides everything the authentication feature requires: {@link HttpClient}
 * (with auth) and {@link AuthenticationStore}. Call with config at app root
 * to build the layer; the feature provides what it needs.
 */
import { Layer } from 'effect'
import {
  httpClientWithAuthLayer,
  type HttpClientWithAuthConfig,
} from '../../lib/httpClientWithAuth'
import { AuthenticationStoreLive } from '../../services/AuthenticationStoreLive'

/**
 * Builds the authentication feature layer: HttpClient (with auth) + AuthenticationStore.
 * Supply config when composing at app root (e.g. baseUrl and token/org/role).
 */
export const AuthenticationFeatureLayer = (
  config: HttpClientWithAuthConfig,
) => {
  const httpLayer = httpClientWithAuthLayer(config)
  return Layer.merge(
    httpLayer,
    AuthenticationStoreLive.pipe(Layer.provide(httpLayer)),
  )
}
