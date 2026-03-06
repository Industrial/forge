/**
 * Authentication feature layer.
 *
 * Provides AuthStateRef (request-time auth/scope), {@link HttpClient} (with auth),
 * and {@link AuthenticationStore}. Scope and token are read at request time from AuthStateRef.
 */
import { AuthStateRefLayer } from '@/lib/authStateRef'
import { AuthenticationApiLive } from '@/features/authentication/services/AuthenticationApiLive'
import { AuthenticationStoreLive } from '@/features/authentication/services/AuthenticationStoreLive'
import { Layer } from 'effect'
import { getBaseUrl } from '@/lib/baseUrl'
import { httpClientWithAuthLayer } from '@/lib/httpClientWithAuth'

// TODO: Get this from a central location
const HttpLayer = httpClientWithAuthLayer(getBaseUrl()).pipe(
  Layer.provide(AuthStateRefLayer),
)

const AuthenticationStoreLayer = AuthenticationStoreLive.pipe(
  Layer.provide(HttpLayer),
  Layer.provide(AuthStateRefLayer),
)

const AuthenticationApiLayer = AuthenticationApiLive.pipe(
  Layer.provide(HttpLayer),
)

export const AuthenticationFeatureLayer = Layer.mergeAll(
  AuthStateRefLayer,
  HttpLayer,
  AuthenticationStoreLayer,
  AuthenticationApiLayer,
)
