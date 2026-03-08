import { Data } from 'effect'

/**
 * Error produced when token is missing from response.
 *
 * @remarks
 * Tagged Data error for pattern matching. Used when authentication response
 * does not contain expected token.
 */
export class TokenMissingError extends Data.TaggedError('TokenMissingError')<{
  readonly message: string
}> {}
