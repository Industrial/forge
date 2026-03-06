/**
 * Generic entity REST API service.
 *
 * Calls GET/POST /api/entities/:entity_id and GET/PATCH/DELETE /api/entities/:entity_id/:id
 * with query spec (filter, sort, pagination). All methods return Effect<A, Error, never>;
 * Live implementation uses HttpClient (with auth/scope headers).
 *
 * @see EntityApiLive – implementation using HttpClient
 * @see docs/frontend-implementation-and-choices.md (Epic 5)
 */

import { Context, Effect } from 'effect'

/** Query params for list: filter (JSON string), sort, order, offset, limit. */
export interface ListQueryParams {
  readonly filter?: string
  readonly sort?: string
  readonly order?: string
  readonly offset?: number
  readonly limit?: number
}

/** List response shape: { data: T[] }. Backend returns JSON; items are untyped. */
export interface ListResponse<T = unknown> {
  readonly data: readonly T[]
}

/**
 * Entity API service interface.
 *
 * Generic CRUD over /api/entities/:entity_id. Use from Effects (yield* EntityApi)
 * or from React via the app runtime.
 */
export interface EntityApiService {
  /** List entities. GET /api/entities/:entity_id?filter=&sort=&order=&offset=&limit= */
  readonly list: (
    entityId: string,
    params?: ListQueryParams,
  ) => Effect.Effect<ListResponse, Error, never>
  /** Get one entity. GET /api/entities/:entity_id/:id */
  readonly get: (
    entityId: string,
    id: string,
  ) => Effect.Effect<unknown, Error, never>
  /** Create entity. POST /api/entities/:entity_id. Body must be a JSON object. */
  readonly create: (
    entityId: string,
    body: Record<string, unknown>,
  ) => Effect.Effect<unknown, Error, never>
  /** Update entity. PATCH /api/entities/:entity_id/:id */
  readonly update: (
    entityId: string,
    id: string,
    body: Record<string, unknown>,
  ) => Effect.Effect<unknown, Error, never>
  /** Delete entity. DELETE /api/entities/:entity_id/:id */
  readonly delete: (
    entityId: string,
    id: string,
  ) => Effect.Effect<void, Error, never>
}

/**
 * Tag for the EntityApi service.
 *
 * Use in Effects: yield* EntityApi then yield* api.list('organization', { limit: 10 }).
 * Provide with EntityApiLive (requires HttpClient).
 */
export const EntityApi =
  Context.GenericTag<EntityApiService>('@forge/EntityApi')
