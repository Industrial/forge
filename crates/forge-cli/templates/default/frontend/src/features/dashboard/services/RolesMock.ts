/**
 * Mock Roles service for tests.
 */

import { Effect, Layer } from 'effect'
import { Roles } from './Roles'
import type { RolesService } from './Roles'
import { Role } from '../domain/Role'
import { DashboardRole } from '../domain/DashboardRole'

function nextId(): string {
  return (
    crypto.randomUUID?.() ??
    `mock-${Date.now()}-${Math.random().toString(36).slice(2, 9)}`
  )
}

/**
 * Creates a mock Roles service. Optionally pass initial roles.
 */
export function createRolesMock(
  initial: readonly Role[] = [],
): RolesService {
  const roles: Role[] = initial.map((r) =>
    r instanceof Role ? r : new Role(r),
  )

  return {
    list: () => Effect.succeed([...roles] as readonly Role[]),

    listByOrg: (orgId: string) =>
      Effect.sync(() => {
        const filtered = roles.filter((r) => r.org_id === orgId)
        return filtered.map(
          (r) =>
            new DashboardRole({
              id: r.id,
              name: r.name,
              display_name: r.display_name,
            }),
        ) as readonly DashboardRole[]
      }),

    create: (body) =>
      Effect.sync(() => {
        roles.push(
          new Role({
            id: nextId(),
            org_id: body.org_id,
            name: body.name,
            display_name: body.display_name ?? null,
          }),
        )
      }),

    update: (body) =>
      Effect.gen(function* () {
        const idx = roles.findIndex((r) => r.id === body.id)
        if (idx < 0) {
          return yield* Effect.fail(new Error('Role not found.'))
        }
        const existing = roles[idx]
        roles[idx] = new Role({
          id: existing.id,
          org_id: existing.org_id,
          name: body.name ?? existing.name,
          display_name: body.display_name ?? existing.display_name,
          created_at: existing.created_at,
          updated_at: existing.updated_at,
        })
      }),

    delete: (id: string) =>
      Effect.gen(function* () {
        const idx = roles.findIndex((r) => r.id === id)
        if (idx < 0) {
          return yield* Effect.fail(new Error('Role not found.'))
        }
        roles.splice(idx, 1)
      }),
  }
}

export const RolesMockLayer = (initial: readonly Role[] = []) =>
  Layer.succeed(Roles, createRolesMock(initial))
