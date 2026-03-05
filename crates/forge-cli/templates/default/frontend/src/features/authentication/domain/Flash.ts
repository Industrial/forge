import { Data } from 'effect'

/**
 * Flash message from the auth API (e.g. success or error after login).
 *
 * @remarks
 * Optional `message` (success) or `error` (failure). Used to display one-off
 * feedback in the auth UI.
 */
export class Flash extends Data.TaggedClass('Flash')<{
  readonly message?: string
  readonly error?: string
}> {}
