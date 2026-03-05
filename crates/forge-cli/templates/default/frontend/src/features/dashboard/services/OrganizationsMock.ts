/**
 * Mock Organizations service for tests.
 *
 * In-memory list; create/update/delete mutate the list. No HTTP.
 * Provide with Layer.succeed(Organizations, createOrganizationsMock()) or use
 * OrganizationsMockLayer for a pre-seeded list.
 */

import { Effect, Layer } from 'effect'
import { Organizations } from './Organizations'
import { Organization } from '../domain/Organization'
import type { OrganizationsService } from './Organizations'

function nowIso(): string {
  return new Date().toISOString()
}

function nextId(): string {
  return (
    crypto.randomUUID?.() ??
    `mock-${Date.now()}-${Math.random().toString(36).slice(2, 9)}`
  )
}

/**
 * Creates a mock Organizations service. Optionally pass initial organizations.
 */
export function createOrganizationsMock(
  initial: readonly Organization[] = [],
): OrganizationsService {
  const list: Organization[] = initial.map((o) =>
    o instanceof Organization ? o : new Organization(o),
  )

  return {
    list: () => Effect.succeed([...list] as readonly Organization[]),
    create: (body) =>
      Effect.sync(() => {
        const id = nextId()
        const now = nowIso()
        list.push(
          new Organization({
            id,
            name: body.name,
            slug: body.slug ?? body.name.toLowerCase().replace(/\s+/g, '-'),
            created_at: now,
            updated_at: now,
          }),
        )
      }),
    update: (body) =>
      Effect.gen(function* () {
        const idx = list.findIndex((o) => o.id === body.id)
        if (idx < 0) {
          return yield* Effect.fail(new Error('Organization not found.'))
        }
        const existing = list[idx]
        list[idx] = new Organization({
          id: existing.id,
          name: body.name ?? existing.name,
          slug: body.slug ?? existing.slug,
          created_at: existing.created_at,
          updated_at: nowIso(),
        })
      }),
    delete: (id) =>
      Effect.gen(function* () {
        const idx = list.findIndex((o) => o.id === id)
        if (idx < 0) {
          return yield* Effect.fail(new Error('Organization not found.'))
        }
        list.splice(idx, 1)
      }),
  }
}

/**
 * Layer that provides the Organizations service with a mock implementation.
 * Optionally pass initial organizations for tests.
 */
export const OrganizationsMockLayer = (initial: readonly Organization[] = []) =>
  Layer.succeed(Organizations, createOrganizationsMock(initial))
