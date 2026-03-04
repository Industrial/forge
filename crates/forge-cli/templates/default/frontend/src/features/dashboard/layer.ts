/**
 * Dashboard feature layer.
 *
 * Provides {@link HttpClient} (with auth: baseUrl and auth headers) and
 * {@link AuthenticationStore} for dashboard and users. Call with config at app
 * root (e.g. baseUrl and token/org/role from auth state) to build the layer.
 */
import { Layer } from "effect";
import {
	httpClientWithAuthLayer,
	type HttpClientWithAuthConfig,
} from "../../lib/httpClientWithAuth";
import { AuthenticationStoreLive } from "../../services/AuthenticationStoreLive";

/**
 * Builds the dashboard feature layer: HttpClient (with auth) + AuthenticationStore.
 * Supply config when composing at app root so the HTTP client has baseUrl and auth headers.
 */
export const DashboardFeatureLayer = (config: HttpClientWithAuthConfig) => {
	const httpLayer = httpClientWithAuthLayer(config);
	return Layer.merge(
		httpLayer,
		AuthenticationStoreLive.pipe(Layer.provide(httpLayer)),
	);
};
