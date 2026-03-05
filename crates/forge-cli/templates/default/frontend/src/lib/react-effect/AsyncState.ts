/**
 * Discriminated union representing the state of an async Effect run.
 *
 * Used by hooks like {@link useActionStateEffect} to represent idle, in-flight,
 * success, and failure. Mirrors the pending/result/error idea of React's
 * `useTransition` and `useActionState`.
 *
 * @typeParam A - Success value type.
 * @typeParam E - Error type.
 *
 * @example
 * ```ts
 * const state: AsyncState<User, string> = success({ id: 1, name: "Alice" });
 * if (isSuccess(state)) {
 *   console.log(state.value.name);
 * }
 * ```
 */
export type AsyncState<A, E> =
  | { readonly _tag: 'idle' }
  | { readonly _tag: 'pending' }
  | { readonly _tag: 'success'; readonly value: A }
  | { readonly _tag: 'failure'; readonly error: E }

/**
 * Constructs an **idle** {@link AsyncState}: no run has been triggered yet.
 *
 * @returns `AsyncState<A, E>` with `_tag: "idle"`.
 */
export const idle = <A, E>(): AsyncState<A, E> => ({ _tag: 'idle' })

/**
 * Constructs a **pending** {@link AsyncState}: an Effect is currently running.
 *
 * @returns `AsyncState<A, E>` with `_tag: "pending"`.
 */
export const pending = <A, E>(): AsyncState<A, E> => ({ _tag: 'pending' })

/**
 * Constructs a **success** {@link AsyncState}: the Effect completed with a value.
 *
 * @param value - The success value.
 * @returns `AsyncState<A, E>` with `_tag: "success"` and `value`.
 */
export const success = <A, E>(value: A): AsyncState<A, E> => ({
  _tag: 'success',
  value,
})

/**
 * Constructs a **failure** {@link AsyncState}: the Effect failed with an error.
 *
 * @param error - The error value.
 * @returns `AsyncState<A, E>` with `_tag: "failure"` and `error`.
 */
export const failure = <A, E>(error: E): AsyncState<A, E> => ({
  _tag: 'failure',
  error,
})

/**
 * Type guard for **idle** {@link AsyncState}.
 *
 * @param s - The state to check.
 * @returns `true` if `s._tag === "idle"`.
 */
export function isIdle<A, E>(s: AsyncState<A, E>): s is { _tag: 'idle' } {
  return s._tag === 'idle'
}

/**
 * Type guard for **pending** {@link AsyncState}.
 *
 * @param s - The state to check.
 * @returns `true` if `s._tag === "pending"`.
 */
export function isPending<A, E>(s: AsyncState<A, E>): s is { _tag: 'pending' } {
  return s._tag === 'pending'
}

/**
 * Type guard for **success** {@link AsyncState}.
 *
 * @param s - The state to check.
 * @returns `true` if `s._tag === "success"`; narrows to `{ _tag: "success"; value: A }`.
 */
export function isSuccess<A, E>(
  s: AsyncState<A, E>,
): s is { _tag: 'success'; value: A } {
  return s._tag === 'success'
}

/**
 * Type guard for **failure** {@link AsyncState}.
 *
 * @param s - The state to check.
 * @returns `true` if `s._tag === "failure"`; narrows to `{ _tag: "failure"; error: E }`.
 */
export function isFailure<A, E>(
  s: AsyncState<A, E>,
): s is { _tag: 'failure'; error: E } {
  return s._tag === 'failure'
}
