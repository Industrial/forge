import { Effect, Runtime } from "effect";
import { useCallback, useRef, useState } from "react";
import { useEffectRuntime } from "./EffectRuntime";

/** Accepted argument for the setter returned by {@link useEffectState}. */
type SetStateAction<A, E, R> =
	| A
	| ((prev: A) => A)
	| Effect.Effect<A, E, R>;

/**
 * Mirrors React's `useState`, but the setter can accept a value, an updater
 * function, or an Effect.
 *
 * - **Value**: sets state directly (same as `useState`).
 * - **Updater** `(prev => next)`: same as React's functional update.
 * - **Effect**: run with the context runtime; on success state is set to the
 *   result; on failure `onError` is called if provided.
 *
 * Use this when some updates are effectful (e.g. fetch then set) and you want
 * a single, consistent setter API.
 *
 * @typeParam A - State value type.
 * @typeParam E - Error type when the setter receives an Effect (default: `unknown`).
 * @typeParam R - Requirements for the Effect (from context runtime).
 * @param initialState - Initial state or a lazy initializer `() => A`.
 * @param onError - Optional. Called when an Effect passed to the setter fails.
 * @returns Tuple `[state, setState]`; `setState` accepts `A`, `(prev => A)`, or `Effect<A, E, R>`.
 *
 * @example
 * ```tsx
 * const [user, setUser] = useEffectState<User | null>(null, (e) => toast.error(String(e)));
 *
 * setUser(await fetchUser(id));        // sync: setUser(value)
 * setUser(prev => prev ?? defaultUser); // updater
 * setUser(fetchUser(id));               // Effect: runs then sets on success, calls onError on failure
 * ```
 */
export function useEffectState<A, E = unknown, R = never>(
	initialState: A | (() => A),
	onError?: (error: E) => void,
): [A, (action: SetStateAction<A, E, R>) => void] {
	const { runtime } = useEffectRuntime<R>();
	const [state, setState] = useState<A>(initialState);
	const runCountRef = useRef(0);

	const set = useCallback(
		(action: SetStateAction<A, E, R>) => {
			if (Effect.isEffect(action)) {
				const runId = ++runCountRef.current;
				Runtime.runPromise(runtime)(action as Effect.Effect<A, E, R>)
					.then((value) => {
						if (runId === runCountRef.current) {
							setState(value);
						}
					})
					.catch((error) => {
						if (runId === runCountRef.current) {
							onError?.(error as E);
						}
					});
				return;
			}
			if (typeof action === "function") {
				setState((prev) => (action as (prev: A) => A)(prev));
				return;
			}
			setState(action as A);
		},
		[runtime, onError],
	);

	return [state, set];
}
