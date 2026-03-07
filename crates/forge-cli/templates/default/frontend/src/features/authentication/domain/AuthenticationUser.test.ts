/**
 * BDD tests for AuthenticationUser domain
 * Tests verify the behavior of the AuthenticationUser Data.TaggedClass
 */

import { describe, it, expect } from 'bun:test'
import { AuthenticationUser } from './AuthenticationUser'

describe('AuthenticationUser domain', () => {
  describe('instance creation behavior', () => {
    it('should create an instance with id, email, and token', () => {
      // Given user data
      const id = 'user-123'
      const email = 'user@example.com'
      const token = 'bearer-token-abc123'

      // When I create an AuthenticationUser instance
      const user = new AuthenticationUser({ id, email, token })

      // Then it should have all three properties
      expect(user.id).toBe(id)
      expect(user.email).toBe(email)
      expect(user.token).toBe(token)
    })

    it('should create an instance with empty strings when provided', () => {
      // Given user data with empty strings
      const id = ''
      const email = ''
      const token = ''

      // When I create an AuthenticationUser instance
      const user = new AuthenticationUser({ id, email, token })

      // Then it should accept empty strings
      expect(user.id).toBe('')
      expect(user.email).toBe('')
      expect(user.token).toBe('')
    })

    it('should have readonly properties', () => {
      // Given an AuthenticationUser instance
      const user = new AuthenticationUser({
        id: 'user-1',
        email: 'test@example.com',
        token: 'token-1',
      })

      // When I try to modify properties (TypeScript should prevent this)
      // Then the properties should remain unchanged
      // Note: In runtime, readonly doesn't prevent modification, but TypeScript enforces it
      expect(user.id).toBe('user-1')
      expect(user.email).toBe('test@example.com')
      expect(user.token).toBe('token-1')
    })
  })

  describe('structural equality behavior', () => {
    it('should be equal to another instance with same values', () => {
      // Given two AuthenticationUser instances with the same values
      const user1 = new AuthenticationUser({
        id: 'user-123',
        email: 'user@example.com',
        token: 'token-abc',
      })
      const user2 = new AuthenticationUser({
        id: 'user-123',
        email: 'user@example.com',
        token: 'token-abc',
      })

      // When I compare them
      // Then they should be structurally equal (Data.TaggedClass provides this)
      expect(user1).toEqual(user2)
    })

    it('should not be equal when id differs', () => {
      // Given two AuthenticationUser instances with different ids
      const user1 = new AuthenticationUser({
        id: 'user-123',
        email: 'user@example.com',
        token: 'token-abc',
      })
      const user2 = new AuthenticationUser({
        id: 'user-456',
        email: 'user@example.com',
        token: 'token-abc',
      })

      // When I compare them
      // Then they should not be equal
      expect(user1).not.toEqual(user2)
    })

    it('should not be equal when email differs', () => {
      // Given two AuthenticationUser instances with different emails
      const user1 = new AuthenticationUser({
        id: 'user-123',
        email: 'user1@example.com',
        token: 'token-abc',
      })
      const user2 = new AuthenticationUser({
        id: 'user-123',
        email: 'user2@example.com',
        token: 'token-abc',
      })

      // When I compare them
      // Then they should not be equal
      expect(user1).not.toEqual(user2)
    })

    it('should not be equal when token differs', () => {
      // Given two AuthenticationUser instances with different tokens
      const user1 = new AuthenticationUser({
        id: 'user-123',
        email: 'user@example.com',
        token: 'token-abc',
      })
      const user2 = new AuthenticationUser({
        id: 'user-123',
        email: 'user@example.com',
        token: 'token-xyz',
      })

      // When I compare them
      // Then they should not be equal
      expect(user1).not.toEqual(user2)
    })
  })

  describe('tagged class behavior', () => {
    it('should have the AuthenticationUser tag', () => {
      // Given an AuthenticationUser instance
      const user = new AuthenticationUser({
        id: 'user-1',
        email: 'test@example.com',
        token: 'token-1',
      })

      // When I check the tag
      // Then it should be tagged as 'AuthenticationUser'
      // Data.TaggedClass provides a _tag property
      expect((user as any)._tag).toBe('AuthenticationUser')
    })

    it('should be identifiable as AuthenticationUser type', () => {
      // Given an AuthenticationUser instance
      const user = new AuthenticationUser({
        id: 'user-1',
        email: 'test@example.com',
        token: 'token-1',
      })

      // When I check the instance
      // Then it should be an instance of AuthenticationUser
      expect(user).toBeInstanceOf(AuthenticationUser)
    })
  })

  describe('property access behavior', () => {
    it('should allow reading id property', () => {
      // Given an AuthenticationUser instance
      const user = new AuthenticationUser({
        id: 'test-id',
        email: 'test@example.com',
        token: 'test-token',
      })

      // When I read the id property
      const id = user.id

      // Then it should return the correct value
      expect(id).toBe('test-id')
    })

    it('should allow reading email property', () => {
      // Given an AuthenticationUser instance
      const user = new AuthenticationUser({
        id: 'test-id',
        email: 'test@example.com',
        token: 'test-token',
      })

      // When I read the email property
      const email = user.email

      // Then it should return the correct value
      expect(email).toBe('test@example.com')
    })

    it('should allow reading token property', () => {
      // Given an AuthenticationUser instance
      const user = new AuthenticationUser({
        id: 'test-id',
        email: 'test@example.com',
        token: 'bearer-token-123',
      })

      // When I read the token property
      const token = user.token

      // Then it should return the correct value
      expect(token).toBe('bearer-token-123')
    })
  })

  describe('usage in authentication flow', () => {
    it('should represent a user identity with session token', () => {
      // Given user identity data from API
      const userId = '550e8400-e29b-41d4-a716-446655440000'
      const userEmail = 'authenticated@example.com'
      const sessionToken = 'eyJhbGciOiJIUzI1NiIsInR5cCI6IkpXVCJ9'

      // When I create an AuthenticationUser
      const user = new AuthenticationUser({
        id: userId,
        email: userEmail,
        token: sessionToken,
      })

      // Then it should represent a complete authenticated user identity
      expect(user.id).toBe(userId)
      expect(user.email).toBe(userEmail)
      expect(user.token).toBe(sessionToken)
    })

    it('should work with UUID strings as id', () => {
      // Given a UUID string as user id
      const uuid = '123e4567-e89b-12d3-a456-426614174000'

      // When I create an AuthenticationUser with UUID id
      const user = new AuthenticationUser({
        id: uuid,
        email: 'user@example.com',
        token: 'token',
      })

      // Then it should accept and store the UUID
      expect(user.id).toBe(uuid)
    })
  })
})
