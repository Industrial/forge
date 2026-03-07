/**
 * BDD tests for TokenStorageMock
 * Tests verify in-memory TokenStorage implementation for tests
 */
import { describe, test, expect, beforeEach } from 'bun:test'
import { Effect, Option } from 'effect'
import { makeTokenStorageMock } from './TokenStorageMock'
import { TokenStorage } from './TokenStorage'

describe('TokenStorageMock', () => {
  let mock: ReturnType<typeof makeTokenStorageMock>

  beforeEach(() => {
    // Given: a fresh TokenStorageMock instance
    mock = makeTokenStorageMock()
  })

  describe('makeTokenStorageMock behavior', () => {
    test('should return layer and control functions', () => {
      // Given: makeTokenStorageMock function
      // When: calling it
      const result = makeTokenStorageMock()

      // Then: should return layer and control functions
      expect(result.layer).toBeDefined()
      expect(typeof result.setToken).toBe('function')
      expect(typeof result.clearToken).toBe('function')
      expect(typeof result.setScope).toBe('function')
      expect(typeof result.clearScope).toBe('function')
    })
  })

  describe('getToken behavior', () => {
    test('should return Option.none() initially', async () => {
      // Given: TokenStorageMock with no token set
      // When: getting token
      const program = Effect.gen(function* () {
        const storage = yield* TokenStorage
        return yield* storage.getToken()
      })

      const result = await Effect.runPromise(
        program.pipe(Effect.provide(mock.layer)),
      )

      // Then: should return none
      expect(Option.isNone(result)).toBe(true)
    })

    test('should return Option.some() after setToken', async () => {
      // Given: TokenStorageMock
      mock.setToken('test-token-123')

      // When: getting token
      const program = Effect.gen(function* () {
        const storage = yield* TokenStorage
        return yield* storage.getToken()
      })

      const result = await Effect.runPromise(
        program.pipe(Effect.provide(mock.layer)),
      )

      // Then: should return some token
      expect(Option.isSome(result)).toBe(true)
      expect(Option.getOrThrow(result)).toBe('test-token-123')
    })

    test('should return Option.none() after clearToken', async () => {
      // Given: TokenStorageMock with token set then cleared
      mock.setToken('test-token')
      mock.clearToken()

      // When: getting token
      const program = Effect.gen(function* () {
        const storage = yield* TokenStorage
        return yield* storage.getToken()
      })

      const result = await Effect.runPromise(
        program.pipe(Effect.provide(mock.layer)),
      )

      // Then: should return none
      expect(Option.isNone(result)).toBe(true)
    })

    test('should return Option.none() for empty string token', async () => {
      // Given: TokenStorageMock with empty string token
      mock.setToken('')

      // When: getting token
      const program = Effect.gen(function* () {
        const storage = yield* TokenStorage
        return yield* storage.getToken()
      })

      const result = await Effect.runPromise(
        program.pipe(Effect.provide(mock.layer)),
      )

      // Then: should return none (empty string treated as no token)
      expect(Option.isNone(result)).toBe(true)
    })
  })

  describe('setToken behavior', () => {
    test('should store token via Effect', async () => {
      // Given: TokenStorageMock
      // When: setting token via Effect
      const program = Effect.gen(function* () {
        const storage = yield* TokenStorage
        yield* storage.setToken('effect-token')
        return yield* storage.getToken()
      })

      const result = await Effect.runPromise(
        program.pipe(Effect.provide(mock.layer)),
      )

      // Then: token should be stored
      expect(Option.isSome(result)).toBe(true)
      expect(Option.getOrThrow(result)).toBe('effect-token')
    })

    test('should overwrite existing token', async () => {
      // Given: TokenStorageMock with existing token
      mock.setToken('old-token')

      // When: setting new token via Effect
      const program = Effect.gen(function* () {
        const storage = yield* TokenStorage
        yield* storage.setToken('new-token')
        return yield* storage.getToken()
      })

      const result = await Effect.runPromise(
        program.pipe(Effect.provide(mock.layer)),
      )

      // Then: should have new token
      expect(Option.getOrThrow(result)).toBe('new-token')
    })
  })

  describe('clearToken behavior', () => {
    test('should clear token via Effect', async () => {
      // Given: TokenStorageMock with token set
      mock.setToken('test-token')

      // When: clearing token via Effect
      const program = Effect.gen(function* () {
        const storage = yield* TokenStorage
        yield* storage.clearToken()
        return yield* storage.getToken()
      })

      const result = await Effect.runPromise(
        program.pipe(Effect.provide(mock.layer)),
      )

      // Then: token should be cleared
      expect(Option.isNone(result)).toBe(true)
    })
  })

  describe('getScope behavior', () => {
    test('should return Option.none() initially', async () => {
      // Given: TokenStorageMock with no scope set
      // When: getting scope
      const program = Effect.gen(function* () {
        const storage = yield* TokenStorage
        return yield* storage.getScope()
      })

      const result = await Effect.runPromise(
        program.pipe(Effect.provide(mock.layer)),
      )

      // Then: should return none
      expect(Option.isNone(result)).toBe(true)
    })

    test('should return Option.some() after setScope', async () => {
      // Given: TokenStorageMock
      mock.setScope('org-42', 'role-99')

      // When: getting scope
      const program = Effect.gen(function* () {
        const storage = yield* TokenStorage
        return yield* storage.getScope()
      })

      const result = await Effect.runPromise(
        program.pipe(Effect.provide(mock.layer)),
      )

      // Then: should return some scope
      expect(Option.isSome(result)).toBe(true)
      const scope = Option.getOrThrow(result)
      expect(scope.organizationId).toBe('org-42')
      expect(scope.roleId).toBe('role-99')
    })

    test('should return Option.none() after clearScope', async () => {
      // Given: TokenStorageMock with scope set then cleared
      mock.setScope('org-1', 'role-1')
      mock.clearScope()

      // When: getting scope
      const program = Effect.gen(function* () {
        const storage = yield* TokenStorage
        return yield* storage.getScope()
      })

      const result = await Effect.runPromise(
        program.pipe(Effect.provide(mock.layer)),
      )

      // Then: should return none
      expect(Option.isNone(result)).toBe(true)
    })
  })

  describe('setScope behavior', () => {
    test('should store scope via Effect', async () => {
      // Given: TokenStorageMock
      // When: setting scope via Effect
      const program = Effect.gen(function* () {
        const storage = yield* TokenStorage
        yield* storage.setScope('org-123', 'role-456')
        return yield* storage.getScope()
      })

      const result = await Effect.runPromise(
        program.pipe(Effect.provide(mock.layer)),
      )

      // Then: scope should be stored
      expect(Option.isSome(result)).toBe(true)
      const scope = Option.getOrThrow(result)
      expect(scope.organizationId).toBe('org-123')
      expect(scope.roleId).toBe('role-456')
    })

    test('should overwrite existing scope', async () => {
      // Given: TokenStorageMock with existing scope
      mock.setScope('org-1', 'role-1')

      // When: setting new scope via Effect
      const program = Effect.gen(function* () {
        const storage = yield* TokenStorage
        yield* storage.setScope('org-2', 'role-2')
        return yield* storage.getScope()
      })

      const result = await Effect.runPromise(
        program.pipe(Effect.provide(mock.layer)),
      )

      // Then: should have new scope
      const scope = Option.getOrThrow(result)
      expect(scope.organizationId).toBe('org-2')
      expect(scope.roleId).toBe('role-2')
    })
  })

  describe('clearScope behavior', () => {
    test('should clear scope via Effect', async () => {
      // Given: TokenStorageMock with scope set
      mock.setScope('org-1', 'role-1')

      // When: clearing scope via Effect
      const program = Effect.gen(function* () {
        const storage = yield* TokenStorage
        yield* storage.clearScope()
        return yield* storage.getScope()
      })

      const result = await Effect.runPromise(
        program.pipe(Effect.provide(mock.layer)),
      )

      // Then: scope should be cleared
      expect(Option.isNone(result)).toBe(true)
    })
  })

  describe('control functions behavior', () => {
    test('should allow direct token manipulation', () => {
      // Given: TokenStorageMock
      // When: using control functions directly
      mock.setToken('direct-token')
      mock.setScope('direct-org', 'direct-role')

      // Then: should work (verified by getToken/getScope tests)
      expect(mock).toBeDefined()
    })

    test('should allow clearing via control functions', () => {
      // Given: TokenStorageMock with values set
      mock.setToken('token')
      mock.setScope('org', 'role')

      // When: clearing via control functions
      mock.clearToken()
      mock.clearScope()

      // Then: should work (verified by getToken/getScope tests)
      expect(mock).toBeDefined()
    })
  })

  describe('integration scenarios', () => {
    test('should handle full token lifecycle', async () => {
      // Given: TokenStorageMock
      // When: performing full lifecycle
      const program = Effect.gen(function* () {
        const storage = yield* TokenStorage

        // Set token
        yield* storage.setToken('lifecycle-token')
        let token = yield* storage.getToken()
        expect(Option.isSome(token)).toBe(true)

        // Clear token
        yield* storage.clearToken()
        token = yield* storage.getToken()
        expect(Option.isNone(token)).toBe(true)

        // Set scope
        yield* storage.setScope('lifecycle-org', 'lifecycle-role')
        let scope = yield* storage.getScope()
        expect(Option.isSome(scope)).toBe(true)

        // Clear scope
        yield* storage.clearScope()
        scope = yield* storage.getScope()
        expect(Option.isNone(scope)).toBe(true)

        return 'done'
      })

      const result = await Effect.runPromise(
        program.pipe(Effect.provide(mock.layer)),
      )

      expect(result).toBe('done')
    })

    test('should work with control functions and Effect together', async () => {
      // Given: TokenStorageMock
      // When: mixing control functions and Effect
      mock.setToken('control-token')

      const program = Effect.gen(function* () {
        const storage = yield* TokenStorage
        let token = yield* storage.getToken()
        expect(Option.getOrThrow(token)).toBe('control-token')

        yield* storage.setToken('effect-token')
        token = yield* storage.getToken()
        expect(Option.getOrThrow(token)).toBe('effect-token')

        mock.clearToken()
        token = yield* storage.getToken()
        expect(Option.isNone(token)).toBe(true)
      })

      await Effect.runPromise(program.pipe(Effect.provide(mock.layer)))
    })
  })
})
