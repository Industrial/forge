/**
 * useAuth – auth state from AuthenticationStore via the Effect runtime.
 *
 * Run AuthenticationStore.getState() on mount and when refresh() is called;
 * state is stored in React. Use inside a tree that has EffectRuntimeProvider
 * with AuthenticationStore (e.g. AuthRuntimeProvider).
 */
import { Effect, Runtime } from "effect";
import { useCallback, useEffect, useState } from "react";
import type { AuthError, AuthStateSnapshot } from "../services/AuthenticationStore";
import { AuthenticationStore } from "../services/AuthenticationStore";
import { useEffectRuntime } from "../lib/react-effect";

const getStateEffect = Effect.gen(function* () {
	const store = yield* AuthenticationStore;
	return yield* store.getState();
});

export interface UseAuthResult {
	/** Current auth snapshot, or null before first load or on error. */
	state: AuthStateSnapshot | null;
	/** Re-run getState (e.g. after login, logout, setScope). */
	refresh: () => void;
	/** True while the getState effect is running. */
	isPending: boolean;
	/** Last error from getState, if any. */
	error: AuthError | null;
}

export function useAuth(): UseAuthResult {
	const { runtime } = useEffectRuntime();
	const [state, setState] = useState<AuthStateSnapshot | null>(null);
	const [isPending, setIsPending] = useState(true);
	const [error, setError] = useState<AuthError | null>(null);

	const runGetState = useCallback(() => {
		setIsPending(true);
		setError(null);
		Runtime.runPromise(runtime)(
			getStateEffect as Effect.Effect<AuthStateSnapshot, AuthError, never>,
		)
			.then((snapshot) => {
				setState(snapshot);
				setIsPending(false);
			})
			.catch((e) => {
				setError(e as AuthError);
				setState(null);
				setIsPending(false);
			});
	}, [runtime]);

	useEffect(() => {
		runGetState();
	}, [runGetState]);

	return {
		state,
		refresh: runGetState,
		isPending,
		error,
	};
}
