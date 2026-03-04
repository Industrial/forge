/**
 * Authentication feature layer.
 *
 * Provides {@link AuthenticationStore} for the authentication feature. Requires
 * {@link HttpClient} to be supplied by the app (e.g. at runtime composition).
 * Compose with an HttpClient layer at the root to get a complete auth stack.
 */
import { AuthenticationStoreLive } from "../../services/AuthenticationStoreLive";

/**
 * Layer that provides AuthenticationStore for the authentication feature.
 * Depends on HttpClient (supply it when composing at app root).
 */
export const AuthenticationFeatureLayer = AuthenticationStoreLive;
