import { Data } from 'effect'

/**
 * Error produced when registration fails.
 *
 * @remarks
 * Tagged Data error for pattern matching. Used when registration request fails
 * due to validation errors, server error, or network issues.
 */
export class RegistrationFailedError extends Data.TaggedError(
  'RegistrationFailedError',
)<{
  readonly message: string
  readonly cause?: unknown
}> {}
