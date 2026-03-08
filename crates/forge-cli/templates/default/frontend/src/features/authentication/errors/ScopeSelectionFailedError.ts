import { Data } from 'effect'

/**
 * Error produced when scope selection fails.
 *
 * @remarks
 * Tagged Data error for pattern matching. Used when scope selection fails
 * due to invalid organization/role or API error.
 */
export class ScopeSelectionFailedError extends Data.TaggedError(
  'ScopeSelectionFailedError',
)<{
  readonly message: string
  readonly cause?: unknown
}> {}
