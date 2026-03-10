/**
 * BDD tests for Authentication service interface
 * Tests verify the behavior of the Authentication service contract and Context tag
 */

import { describe, it, expect } from 'bun:test'
import { Effect, Option, Layer } from 'effect'
import { Authentication, type AuthenticationService } from './Authentication'

describe('Authentication service interface', () => {
  describe('export behavior', () => {
    it('should export Authentication as a Context tag', () => {
      // Given the Authentication module
      // When I check the export
      // Then Authentication should be a Context tag
      expect(Authentication).toBeDefined()
      expect(typeof Authentication).toBe('object')
      // Context.GenericTag returns an object with key, etc.
      expect(Authentication).toHaveProperty('key')
    })

    it('should export AuthenticationService as a type alias', () => {
      // Given the Authentication module
      // When I check the type alias
      // Then AuthenticationService should be defined
      // This is verified by TypeScript at compile time
      // Runtime check: we can verify the interface exists
      expect(Authentication).toBeDefined()
    })
  })

  describe('Context tag behavior', () => {
    it('should have the correct tag identifier', () => {
      // Given the Authentication Context tag
      // When I check its key
      // Then it should have the identifier '@forge/Authentication'
      expect(Authentication.key).toBe('@forge/Authentication')
    })

    it('should be usable with Layer.succeed', () => {
      // Given a mock Authentication implementation
      const mockAuth: AuthenticationService = {
        restoreSession: () => Effect.void,
        getCurrentUser: () => Effect.succeed(Option.none()),
        login: () =>
          Effect.fail(new AuthenticationError({ message: 'Not implemented' })),
        register: () =>
          Effect.fail(new AuthenticationError({ message: 'Not implemented' })),
        logout: () => Effect.void,
        selectScope: () =>
          Effect.fail(new ScopeError({ message: 'Not implemented' })),
      }

      // When I create a Layer with it
      const layer = Layer.succeed(Authentication, mockAuth)

      // Then it should be a valid Layer
      expect(layer).toBeDefined()
      expect(typeof layer).toBe('object')
    })
  })

  describe('interface structure behavior', () => {
    it('should require restoreSession method', () => {
      // Given the Authentication interface
      // When I create an implementation
      // Then it must have restoreSession method
      const mockAuth: AuthenticationService = {
        restoreSession: () => Effect.void,
        getCurrentUser: () => Effect.succeed(Option.none()),
        login: () =>
          Effect.fail(new AuthenticationError({ message: 'Not implemented' })),
        register: () =>
          Effect.fail(new AuthenticationError({ message: 'Not implemented' })),
        logout: () => Effect.void,
        selectScope: () =>
          Effect.fail(new ScopeError({ message: 'Not implemented' })),
      }

      // Then restoreSession should exist and return Effect<void, never, never>
      expect(mockAuth.restoreSession).toBeDefined()
      expect(typeof mockAuth.restoreSession).toBe('function')
    })

    it('should require getCurrentUser method', () => {
      // Given the Authentication interface
      // When I create an implementation
      const mockAuth: AuthenticationService = {
        restoreSession: () => Effect.void,
        getCurrentUser: () => Effect.succeed(Option.none()),
        login: () =>
          Effect.fail(new AuthenticationError({ message: 'Not implemented' })),
        register: () =>
          Effect.fail(new AuthenticationError({ message: 'Not implemented' })),
        logout: () => Effect.void,
        selectScope: () =>
          Effect.fail(new ScopeError({ message: 'Not implemented' })),
      }

      // Then getCurrentUser should exist and return Effect<Option<AuthenticationUser>, AuthenticationError, never>
      expect(mockAuth.getCurrentUser).toBeDefined()
      expect(typeof mockAuth.getCurrentUser).toBe('function')
    })

    it('should require login method', () => {
      // Given the Authentication interface
      // When I create an implementation
      const mockAuth: AuthenticationService = {
        restoreSession: () => Effect.void,
        getCurrentUser: () => Effect.succeed(Option.none()),
        login: (_email: string, _password: string) =>
          Effect.fail(new AuthenticationError({ message: 'Not implemented' })),
        register: () =>
          Effect.fail(new AuthenticationError({ message: 'Not implemented' })),
        logout: () => Effect.void,
        selectScope: () =>
          Effect.fail(new ScopeError({ message: 'Not implemented' })),
      }

      // Then login should exist and accept email and password
      expect(mockAuth.login).toBeDefined()
      expect(typeof mockAuth.login).toBe('function')
    })

    it('should require register method', () => {
      // Given the Authentication interface
      // When I create an implementation
      const mockAuth: AuthenticationService = {
        restoreSession: () => Effect.void,
        getCurrentUser: () => Effect.succeed(Option.none()),
        login: () =>
          Effect.fail(new AuthenticationError({ message: 'Not implemented' })),
        register: (_email: string, _password: string) =>
          Effect.fail(new AuthenticationError({ message: 'Not implemented' })),
        logout: () => Effect.void,
        selectScope: () =>
          Effect.fail(new ScopeError({ message: 'Not implemented' })),
      }

      // Then register should exist and accept email and password
      expect(mockAuth.register).toBeDefined()
      expect(typeof mockAuth.register).toBe('function')
    })

    it('should require logout method', () => {
      // Given the Authentication interface
      // When I create an implementation
      const mockAuth: AuthenticationService = {
        restoreSession: () => Effect.void,
        getCurrentUser: () => Effect.succeed(Option.none()),
        login: () =>
          Effect.fail(new AuthenticationError({ message: 'Not implemented' })),
        register: () =>
          Effect.fail(new AuthenticationError({ message: 'Not implemented' })),
        logout: () => Effect.void,
        selectScope: () =>
          Effect.fail(new ScopeError({ message: 'Not implemented' })),
      }

      // Then logout should exist and return Effect<void, never, never>
      expect(mockAuth.logout).toBeDefined()
      expect(typeof mockAuth.logout).toBe('function')
    })

    it('should require selectScope method', () => {
      // Given the Authentication interface
      // When I create an implementation
      const mockAuth: AuthenticationService = {
        restoreSession: () => Effect.void,
        getCurrentUser: () => Effect.succeed(Option.none()),
        login: () =>
          Effect.fail(new AuthenticationError({ message: 'Not implemented' })),
        register: () =>
          Effect.fail(new AuthenticationError({ message: 'Not implemented' })),
        logout: () => Effect.void,
        selectScope: (_organizationId: string, _roleId: string) =>
          Effect.fail(new ScopeError({ message: 'Not implemented' })),
      }

      // Then selectScope should exist and accept organizationId and roleId
      expect(mockAuth.selectScope).toBeDefined()
      expect(typeof mockAuth.selectScope).toBe('function')
    })
  })

  describe('Effect type compatibility', () => {
    it('should work with Effect.runPromise', async () => {
      // Given a mock Authentication implementation
      const mockAuth: AuthenticationService = {
        restoreSession: () => Effect.void,
        getCurrentUser: () => Effect.succeed(Option.none()),
        login: () =>
          Effect.fail(new AuthenticationError({ message: 'Not implemented' })),
        register: () =>
          Effect.fail(new AuthenticationError({ message: 'Not implemented' })),
        logout: () => Effect.void,
        selectScope: () =>
          Effect.fail(new ScopeError({ message: 'Not implemented' })),
      }

      // When I run an Effect that uses it
      const layer = Layer.succeed(Authentication, mockAuth)
      const program = Effect.gen(function* () {
        const auth = yield* Authentication
        yield* auth.restoreSession()
      })

      // Then it should be runnable with Effect.runPromise
      await expect(
        Effect.runPromise(program.pipe(Effect.provide(layer))),
      ).resolves.toBeUndefined()
    })

    it('should handle getCurrentUser returning Option', async () => {
      // Given a mock Authentication implementation
      const mockAuth: AuthenticationService = {
        restoreSession: () => Effect.void,
        getCurrentUser: () => Effect.succeed(Option.none()),
        login: () =>
          Effect.fail(new AuthenticationError({ message: 'Not implemented' })),
        register: () =>
          Effect.fail(new AuthenticationError({ message: 'Not implemented' })),
        logout: () => Effect.void,
        selectScope: () =>
          Effect.fail(new ScopeError({ message: 'Not implemented' })),
      }

      // When I run getCurrentUser
      const layer = Layer.succeed(Authentication, mockAuth)
      const program = Effect.gen(function* () {
        const auth = yield* Authentication
        return yield* auth.getCurrentUser()
      })

      // Then it should return Option<AuthenticationUser>
      const result = await Effect.runPromise(
        program.pipe(Effect.provide(layer)),
      )
      expect(Option.isNone(result)).toBe(true)
    })
  })

  describe('service contract behavior', () => {
    it('should enforce readonly methods', () => {
      // Given the Authentication interface
      // When I check the interface
      // Then all methods should be readonly (enforced by TypeScript)
      // Runtime check: we can verify methods exist
      const mockAuth: AuthenticationService = {
        restoreSession: () => Effect.void,
        getCurrentUser: () => Effect.succeed(Option.none()),
        login: () =>
          Effect.fail(new AuthenticationError({ message: 'Not implemented' })),
        register: () =>
          Effect.fail(new AuthenticationError({ message: 'Not implemented' })),
        logout: () => Effect.void,
        selectScope: () =>
          Effect.fail(new ScopeError({ message: 'Not implemented' })),
      }

      // All methods should be functions
      expect(typeof mockAuth.restoreSession).toBe('function')
      expect(typeof mockAuth.getCurrentUser).toBe('function')
      expect(typeof mockAuth.login).toBe('function')
      expect(typeof mockAuth.register).toBe('function')
      expect(typeof mockAuth.logout).toBe('function')
      expect(typeof mockAuth.selectScope).toBe('function')
    })

    it('should match AuthenticationService type alias', () => {
      // Given AuthenticationService type alias
      // When I create an implementation
      const mockAuth: AuthenticationService = {
        restoreSession: () => Effect.void,
        getCurrentUser: () => Effect.succeed(Option.none()),
        login: () =>
          Effect.fail(new AuthenticationError({ message: 'Not implemented' })),
        register: () =>
          Effect.fail(new AuthenticationError({ message: 'Not implemented' })),
        logout: () => Effect.void,
        selectScope: () =>
          Effect.fail(new ScopeError({ message: 'Not implemented' })),
      }

      // Then it should be assignable to AuthenticationService
      // This is verified by TypeScript, runtime check verifies structure
      expect(mockAuth).toBeDefined()
      expect(typeof mockAuth.restoreSession).toBe('function')
    })
  })

  describe('usage in Effect programs', () => {
    it('should be accessible via yield* Authentication', async () => {
      // Given an Effect program
      const mockAuth: AuthenticationService = {
        restoreSession: () => Effect.void,
        getCurrentUser: () => Effect.succeed(Option.none()),
        login: () =>
          Effect.fail(new AuthenticationError({ message: 'Not implemented' })),
        register: () =>
          Effect.fail(new AuthenticationError({ message: 'Not implemented' })),
        logout: () => Effect.void,
        selectScope: () =>
          Effect.fail(new ScopeError({ message: 'Not implemented' })),
      }

      const layer = Layer.succeed(Authentication, mockAuth)
      const program = Effect.gen(function* () {
        const auth = yield* Authentication
        return auth
      })

      // When I run the program
      const result = await Effect.runPromise(
        program.pipe(Effect.provide(layer)),
      )

      // Then it should return the Authentication service
      expect(result).toBe(mockAuth)
    })

    it('should allow calling methods in Effect programs', async () => {
      // Given an Effect program that calls Authentication methods
      const mockAuth: AuthenticationService = {
        restoreSession: () => Effect.void,
        getCurrentUser: () => Effect.succeed(Option.none()),
        login: () =>
          Effect.fail(new AuthenticationError({ message: 'Not implemented' })),
        register: () =>
          Effect.fail(new AuthenticationError({ message: 'Not implemented' })),
        logout: () => Effect.void,
        selectScope: () =>
          Effect.fail(new ScopeError({ message: 'Not implemented' })),
      }

      const layer = Layer.succeed(Authentication, mockAuth)
      const program = Effect.gen(function* () {
        const auth = yield* Authentication
        yield* auth.restoreSession()
        const user = yield* auth.getCurrentUser()
        yield* auth.logout()
        return user
      })

      // When I run the program
      const result = await Effect.runPromise(
        program.pipe(Effect.provide(layer)),
      )

      // Then it should execute successfully
      expect(Option.isNone(result)).toBe(true)
    })
  })
})
