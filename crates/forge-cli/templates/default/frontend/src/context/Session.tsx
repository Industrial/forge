/**
 * Session context: re-exports Auth as SessionProvider / useSession for compatibility.
 */
import { AuthProvider, useAuth } from "./Auth";

export const SessionProvider = AuthProvider;

export function useSession() {
	const auth = useAuth();
	return {
		...auth,
		refresh: auth.fetchMe,
	};
}
