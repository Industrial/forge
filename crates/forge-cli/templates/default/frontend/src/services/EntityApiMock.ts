import { Effect, Layer } from 'effect'
import type {
  EntityApiService,
  ListQueryParams,
  ListResponse,
} from './EntityApi'
import { EntityApi } from './EntityApi'

/**
 * Mock EntityApi for tests: no real HTTP; list returns empty data, get/create/update
 * return empty objects, delete succeeds.
 */
function makeEntityApiMock(): EntityApiService {
  return {
    list: (_entityId: string, _params?: ListQueryParams) =>
      Effect.succeed({ data: [] } satisfies ListResponse),

    get: (_entityId: string, _id: string) => Effect.succeed({}),

    create: (_entityId: string, _body: Record<string, unknown>) =>
      Effect.succeed({}),

    update: (
      _entityId: string,
      _id: string,
      _body: Record<string, unknown>,
    ) => Effect.succeed({}),

    delete: (_entityId: string, _id: string) => Effect.void,
  }
}

/**
 * Layer that provides a mock EntityApi for tests.
 * No dependencies; no real HTTP calls.
 */
export const EntityApiMock = Layer.succeed(EntityApi, makeEntityApiMock())
