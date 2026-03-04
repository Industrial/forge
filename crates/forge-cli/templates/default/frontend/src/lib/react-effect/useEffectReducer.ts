import { Effect, Runtime } from "effect";
import { useCallback, useRef, useState } from "react";
import { useEffectRuntime } from "./EffectRuntime";

/**
 * Mirrors React's `useReducer`, but the reducer returns an Effect.
 *
 * When you dispatch an action, the reducer is called with the current state
 * (from a ref, so it's always up to date) and the action; it returns
 * `Effect<NextState, E, R>`. That Effect is run with the context runtime;
 * on success state is updated, on failure `error` is set and `onError` is
 * called if provided. `isPending` is true while the Effect is running.
 *
 * @typeParam S - State type.
 * @typeParam A - Action type (reducer second argument).
 * @typeParam E - Error type of the Effect (default: `unknown`).
 * @typeParam R - Requirements for the Effect (from context runtime).
 * @param reducer - `(state, action) => Effect<S, E, R>`. Receives latest state.
 * @param initialState - Initial state value.
 * @param onError - Optional. Called when the Effect fails.
 * @returns Tuple `[state, dispatch, isPending, error]`.
 *
 * @example
 * ```tsx
 * const [state, dispatch, isPending, error] = useEffectReducer(
 *   (state, action) =>
 *     action.type === "fetch"
 *       ? api.getUser(action.id).pipe(Effect.map((user) => ({ ...state, user })))
 *       : Effect.succeed({ ...state, user: action.user }),
 *   { user: null },
 *   (e) => toast.error(String(e))
 * );
 *
 * dispatch({ type: "fetch", id: 1 });
 * ```
 */
export function useEffectReducer<S, A, E = unknown, R = never>(
	reducer: (state: S, action: A) => Effect.Effect<S, E, R>,
	initialState: S,
	onError?: (error: E) => void,
): [state: S, dispatch: (action: A) => void, isPending: boolean, error: E | null] {
	const { runtime } = useEffectRuntime<R>();
	const [state, setState] = useState<S>(initialState);
	const stateRef = useRef(state);
	stateRef.current = state;
	const [error, setError] = useState<E | null>(null);
	const [isPending, setIsPending] = useState(false);
	const runCountRef = useRef(0);

	const dispatch = useCallback(
		(action: A) => {
			const runId = ++runCountRef.current;
			setIsPending(true);
			setError(null);

			const eff = reducer(stateRef.current, action);
			Runtime.runPromise(runtime)(eff)
				.then((nextState) => {
					if (runId === runCountRef.current) {
						setState(nextState);
						setError(null);
					}
				})
				.catch((err) => {
					if (runId === runCountRef.current) {
						setError(err as E);
						onError?.(err as E);
					}
				})
				.finally(() => {
					if (runId === runCountRef.current) {
						setIsPending(false);
					}
				});
		},
		[reducer, runtime, onError],
	);

	return [state, dispatch, isPending, error];
}
