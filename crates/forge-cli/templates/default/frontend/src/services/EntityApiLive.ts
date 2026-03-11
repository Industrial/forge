/**
 * Live implementation of EntityApi using HttpClient.
 *
 * Uses REST: GET/POST /api/entities/:entity_id, GET/PATCH/DELETE /api/entities/:entity_id/:id.
 * Requires HttpClient (with auth/scope headers); provide via AppLayer.
 */

import { HttpClientRequest } from '@effect/platform'
import { AuthenticatedHttpClient } from '../services/AuthenticatedHttpClient'
import { Effect, Layer } from 'effect'
import { parseError } from '../lib/parseError'
import type {
  EntityApiService,
  ListQueryParams,
  ListResponse,
} from './EntityApi'
import { EntityApi } from './EntityApi'

function buildListQuery(params?: ListQueryParams): string {
  if (!params) {
    return ''
  }
  const search = new URLSearchParams()
  if (params.filter != null && params.filter !== '') {
    search.set('filter', params.filter)
  }
  if (params.sort != null && params.sort !== '') {
    search.set('sort', params.sort)
  }
  if (params.order != null && params.order !== '') {
    search.set('order', params.order)
  }
  if (params.offset != null) {
    search.set('offset', String(params.offset))
  }
  if (params.limit != null) {
    search.set('limit', String(params.limit))
  }
  const q = search.toString()
  return q ? `?${q}` : ''
}

function toError(e: unknown): Error {
  return e instanceof Error ? e : new Error(String(e))
}

export const EntityApiLive = Layer.effect(
  EntityApi,
  Effect.gen(function* () {
    const client = yield* AuthenticatedHttpClient

    const list: EntityApiService['list'] = (entityId, params) =>
      Effect.gen(function* () {
        yield* Effect.logTrace('EntityApiLive.list')
        yield* Effect.logDebug(
          `EntityApiLive.list: entityId=${entityId}, params=${JSON.stringify(params ?? {})}`,
        )
        const path = `/api/entities/${encodeURIComponent(entityId)}${buildListQuery(params ?? undefined)}`
        const response = yield* client
          .execute(HttpClientRequest.get(path))
          .pipe(Effect.mapError(toError))
        const body = yield* response.json.pipe(Effect.mapError(toError))
        if (response.status >= 200 && response.status < 300) {
          const o = body as { data?: unknown[] }
          return {
            data: (o.data ?? []) as readonly unknown[],
          } satisfies ListResponse
        }
        return yield* Effect.fail(new Error(parseError(body)))
      })

    const get: EntityApiService['get'] = (entityId, id) =>
      Effect.gen(function* () {
        yield* Effect.logTrace('EntityApiLive.get')
        yield* Effect.logDebug(
          `EntityApiLive.get: entityId=${entityId}, id=${id}`,
        )
        const path = `/api/entities/${encodeURIComponent(entityId)}/${encodeURIComponent(id)}`
        const response = yield* client
          .execute(HttpClientRequest.get(path))
          .pipe(Effect.mapError(toError))
        const body = yield* response.json.pipe(Effect.mapError(toError))
        if (response.status >= 200 && response.status < 300) {
          return body
        }
        return yield* Effect.fail(new Error(parseError(body)))
      })

    const create: EntityApiService['create'] = (entityId, body) =>
      Effect.gen(function* () {
        yield* Effect.logTrace('EntityApiLive.create')
        yield* Effect.logDebug(`EntityApiLive.create: entityId=${entityId}`)
        const path = `/api/entities/${encodeURIComponent(entityId)}`
        const response = yield* client
          .execute(
            HttpClientRequest.post(path).pipe(
              HttpClientRequest.bodyUnsafeJson(body),
            ),
          )
          .pipe(Effect.mapError(toError))
        const resBody = yield* response.json.pipe(Effect.mapError(toError))
        if (response.status >= 200 && response.status < 300) {
          return resBody
        }
        return yield* Effect.fail(new Error(parseError(resBody)))
      })

    const update: EntityApiService['update'] = (entityId, id, body) =>
      Effect.gen(function* () {
        yield* Effect.logTrace('EntityApiLive.update')
        yield* Effect.logDebug(
          `EntityApiLive.update: entityId=${entityId}, id=${id}`,
        )
        const path = `/api/entities/${encodeURIComponent(entityId)}/${encodeURIComponent(id)}`
        const response = yield* client
          .execute(
            HttpClientRequest.patch(path).pipe(
              HttpClientRequest.bodyUnsafeJson(body),
            ),
          )
          .pipe(Effect.mapError(toError))
        const resBody = yield* response.json.pipe(Effect.mapError(toError))
        if (response.status >= 200 && response.status < 300) {
          return resBody
        }
        return yield* Effect.fail(new Error(parseError(resBody)))
      })

    const del: EntityApiService['delete'] = (entityId, id) =>
      Effect.gen(function* () {
        yield* Effect.logTrace('EntityApiLive.delete')
        yield* Effect.logDebug(
          `EntityApiLive.delete: entityId=${entityId}, id=${id}`,
        )
        const path = `/api/entities/${encodeURIComponent(entityId)}/${encodeURIComponent(id)}`
        const response = yield* client
          .execute(HttpClientRequest.del(path))
          .pipe(Effect.mapError(toError))
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
