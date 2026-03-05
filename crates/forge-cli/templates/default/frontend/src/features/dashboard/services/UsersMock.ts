/**
 * Mock Users service for tests.
 */

import { Effect, Layer } from 'effect'
import { Users } from './Users'
import type { UsersService } from './Users'
import { User } from '../domain/User'

function nextId(): string {
  return (
    crypto.randomUUID?.() ??
    `mock-${Date.now()}-${Math.random().toString(36).slice(2, 9)}`
  )
}

/**
 * Creates a mock Users service. Optionally pass initial users.
 */
export function createUsersMock(initial: readonly User[] = []): UsersService {
  const users: User[] = initial.map((u) =>
    u instanceof User ? u : new User(u),
  )

  return {
    list: () => Effect.succeed([...users] as readonly User[]),

    create: (body) =>
      Effect.sync(() => {
        users.push(
          new User({
            id: nextId(),
            email: body.email,
            is_active: true,
            is_admin: false,
            created_at: new Date().toISOString(),
            memberships: [],
          }),
        )
      }),

    update: (body) =>
      Effect.gen(function* () {
        const idx = users.findIndex((u) => u.id === body.id)
        if (idx < 0) {
          return yield* Effect.fail(new Error('User not found.'))
        }
        const existing = users[idx]
        users[idx] = new User({
          id: existing.id,
          email: body.email ?? existing.email,
          is_active: body.is_active ?? existing.is_active,
          is_admin: existing.is_admin,
          created_at: existing.created_at,
          memberships: existing.memberships,
        })
      }),

    delete: (id: string) =>
      Effect.gen(function* () {
        const idx = users.findIndex((u) => u.id === id)
        if (idx < 0) {
          return yield* Effect.fail(new Error('User not found.'))
        }
        users.splice(idx, 1)
      }),
  }
}

export const UsersMockLayer = (initial: readonly User[] = []) =>
  Layer.succeed(Users, createUsersMock(initial))
