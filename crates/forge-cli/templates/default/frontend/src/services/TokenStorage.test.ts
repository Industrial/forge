/**
 * BDD tests for TokenStorage service interface
 * Tests verify the service contract and type definitions
 */
import { describe, test, expect } from 'bun:test'
import { Effect, Option } from 'effect'
import { TokenStorage, type TokenStorageService } from './TokenStorage'

describe('TokenStorage service interface', () => {
  describe('TokenStorage tag behavior', () => {
    test('should export TokenStorage as Context.GenericTag', () => {
      // Given: TokenStorage module
      // When: checking TokenStorage export
      // Then: should be defined
      expect(TokenStorage).toBeDefined()
      expect(TokenStorage.key).toBeDefined()
    })

    test('should have correct tag key', () => {
      // Given: TokenStorage tag
      // When: checking tag key
      // Then: should be '@forge/TokenStorage'
      expect(TokenStorage.key).toBe('@forge/TokenStorage')
    })
  })

  describe('TokenStorageService type behavior', () => {
    test('should export TokenStorageService type', () => {
      // Given: TokenStorageService type
      // When: checking type export
      // Then: should be usable (TypeScript enforces this at compile time)
      const testService: TokenStorageService = {
        getToken: () => Effect.succeed(Option.none()),
        setToken: () => Effect.void,
        clearToken: () => Effect.void,
        getScope: () => Effect.succeed(Option.none()),
        setScope: () => Effect.void,
        clearScope: () => Effect.void,
      }
      expect(testService).toBeDefined()
    })
  })

  describe('TokenStorage interface contract', () => {
    test('should require getToken method', () => {
      // Given: TokenStorage interface
      // When: checking required methods
      // Then: getToken should be required
      const service: TokenStorage = {
        getToken: () => Effect.succeed(Option.none()),
        setToken: () => Effect.void,
        clearToken: () => Effect.void,
        getScope: () => Effect.succeed(Option.none()),
        setScope: () => Effect.void,
        clearScope: () => Effect.void,
      }
      expect(typeof service.getToken).toBe('function')
    })

    test('should require setToken method', () => {
      // Given: TokenStorage interface
      // When: checking required methods
      // Then: setToken should be required
      const service: TokenStorage = {
        getToken: () => Effect.succeed(Option.none()),
        setToken: () => Effect.void,
        clearToken: () => Effect.void,
        getScope: () => Effect.succeed(Option.none()),
        setScope: () => Effect.void,
        clearScope: () => Effect.void,
      }
      expect(typeof service.setToken).toBe('function')
    })

    test('should require clearToken method', () => {
      // Given: TokenStorage interface
      // When: checking required methods
      // Then: clearToken should be required
      const service: TokenStorage = {
        getToken: () => Effect.succeed(Option.none()),
        setToken: () => Effect.void,
        clearToken: () => Effect.void,
        getScope: () => Effect.succeed(Option.none()),
        setScope: () => Effect.void,
        clearScope: () => Effect.void,
      }
      expect(typeof service.clearToken).toBe('function')
    })

    test('should require getScope method', () => {
      // Given: TokenStorage interface
      // When: checking required methods
      // Then: getScope should be required
      const service: TokenStorage = {
        getToken: () => Effect.succeed(Option.none()),
        setToken: () => Effect.void,
        clearToken: () => Effect.void,
        getScope: () => Effect.succeed(Option.none()),
        setScope: () => Effect.void,
        clearScope: () => Effect.void,
      }
      expect(typeof service.getScope).toBe('function')
    })

    test('should require setScope method', () => {
      // Given: TokenStorage interface
      // When: checking required methods
      // Then: setScope should be required
      const service: TokenStorage = {
        getToken: () => Effect.succeed(Option.none()),
        setToken: () => Effect.void,
        clearToken: () => Effect.void,
        getScope: () => Effect.succeed(Option.none()),
        setScope: () => Effect.void,
        clearScope: () => Effect.void,
      }
      expect(typeof service.setScope).toBe('function')
    })

    test('should require clearScope method', () => {
      // Given: TokenStorage interface
      // When: checking required methods
      // Then: clearScope should be required
      const service: TokenStorage = {
        getToken: () => Effect.succeed(Option.none()),
        setToken: () => Effect.void,
        clearToken: () => Effect.void,
        getScope: () => Effect.succeed(Option.none()),
        setScope: () => Effect.void,
        clearScope: () => Effect.void,
      }
      expect(typeof service.clearScope).toBe('function')
    })
  })

  describe('getToken return type behavior', () => {
    test('should return Effect<Option<string>, never, never>', async () => {
      // Given: a TokenStorage implementation
      const service: TokenStorage = {
        getToken: () => Effect.succeed(Option.none()),
        setToken: () => Effect.void,
        clearToken: () => Effect.void,
        getScope: () => Effect.succeed(Option.none()),
        setScope: () => Effect.void,
        clearScope: () => Effect.void,
      }

      // When: calling getToken
      const result = await Effect.runPromise(service.getToken())

      // Then: should return Option<string>
      expect(Option.isOption(result)).toBe(true)
    })
  })

  describe('getScope return type behavior', () => {
    test('should return Effect<Option<{organizationId: string, roleId: string}>, never, never>', async () => {
      // Given: a TokenStorage implementation
      const service: TokenStorage = {
        getToken: () => Effect.succeed(Option.none()),
        setToken: () => Effect.void,
        clearToken: () => Effect.void,
        getScope: () => Effect.succeed(Option.none()),
        setScope: () => Effect.void,
        clearScope: () => Effect.void,
      }

      // When: calling getScope
      const result = await Effect.runPromise(service.getScope())

      // Then: should return Option<scope>
      expect(Option.isOption(result)).toBe(true)
    })
  })

  describe('setToken parameter behavior', () => {
    test('should accept string token parameter', async () => {
      // Given: a TokenStorage implementation
      const service: TokenStorage = {
        getToken: () => Effect.succeed(Option.none()),
        setToken: (token: string) => {
          expect(typeof token).toBe('string')
          return Effect.void
        },
        clearToken: () => Effect.void,
        getScope: () => Effect.succeed(Option.none()),
        setScope: () => Effect.void,
        clearScope: () => Effect.void,
      }

      // When: calling setToken with string
      await Effect.runPromise(service.setToken('test-token'))
    })
  })

  describe('setScope parameter behavior', () => {
    test('should accept organizationId and roleId string parameters', async () => {
      // Given: a TokenStorage implementation
      const service: TokenStorage = {
        getToken: () => Effect.succeed(Option.none()),
        setToken: () => Effect.void,
        clearToken: () => Effect.void,
        getScope: () => Effect.succeed(Option.none()),
        setScope: (organizationId: string, roleId: string) => {
          expect(typeof organizationId).toBe('string')
          expect(typeof roleId).toBe('string')
          return Effect.void
        },
        clearScope: () => Effect.void,
      }

      // When: calling setScope with strings
      await Effect.runPromise(service.setScope('org-1', 'role-1'))
    })
  })
})
