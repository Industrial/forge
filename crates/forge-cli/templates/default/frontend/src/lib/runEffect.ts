import { Effect } from 'effect'

/**
 * Run an Effect and return a Promise. Use in event handlers and useEffect
 * for API calls and side effects. Prefer this over raw fetch in components
 * so logic can be composed with Effect (map, flatMap, error handling).
 *
 * @example
 * const data = await runPromise(fetchUsersEffect);
 * setUsers(data);
 */
export function runPromise<A, E>(effect: Effect.Effect<A, E>): Promise<A> {
  return Effect.runPromise(effect)
}
