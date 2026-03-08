import { Data } from 'effect'

/**
 * Error produced when login fails.
 *
 * @remarks
 * Tagged Data error for pattern matching. Used when login request fails
 * due to invalid credentials, server error, or network issues.
 */
export class LoginFailedError extends Data.TaggedError('LoginFailedError')<{
  readonly message: string
  readonly cause?: unknown
}> {}
