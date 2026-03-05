/**
 * Organizations service – backend operations for the organizations page.
 *
 * Provides list, create, update, and delete via the REST API (/api/organizations).
 * All methods return `Effect<A, Error, never>` (no requirement leakage); the Live
 * implementation uses HttpClient.
 *
 * **State:** This service does not hold state. It only performs requests. Manage
 * state in React (e.g. `useState` + `runWithAppRuntime`, or a custom hook like
 * `useOrganizations()` that calls this service and exposes `{ organizations, loading, error, refresh }`).
 *
 * @see OrganizationsLive – implementation using HttpClient
 * @see OrganizationsMock – test double with in-memory list
 */

import { Context, Effect } from 'effect'
import type { Organization } from '../domain/Organization'

/**
 * Organizations service interface.
 *
 * Backend operations for the organizations page. Use from Effects (yield* Organizations)
 * or from React via the app runtime (runWithAppRuntime(runtime, orgs.list()) etc.).
 */
export interface OrganizationsService {
  /** List all organizations. GET /api/organizations. */
  readonly list: () => Effect.Effect<readonly Organization[], Error, never>
  /** Create an organization. POST /api/organizations. */
  readonly create: (body: {
    name: string
    slug?: string
  }) => Effect.Effect<void, Error, never>
  /** Update an organization. PATCH /api/organizations/:id. */
  readonly update: (body: {
    id: string
    name?: string
    slug?: string
  }) => Effect.Effect<void, Error, never>
  /** Delete an organization. DELETE /api/organizations/:id. */
  readonly delete: (id: string) => Effect.Effect<void, Error, never>
}

/**
 * Tag for the Organizations service.
 *
 * Use in Effects: `yield* Organizations` then `yield* orgs.list()`.
 * Provide with OrganizationsLive (requires HttpClient).
 */
export const Organizations = Context.GenericTag<OrganizationsService>(
  'dashboard/Organizations',
)
