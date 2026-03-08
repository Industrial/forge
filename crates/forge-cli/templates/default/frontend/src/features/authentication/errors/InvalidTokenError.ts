import { Data } from 'effect'

/**
 * Error produced when token is invalid or expired.
 *
 * @remarks
 * Tagged Data error for pattern matching. Used when token validation fails
 * or token is expired.
 */
export class InvalidTokenError extends Data.TaggedError('InvalidTokenError')<{
  readonly message: string
}> {}
