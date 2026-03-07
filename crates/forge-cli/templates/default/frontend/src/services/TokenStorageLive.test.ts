/**
 * BDD tests for TokenStorageLive (localStorage-backed TokenStorage).
 * Tests use bun:test with Given/When/Then structure and Effect.runPromise + Layer.
 */
import { describe, test, expect } from 'bun:test'
import { Effect, Option } from 'effect'
import { TokenStorageLive } from './TokenStorageLive'
import { TokenStorage } from './TokenStorage'

function makeFakeStorage(): Storage {
  const data: Record<string, string> = {}
  return {
    getItem(key: string) {
      return key in data ? data[key]! : null
    },
    setItem(key: string, value: string) {
      data[key] = value
    },
    removeItem(key: string) {
      delete data[key]
    },
    get length() {
      return Object.keys(data).length
    },
    key() {
      return null
    },
    clear() {
      for (const k of Object.keys(data)) delete data[k]
    },
  }
}

describe('TokenStorageLive', () => {
  describe('when no storage (e.g. SSR/Node)', () => {
    test('getToken should return Option.none()', async () => {
      // Given: TokenStorageLive in environment without window.localStorage
      // When: getToken is called
      const program = Effect.gen(function* () {
        const storage = yield* TokenStorage
        return yield* storage.getToken()
      })

      const result = await Effect.runPromise(
        program.pipe(Effect.provide(TokenStorageLive)),
      )

      // Then: result is none
      expect(Option.isNone(result)).toBe(true)
    })

    test('setToken should not throw', async () => {
      // Given: TokenStorageLive in environment without window.localStorage
      // When: setToken is called
      const program = Effect.gen(function* () {
        const storage = yield* TokenStorage
        yield* storage.setToken('some-token')
      })

      // Then: effect completes successfully
      await expect(Effect.runPromise(program.pipe(Effect.provide(TokenStorageLive)))).resolves.toBeUndefined()
    })

    test('clearToken should not throw', async () => {
      // Given: TokenStorageLive in environment without window.localStorage
      // When: clearToken is called
      const program = Effect.gen(function* () {
        const storage = yield* TokenStorage
        yield* storage.clearToken()
      })

      await expect(Effect.runPromise(program.pipe(Effect.provide(TokenStorageLive)))).resolves.toBeUndefined()
    })

    test('getScope should return Option.none()', async () => {
      // Given: TokenStorageLive in environment without window.localStorage
      // When: getScope is called
      const program = Effect.gen(function* () {
        const storage = yield* TokenStorage
        return yield* storage.getScope()
      })

      const result = await Effect.runPromise(
        program.pipe(Effect.provide(TokenStorageLive)),
      )

      expect(Option.isNone(result)).toBe(true)
    })

    test('setScope should not throw', async () => {
      // Given: TokenStorageLive in environment without window.localStorage
      // When: setScope is called
      const program = Effect.gen(function* () {
        const storage = yield* TokenStorage
        yield* storage.setScope('org-1', 'role-1')
      })

      await expect(Effect.runPromise(program.pipe(Effect.provide(TokenStorageLive)))).resolves.toBeUndefined()
    })

    test('clearScope should not throw', async () => {
      // Given: TokenStorageLive in environment without window.localStorage
      // When: clearScope is called
      const program = Effect.gen(function* () {
        const storage = yield* TokenStorage
        yield* storage.clearScope()
      })

      await expect(Effect.runPromise(program.pipe(Effect.provide(TokenStorageLive)))).resolves.toBeUndefined()
    })
  })

  describe('when storage is available', () => {
    test('setToken then getToken should return the stored token', async () => {
      // Given: fake localStorage and TokenStorageLive
      const fakeStorage = makeFakeStorage()
      const g = globalThis as typeof globalThis & { window?: { localStorage: Storage } }
      const origWindow = g.window
      g.window = { localStorage: fakeStorage }

      try {
        // When: setToken then getToken
        const program = Effect.gen(function* () {
          const storage = yield* TokenStorage
          yield* storage.setToken('my-jwt-token')
          return yield* storage.getToken()
        })

        const result = await Effect.runPromise(
          program.pipe(Effect.provide(TokenStorageLive)),
        )

        // Then: getToken returns the stored value
        expect(Option.isSome(result)).toBe(true)
        expect(Option.getOrThrow(result)).toBe('my-jwt-token')
      } finally {
        g.window = origWindow
      }
    })

    test('clearToken should remove stored token', async () => {
      const fakeStorage = makeFakeStorage()
      fakeStorage.setItem('token', 'old-token')
      const g = globalThis as typeof globalThis & { window?: { localStorage: Storage } }
      const origWindow = g.window
      g.window = { localStorage: fakeStorage }

      try {
        const program = Effect.gen(function* () {
          const storage = yield* TokenStorage
          yield* storage.clearToken()
          return yield* storage.getToken()
        })

        const result = await Effect.runPromise(
          program.pipe(Effect.provide(TokenStorageLive)),
        )

        expect(Option.isNone(result)).toBe(true)
      } finally {
        g.window = origWindow
      }
    })

    test('setScope then getScope should return the stored scope', async () => {
      const fakeStorage = makeFakeStorage()
      const g = globalThis as typeof globalThis & { window?: { localStorage: Storage } }
      const origWindow = g.window
      g.window = { localStorage: fakeStorage }

      try {
        const program = Effect.gen(function* () {
          const storage = yield* TokenStorage
          yield* storage.setScope('org-42', 'role-99')
          return yield* storage.getScope()
        })

        const result = await Effect.runPromise(
          program.pipe(Effect.provide(TokenStorageLive)),
        )

        expect(Option.isSome(result)).toBe(true)
        const scope = Option.getOrThrow(result)
        expect(scope.organizationId).toBe('org-42')
        expect(scope.roleId).toBe('role-99')
      } finally {
        g.window = origWindow
      }
    })

    test('clearScope should remove stored scope', async () => {
      const fakeStorage = makeFakeStorage()
      fakeStorage.setItem('currentOrgId', 'org-1')
      fakeStorage.setItem('currentRoleId', 'role-1')
      const g = globalThis as typeof globalThis & { window?: { localStorage: Storage } }
      const origWindow = g.window
      g.window = { localStorage: fakeStorage }

      try {
        const program = Effect.gen(function* () {
          const storage = yield* TokenStorage
          yield* storage.clearScope()
          return yield* storage.getScope()
        })

        const result = await Effect.runPromise(
          program.pipe(Effect.provide(TokenStorageLive)),
        )

        expect(Option.isNone(result)).toBe(true)
      } finally {
        g.window = origWindow
      }
    })

    test('empty token string should yield getToken none', async () => {
      const fakeStorage = makeFakeStorage()
      fakeStorage.setItem('token', '')
      const g = globalThis as typeof globalThis & { window?: { localStorage: Storage } }
      const origWindow = g.window
      g.window = { localStorage: fakeStorage }

      try {
        const program = Effect.gen(function* () {
          const storage = yield* TokenStorage
          return yield* storage.getToken()
        })

        const result = await Effect.runPromise(
          program.pipe(Effect.provide(TokenStorageLive)),
        )

        expect(Option.isNone(result)).toBe(true)
      } finally {
        g.window = origWindow
      }
    })
  })
})
