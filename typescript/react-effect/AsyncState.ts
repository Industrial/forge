/**
 * Discriminated union for async operation state: idle, pending, success, or failure.
 *
 * @typeParam A - Success value type.
 * @typeParam E - Error type.
 */
export type AsyncState<A, E> =
  | { readonly _tag: 'idle' }
  | { readonly _tag: 'pending' }
  | { readonly _tag: 'success'; readonly value: A }
  | { readonly _tag: 'failure'; readonly error: E }

export const idle = <A, E>(): AsyncState<A, E> => ({ _tag: 'idle' })
export const pending = <A, E>(): AsyncState<A, E> => ({ _tag: 'pending' })
export const success = <A, E>(value: A): AsyncState<A, E> => ({
  _tag: 'success',
  value,
})
export const failure = <A, E>(error: E): AsyncState<A, E> => ({
  _tag: 'failure',
  error,
})

export function isIdle<A, E>(s: AsyncState<A, E>): s is { _tag: 'idle' } {
  return s._tag === 'idle'
}
export function isPending<A, E>(s: AsyncState<A, E>): s is { _tag: 'pending' } {
  return s._tag === 'pending'
}
export function isSuccess<A, E>(
  s: AsyncState<A, E>,
): s is { _tag: 'success'; value: A } {
  return s._tag === 'success'
}
export function isFailure<A, E>(
  s: AsyncState<A, E>,
): s is { _tag: 'failure'; error: E } {
  return s._tag === 'failure'
}
