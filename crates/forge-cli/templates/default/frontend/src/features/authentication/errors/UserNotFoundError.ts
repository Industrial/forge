import { Data } from 'effect'

/**
 * Error produced when user is not found after successful login.
 *
 * @remarks
 * Tagged Data error for pattern matching. This is an unexpected state
 * that occurs when login succeeds but fetching user data fails.
 */
export class UserNotFoundError extends Data.TaggedError('UserNotFoundError')<{
  readonly message: string
}> {}
