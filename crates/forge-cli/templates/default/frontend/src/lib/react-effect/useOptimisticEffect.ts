import { Effect, Runtime } from "effect";
import { useCallback, useRef, useState } from "react";
import { useEffectRuntime } from "./EffectRuntime";

/**
 * Mirrors React's `useOptimistic`: optimistic state plus a setter that can
 * apply an update immediately and optionally run an Effect to "commit".
 *
 * When you call `setOptimistic(update, effect?)`: the UI shows `update` right
 * away (optimistic). If `effect` is provided, it is run; on success the
 * committed state is updated (and optionally merged via `reducer`); on failure
 * the optimistic state reverts to the previous committed value.
 *
 * @typeParam A - Type of the state value.
 * @typeParam E - Error type of the commit Effect.
 * @typeParam R - Requirements for the Effect (from context runtime).
 * @param initialValue - Initial committed (and optimistic) state.
 * @param reducer - Optional. When the Effect succeeds, `(currentCommitted, effectResult) => newCommitted`. If omitted, committed is replaced by the effect result.
 * @returns Tuple `[optimisticState, setOptimistic]`.
 *
 * @example
 * ```tsx
 * const [items, setOptimistic] = useOptimisticEffect(todos);
 *
 * const addItem = (todo: Todo) => {
 *   setOptimistic([...items, todo], api.addTodo(todo));
 * };
 * // UI shows new item immediately; on server error, list reverts.
 * ```
 *
 * @example
 * ```tsx
 * setOptimistic(prev => prev + 1); // sync update, no Effect
 * ```
 */
export function useOptimisticEffect<A, E, R = never>(
	initialValue: A,
	reducer?: (current: A, optimisticUpdate: A) => A,
): [
	optimisticState: A,
	setOptimistic: (
		update: A | ((current: A) => A),
		effect?: Effect.Effect<A, E, R>,
	) => void,
] {
	const { runtime } = useEffectRuntime<R>();
	const [committed, setCommitted] = useState<A>(initialValue);
	const [optimistic, setOptimisticState] = useState<A>(initialValue);
	const committedRef = useRef(initialValue);
	const runCountRef = useRef(0);

	committedRef.current = committed;

	const setOptimistic = useCallback(
		(
			update: A | ((current: A) => A),
			effect?: Effect.Effect<A, E, R>,
		) => {
			const nextOptimistic =
				typeof update === "function"
					? (update as (current: A) => A)(optimistic)
					: update;
			setOptimisticState(nextOptimistic);

			if (effect === undefined) {
				setCommitted(nextOptimistic);
				return;
			}

			const runId = ++runCountRef.current;
			const prevCommitted = committedRef.current;

			Runtime.runPromise(runtime)(effect)
				.then((value) => {
					if (runId === runCountRef.current) {
						const next = reducer
							? reducer(committedRef.current, value)
							: value;
						setCommitted(next);
						setOptimisticState(next);
					}
				})
				.catch(() => {
					if (runId === runCountRef.current) {
						setOptimisticState(prevCommitted);
					}
				});
		},
		[optimistic, reducer, runtime],
	);

	return [optimistic, setOptimistic];
}
