import { useAuth } from "../context/Auth";

export function useApi() {
	const { token, currentOrgId, currentRoleId } = useAuth();

	const api = async (url: string, options?: RequestInit) => {
		const headers = new Headers(options?.headers);
		if (token) {
			headers.set("Authorization", `Bearer ${token}`);
		}
		if (currentOrgId) {
			headers.set("X-Organization-Id", currentOrgId);
		}
		if (currentRoleId) {
			headers.set("X-Role-Id", currentRoleId);
		}

		const res = await fetch(url, { ...options, headers });

		if (res.status === 401) {
			// TODO: Handle token expiration / unauthorized
			console.log("Unauthorized API call");
		}

		return res;
	};

	return api;
}
