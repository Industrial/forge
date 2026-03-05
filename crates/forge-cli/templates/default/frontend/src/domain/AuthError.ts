import { Data } from 'effect'

/**
 * Error produced by AuthenticationStore operations.
 *
 * @remarks
 * Tagged Data error for pattern matching. Typically used when fetchMe fails
 * (e.g. network or invalid token) or when a method is used in an invalid state.
 */
export class AuthError extends Data.TaggedError('AuthError')<{
  readonly message: string
  readonly cause?: unknown
}> {}
