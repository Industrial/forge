import { Effect, Exit, Runtime, Scope } from "effect";
import { useEffect, useRef } from "react";
import { useEffectRuntime } from "./EffectRuntime";

/**
 * Runs an Effect when the component mounts and when dependencies change.
 *
 * The Effect is run in a Scope; when the component unmounts or when `deps`
 * change, the Scope is closed. That interrupts the running Effect and runs
 * any finalizers (e.g. closing connections, clearing timers). Mirrors React's
 * `useEffect(setup, deps)` with the setup expressed as an Effect and cleanup
 * handled via Scope.
 *
 * @param effect - The Effect to run. It will be forked in a Scope so it can be interrupted on cleanup.
 * @param deps - Dependency list (same semantics as `useEffect`). When any value changes, the previous run is cleaned up and the effect runs again.
 *
 * @example
 * ```tsx
 * useRunEffect(
 *   Effect.gen(function* () {
 *     const conn = yield* connectToServer(roomId);
 *     yield* Effect.addFinalizer(() => conn.disconnect());
 *     yield* subscribe(conn);
 *   }),
 *   [roomId]
 * );
 * ```
 */
export function useRunEffect<A, E, R>(
	effect: Effect.Effect<A, E, R>,
	deps: React.DependencyList,
): void {
	const { runtime } = useEffectRuntime<R>();
	type ScopeRef = Parameters<typeof Scope.close>[0];
	const scopeRef = useRef<ScopeRef | null>(null);
	const isMountedRef = useRef(true);

	useEffect(() => {
		isMountedRef.current = true;
		let cancelled = false;

		const run = async () => {
			try {
				const scoped = Effect.gen(function* () {
					const scope = yield* Scope.make();
					yield* Effect.fork(Scope.extend(effect, scope));
					return scope;
				});
				const scope = await Runtime.runPromise(runtime)(
					scoped as Effect.Effect<Scope.CloseableScope, never, R>,
				);
				if (!cancelled && isMountedRef.current) {
					scopeRef.current = scope;
				} else {
					Runtime.runPromise(runtime)(Scope.close(scope, Exit.void)).catch(
						() => {},
					);
				}
			} catch (_) {
				// Effect failed or was interrupted; ignore for fire-and-forget
			}
		};

		run();

		return () => {
			cancelled = true;
			isMountedRef.current = false;
			const scope = scopeRef.current;
			scopeRef.current = null;
			if (scope) {
				Runtime.runPromise(runtime)(Scope.close(scope, Exit.void)).catch(
					() => {},
				);
			}
		};
	}, deps);
}
