import { Data } from 'effect'

/**
 * User identity returned from `/api/auth/me`.
 *
 * @remarks
 * Tagged Data class for structural equality. The token is the session bearer
 * token used for API and WebSocket auth.
 */
export class AuthenticationUser extends Data.TaggedClass('AuthenticationUser')<{
  readonly id: string
  readonly email: string
  readonly token: string
}> {}
