import { FetchHttpClient, HttpClient, HttpClientRequest } from "@effect/platform";
import { Effect, Layer } from "effect";

/**
 * Config for the HttpClient layer: base URL and optional auth/scope headers.
 * Used at runtime construction (e.g. AppRuntimeProvider) so effects only depend on HttpClient.
 */
export interface HttpClientWithAuthConfig {
	/** Base URL for API requests (e.g. window.location.origin). */
	readonly baseUrl: string;
	readonly token?: string;
	readonly organizationId?: string;
	readonly roleId?: string;
}

/**
 * Layer that provides HttpClient (from @effect/platform) with baseUrl and auth headers
 * applied via mapRequest. Composes FetchHttpClient; no ApiClient service.
 */
export const httpClientWithAuthLayer = (config: HttpClientWithAuthConfig) =>
	Layer.effect(
		HttpClient.HttpClient,
		Effect.gen(function* () {
			const base = yield* HttpClient.HttpClient;
			return HttpClient.mapRequest(base, (req) => {
				let r = HttpClientRequest.prependUrl(req, config.baseUrl);
				if (config.token != null) {
					r = HttpClientRequest.setHeader(r, "Authorization", `Bearer ${config.token}`);
				}
				if (config.organizationId != null) {
					r = HttpClientRequest.setHeader(r, "X-Organization-Id", config.organizationId);
				}
				if (config.roleId != null) {
					r = HttpClientRequest.setHeader(r, "X-Role-Id", config.roleId);
				}
				return r;
			});
		})
	).pipe(Layer.provide(FetchHttpClient.layer));
