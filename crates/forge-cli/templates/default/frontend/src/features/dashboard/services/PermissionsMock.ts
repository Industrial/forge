/**
 * Mock Permissions service for tests.
 */

import { Effect, Layer } from 'effect'
import { Permissions } from './Permissions'
import type { PermissionsData, PermissionsService } from './Permissions'
import { Assignment } from '../domain/Assignment'

/**
 * Creates a mock Permissions service. Optionally pass initial data.
 */
export function createPermissionsMock(
  initial: {
    assignments?: readonly Assignment[]
    permissions?: readonly string[]
  } = {},
): PermissionsService {
  const assignments: Assignment[] = (initial.assignments ?? []).map((a) =>
    a instanceof Assignment ? a : new Assignment(a),
  )
  const permissions: string[] = [...(initial.permissions ?? [])]

  return {
    getData: () =>
      Effect.succeed({
        assignments: [...assignments] as readonly Assignment[],
        permissions: [...permissions],
      } satisfies PermissionsData),

    add: (body) =>
      Effect.sync(() => {
        assignments.push(
          new Assignment({
            scope: body.scope,
            role_name: body.role_name,
            permission_key: body.permission_key,
          }),
        )
      }),

    delete: (body) =>
      Effect.gen(function* () {
        const idx = assignments.findIndex(
          (a) =>
            a.scope === body.scope &&
            a.role_name === body.role_name &&
            a.permission_key === body.permission_key &&
            (a.org_id ?? null) === (body.org_id ?? null),
        )
        if (idx < 0) {
          return yield* Effect.fail(new Error('Assignment not found.'))
        }
        assignments.splice(idx, 1)
      }),
  }
}

export const PermissionsMockLayer = (
  initial: {
    assignments?: readonly Assignment[]
    permissions?: readonly string[]
  } = {},
) => Layer.succeed(Permissions, createPermissionsMock(initial))
