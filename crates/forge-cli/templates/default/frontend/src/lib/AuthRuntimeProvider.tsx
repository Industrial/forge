import { Effect, Layer, Runtime } from "effect";
import { type ReactNode, useEffect, useMemo, useState } from "react";
import { EffectRuntimeProvider } from "./react-effect";
import type { HttpClientWithAuthConfig } from "./httpClientWithAuth";
import { AuthenticationFeatureLayer } from "../features/authentication/layer";

function getInitialConfig(): HttpClientWithAuthConfig {
	if (typeof window === "undefined") {
		return { baseUrl: "" };
	}
	return {
		baseUrl: window.location.origin,
		token: localStorage.getItem("token") ?? undefined,
		organizationId: localStorage.getItem("currentOrgId") ?? undefined,
		roleId: localStorage.getItem("currentRoleId") ?? undefined,
	};
}

export interface AuthRuntimeProviderProps {
	/**
	 * Optional config for HttpClient + auth. If omitted, baseUrl and token/org/role
	 * are read from window.location and localStorage so the runtime can be built once.
	 * Pass config when you need to rebuild the runtime after login/scope change.
	 */
	readonly config?: HttpClientWithAuthConfig;
	readonly children: ReactNode;
}

/**
 * Builds the Effect runtime with AuthenticationFeatureLayer (HttpClient + AuthenticationStore)
 * and provides it via EffectRuntimeProvider. useAuth() and other effects that need
 * AuthenticationStore must run under this provider.
 */
export function AuthRuntimeProvider({
	config: configProp,
	children,
}: AuthRuntimeProviderProps) {
	const initialConfig = useMemo(getInitialConfig, []);
	const config = configProp ?? initialConfig;
	const [runtime, setRuntime] = useState<Runtime.Runtime<unknown> | null>(
		null,
	);

	useEffect(() => {
		const layer = AuthenticationFeatureLayer(config);
		const program = Effect.scoped(
			Effect.gen(function* () {
				return yield* Layer.toRuntime(layer);
			}),
		);
		Effect.runPromise(program).then((r) => setRuntime(r as Runtime.Runtime<unknown>));
	}, [
		config.baseUrl,
		config.token,
		config.organizationId,
		config.roleId,
	]);

	if (runtime == null) {
		return null;
	}
	return (
		<EffectRuntimeProvider runtime={runtime}>{children}</EffectRuntimeProvider>
	);
}
