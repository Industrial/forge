import { Data } from 'effect'

/**
 * Error produced when scope selection (ReBAC) fails.
 *
 * @remarks
 * Tagged Data error for pattern matching. Used when selectScope fails
 * (e.g. invalid organization/role or API error).
 */
export class ScopeError extends Data.TaggedError('ScopeError')<{
  readonly message: string
  readonly cause?: unknown
}> {}
