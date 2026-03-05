/**
 * Live implementation of Organizations service using HttpClient.
 *
 * Uses REST API: GET/POST /api/organizations, PATCH/DELETE /api/organizations/:id.
 * Requires HttpClient (with auth headers); provide via DashboardFeatureLayer or AppLayer.
 */

import { HttpClient, HttpClientRequest } from '@effect/platform'
import { Effect, Layer } from 'effect'
import { Organization, Organizations } from './Organizations'
import type { OrganizationsService } from './Organizations'

function parseError(body: unknown): string {
  if (typeof body === 'object' && body !== null && 'error' in body) {
    return String((body as { error: unknown }).error)
  }
  return 'Request failed.'
}

const OrganizationsLive = Layer.effect(
  Organizations,
  Effect.gen(function* () {
    const client = yield* HttpClient.HttpClient

    const list: OrganizationsService['list'] = () =>
      Effect.gen(function* () {
        const response = yield* client.execute(
          HttpClientRequest.get('/api/organizations'),
        )
        const body = yield* response.json
        if (response.status < 200 || response.status >= 300) {
          return yield* Effect.fail(
            new Error(
              typeof body === 'object' && body !== null && 'error' in body
                ? String((body as { error: unknown }).error)
                : `HTTP ${response.status}`,
            ),
          )
        }
        const data = body as {
          organizations?: Array<{
            id: string
            name: string
            slug: string
            created_at?: string
            updated_at?: string
          }>
        }
        const raw = data.organizations ?? []
        return raw.map(
          (o) =>
            new Organization({
              id: o.id,
              name: o.name,
              slug: o.slug,
              created_at: o.created_at,
              updated_at: o.updated_at,
            }),
        ) as readonly Organization[]
      })

    const create: OrganizationsService['create'] = (body) =>
      Effect.gen(function* () {
        const response = yield* client.execute(
          HttpClientRequest.post('/api/organizations').pipe(
            HttpClientRequest.bodyUnsafeJson(body),
          ),
        )
        const resBody = yield* response.json
        if (response.status === 403) {
          return yield* Effect.fail(
            new Error('You do not have permission to create organizations.'),
          )
        }
        if (response.status < 200 || response.status >= 300) {
          return yield* Effect.fail(new Error(parseError(resBody)))
        }
      })

    const update: OrganizationsService['update'] = (body) =>
      Effect.gen(function* () {
        const { id, ...rest } = body
        const response = yield* client.execute(
          HttpClientRequest.patch(
            `/api/organizations/${encodeURIComponent(id)}`,
          ).pipe(HttpClientRequest.bodyUnsafeJson(rest)),
        )
        const resBody = yield* response.json
        if (response.status === 403) {
          return yield* Effect.fail(
            new Error('You do not have permission to update organizations.'),
          )
        }
        if (response.status === 404) {
          return yield* Effect.fail(new Error('Organization not found.'))
        }
        if (response.status < 200 || response.status >= 300) {
          return yield* Effect.fail(new Error(parseError(resBody)))
        }
      })

    const del: OrganizationsService['delete'] = (id) =>
      Effect.gen(function* () {
        const response = yield* client.execute(
          HttpClientRequest.del(`/api/organizations/${encodeURIComponent(id)}`),
        )
        const resBody = yield* response.json
        if (response.status === 403) {
          return yield* Effect.fail(
            new Error('You do not have permission to delete organizations.'),
          )
        }
        if (response.status === 404) {
          return yield* Effect.fail(new Error('Organization not found.'))
        }
        if (response.status < 200 || response.status >= 300) {
          return yield* Effect.fail(new Error(parseError(resBody)))
        }
      })

    return {
      list,
      create,
      update,
      delete: del,
    }
  }),
)

export { OrganizationsLive }
