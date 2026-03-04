/**
 * Session context: re-exports Authentication as SessionProvider / useSession for compatibility.
 */
import {
	AuthenticationProvider,
	useAuthentication,
} from "./AuthenticationContext";

export const SessionProvider = AuthenticationProvider;

export function useSession() {
	const auth = useAuthentication();
	return {
		...auth,
		refresh: auth.fetchMe,
	};
}
