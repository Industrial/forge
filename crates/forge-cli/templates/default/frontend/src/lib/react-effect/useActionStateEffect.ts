import { Effect, Runtime } from "effect";
import { useCallback, useRef, useState } from "react";
import { useEffectRuntime } from "./EffectRuntime";
import type { AsyncState } from "./AsyncState";
import { failure, idle, pending, success } from "./AsyncState";

/**
 * Mirrors React's `useActionState`: state for an "action" (Effect), a dispatch
 * to run it, and a pending flag.
 *
 * The action is a function that receives the current {@link AsyncState} and
 * returns an Effect. When you call `dispatchAction()`, that Effect is run
 * with the context runtime; the state moves to `pending`, then to `success(value)`
 * or `failure(error)`. The reducer always sees the latest state via an internal ref.
 *
 * @typeParam A - Success value type (and state on success).
 * @typeParam E - Error type (and state on failure).
 * @typeParam R - Requirements for the Effect (from context runtime).
 * @param action - Function from current state to an Effect. Receives latest {@link AsyncState}; run is fire-and-forget from the caller's perspective.
 * @param initialState - Initial state (default: {@link idle}).
 * @returns Tuple `[state, dispatchAction, isPending]`.
 *
 * @example
 * ```tsx
 * const [state, dispatchAction, isPending] = useActionStateEffect(
 *   () => api.updateProfile(formData),
 *   idle()
 * );
 *
 * return (
 *   <>
 *     <button onClick={dispatchAction} disabled={isPending}>Save</button>
 *     {isSuccess(state) && <p>Saved: {state.value.name}</p>}
 *     {isFailure(state) && <p>Error: {state.error}</p>}
 *   </>
 * );
 * ```
 */
export function useActionStateEffect<A, E, R = never>(
	action: (prev: AsyncState<A, E>) => Effect.Effect<A, E, R>,
	initialState: AsyncState<A, E> = idle<A, E>(),
): [
	state: AsyncState<A, E>,
	dispatchAction: () => void,
	isPending: boolean,
] {
	const { runtime } = useEffectRuntime<R>();
	const [state, setState] = useState<AsyncState<A, E>>(initialState);
	const stateRef = useRef(state);
	stateRef.current = state;
	const runCountRef = useRef(0);

	const dispatchAction = useCallback(() => {
		const runId = ++runCountRef.current;
		setState(pending<A, E>());

		const eff = action(stateRef.current);
		Runtime.runPromise(runtime)(eff)
			.then((value) => {
				if (runId === runCountRef.current) {
					setState(success(value));
				}
			})
			.catch((error) => {
				if (runId === runCountRef.current) {
					setState(failure(error));
				}
			});
	}, [action, runtime]);

	const isPending = state._tag === "pending";

	return [state, dispatchAction, isPending];
}
