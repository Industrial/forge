/**
 * Live implementation of EntityApi using HttpClient.
 *
 * Uses REST: GET/POST /api/entities/:entity_id, GET/PATCH/DELETE /api/entities/:entity_id/:id.
 * Requires HttpClient (with auth/scope headers); provide via AppLayer.
 */

import { HttpClient, HttpClientRequest } from '@effect/platform'
import { Effect, Layer } from 'effect'
import type {
  EntityApiService,
  ListQueryParams,
  ListResponse,
} from './EntityApi'
import { EntityApi } from './EntityApi'

function buildListQuery(params?: ListQueryParams): string {
  if (!params) return ''
  const search = new URLSearchParams()
  if (params.filter != null && params.filter !== '')
    search.set('filter', params.filter)
  if (params.sort != null && params.sort !== '') search.set('sort', params.sort)
  if (params.order != null && params.order !== '')
    search.set('order', params.order)
  if (params.offset != null) search.set('offset', String(params.offset))
  if (params.limit != null) search.set('limit', String(params.limit))
  const q = search.toString()
  return q ? `?${q}` : ''
}

function parseError(body: unknown): string {
  if (
    typeof body === 'object' &&
    body !== null &&
    'message' in body &&
    typeof (body as { message: unknown }).message === 'string'
  ) {
    return (body as { message: string }).message
  }
  if (
    typeof body === 'object' &&
    body !== null &&
    'error' in body &&
    typeof (body as { error: unknown }).error === 'string'
  ) {
    return (body as { error: string }).error
  }
  return 'Request failed.'
}

function toError(e: unknown): Error {
  return e instanceof Error ? e : new Error(String(e))
}

const EntityApiLive = Layer.effect(
  EntityApi,
  Effect.gen(function* () {
    const client = yield* HttpClient.HttpClient

    const list: EntityApiService['list'] = (entityId, params) =>
      Effect.gen(function* () {
        const path = `/api/entities/${encodeURIComponent(entityId)}${buildListQuery(params ?? undefined)}`
        const response = yield* client.execute(HttpClientRequest.get(path)).pipe(
          Effect.mapError(toError),
        )
        const body = yield* response.json.pipe(Effect.mapError(toError))
        if (response.status >= 200 && response.status < 300) {
          const o = body as { data?: unknown[] }
          return { data: (o.data ?? []) as readonly unknown[] } satisfies ListResponse
        }
        return yield* Effect.fail(new Error(parseError(body)))
      })

    const get: EntityApiService['get'] = (entityId, id) =>
      Effect.gen(function* () {
        const path = `/api/entities/${encodeURIComponent(entityId)}/${encodeURIComponent(id)}`
        const response = yield* client.execute(HttpClientRequest.get(path)).pipe(
          Effect.mapError(toError),
        )
        const body = yield* response.json.pipe(Effect.mapError(toError))
        if (response.status >= 200 && response.status < 300) return body
        return yield* Effect.fail(new Error(parseError(body)))
      })

    const create: EntityApiService['create'] = (entityId, body) =>
      Effect.gen(function* () {
        const path = `/api/entities/${encodeURIComponent(entityId)}`
        const response = yield* client
          .execute(
            HttpClientRequest.post(path).pipe(
              HttpClientRequest.bodyUnsafeJson(body),
            ),
          )
          .pipe(Effect.mapError(toError))
        const resBody = yield* response.json.pipe(Effect.mapError(toError))
        if (response.status >= 200 && response.status < 300) return resBody
        return yield* Effect.fail(new Error(parseError(resBody)))
      })

    const update: EntityApiService['update'] = (entityId, id, body) =>
      Effect.gen(function* () {
        const path = `/api/entities/${encodeURIComponent(entityId)}/${encodeURIComponent(id)}`
        const response = yield* client
          .execute(
            HttpClientRequest.patch(path).pipe(
              HttpClientRequest.bodyUnsafeJson(body),
            ),
          )
          .pipe(Effect.mapError(toError))
        const resBody = yield* response.json.pipe(Effect.mapError(toError))
        if (response.status >= 200 && response.status < 300) return resBody
        return yield* Effect.fail(new Error(parseError(resBody)))
      })

    const del: EntityApiService['delete'] = (entityId, id) =>
      Effect.gen(function* () {
        const path = `/api/entities/${encodeURIComponent(entityId)}/${encodeURIComponent(id)}`
        const response = yield* client.execute(HttpClientRequest.del(path)).pipe(
          Effect.mapError(toError),
        )
        const body = yield* response.json.pipe(Effect.mapError(toError))
        if (response.status >= 200 && response.status < 300) {
          return undefined
        }
        return yield* Effect.fail(new Error(parseError(body)))
      })

    return {
      list,
      get,
      create,
      update,
      delete: del,
    }
  }),
)

export { EntityApiLive }
