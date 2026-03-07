/**
 * BDD tests for userFormSchemas
 * Tests verify form schema validation behavior using Effect Schema
 */

import { describe, test, expect } from 'bun:test'
import { Schema } from 'effect'
import {
  loginFormSchema,
  registerFormSchema,
  userAddFormSchema,
  userAddFormSchemaStrict,
  userEditFormSchema,
  type LoginFormValues,
  type RegisterFormValues,
  type UserAddFormValues,
  type UserAddFormValuesStrict,
  type UserEditFormValues,
} from './userFormSchemas'

describe('userFormSchemas', () => {
  describe('loginFormSchema behavior', () => {
    test('should validate valid login form data', () => {
      // Given: valid login form data
      const validData = {
        email: 'user@example.com',
        password: 'anypassword',
      }

      // When: decoding the data
      const result = Schema.decodeUnknownSync(loginFormSchema)(validData)

      // Then: should decode successfully
      expect(result.email).toBe('user@example.com')
      expect(result.password).toBe('anypassword')
    })

    test('should reject invalid email format', () => {
      // Given: login form data with invalid email
      const invalidData = {
        email: 'not-an-email',
        password: 'password123',
      }

      // When: decoding the data
      // Then: should throw validation error
      expect(() => {
        Schema.decodeUnknownSync(loginFormSchema)(invalidData)
      }).toThrow()
    })

    test('should reject empty email', () => {
      // Given: login form data with empty email
      const invalidData = {
        email: '',
        password: 'password123',
      }

      // When: decoding the data
      // Then: should throw validation error
      expect(() => {
        Schema.decodeUnknownSync(loginFormSchema)(invalidData)
      }).toThrow()
    })

    test('should reject missing email', () => {
      // Given: login form data without email
      const invalidData = {
        password: 'password123',
      }

      // When: decoding the data
      // Then: should throw validation error
      expect(() => {
        Schema.decodeUnknownSync(loginFormSchema)(invalidData)
      }).toThrow()
    })

    test('should reject missing password', () => {
      // Given: login form data without password
      const invalidData = {
        email: 'user@example.com',
      }

      // When: decoding the data
      // Then: should throw validation error
      expect(() => {
        Schema.decodeUnknownSync(loginFormSchema)(invalidData)
      }).toThrow()
    })

    test('should reject email with whitespace (pattern validation before trim)', () => {
      // Given: login form data with email containing whitespace
      // Note: Pattern validation happens before trimming, so whitespace causes failure
      const invalidData = {
        email: '  user@example.com  ',
        password: 'password123',
      }

      // When: decoding the data
      // Then: should throw validation error (pattern doesn't match due to whitespace)
      expect(() => {
        Schema.decodeUnknownSync(loginFormSchema)(invalidData)
      }).toThrow()
    })
  })

  describe('registerFormSchema behavior', () => {
    test('should validate valid register form data', () => {
      // Given: valid register form data
      const validData = {
        email: 'newuser@example.com',
        password: 'password123',
      }

      // When: decoding the data
      const result = Schema.decodeUnknownSync(registerFormSchema)(validData)

      // Then: should decode successfully
      expect(result.email).toBe('newuser@example.com')
      expect(result.password).toBe('password123')
    })

    test('should reject password shorter than 8 characters', () => {
      // Given: register form data with short password
      const invalidData = {
        email: 'user@example.com',
        password: 'short',
      }

      // When: decoding the data
      // Then: should throw validation error
      expect(() => {
        Schema.decodeUnknownSync(registerFormSchema)(invalidData)
      }).toThrow()
    })

    test('should accept password with exactly 8 characters', () => {
      // Given: register form data with 8-character password
      const validData = {
        email: 'user@example.com',
        password: '12345678',
      }

      // When: decoding the data
      const result = Schema.decodeUnknownSync(registerFormSchema)(validData)

      // Then: should decode successfully
      expect(result.password).toBe('12345678')
    })

    test('should reject invalid email format', () => {
      // Given: register form data with invalid email
      const invalidData = {
        email: 'invalid-email',
        password: 'password123',
      }

      // When: decoding the data
      // Then: should throw validation error
      expect(() => {
        Schema.decodeUnknownSync(registerFormSchema)(invalidData)
      }).toThrow()
    })

    test('should reject empty password', () => {
      // Given: register form data with empty password
      const invalidData = {
        email: 'user@example.com',
        password: '',
      }

      // When: decoding the data
      // Then: should throw validation error
      expect(() => {
        Schema.decodeUnknownSync(registerFormSchema)(invalidData)
      }).toThrow()
    })
  })

  describe('userAddFormSchema behavior', () => {
    test('should validate valid user add form data', () => {
      // Given: valid user add form data
      const validData = {
        email: 'newuser@example.com',
        password: 'password123',
        orgId: 'org-123',
        roleIds: ['role-1', 'role-2'],
      }

      // When: decoding the data
      const result = Schema.decodeUnknownSync(userAddFormSchema)(validData)

      // Then: should decode successfully
      expect(result.email).toBe('newuser@example.com')
      expect(result.password).toBe('password123')
      expect(result.orgId).toBe('org-123')
      expect(result.roleIds).toEqual(['role-1', 'role-2'])
    })

    test('should accept empty roleIds array', () => {
      // Given: user add form data with empty roleIds
      const validData = {
        email: 'user@example.com',
        password: 'password123',
        orgId: 'org-123',
        roleIds: [],
      }

      // When: decoding the data
      const result = Schema.decodeUnknownSync(userAddFormSchema)(validData)

      // Then: should decode successfully with empty array
      expect(result.roleIds).toEqual([])
    })

    test('should reject invalid email format', () => {
      // Given: user add form data with invalid email
      const invalidData = {
        email: 'invalid-email',
        password: 'password123',
        orgId: 'org-123',
        roleIds: ['role-1'],
      }

      // When: decoding the data
      // Then: should throw validation error
      expect(() => {
        Schema.decodeUnknownSync(userAddFormSchema)(invalidData)
      }).toThrow()
    })

    test('should reject password shorter than 8 characters', () => {
      // Given: user add form data with short password
      const invalidData = {
        email: 'user@example.com',
        password: 'short',
        orgId: 'org-123',
        roleIds: ['role-1'],
      }

      // When: decoding the data
      // Then: should throw validation error
      expect(() => {
        Schema.decodeUnknownSync(userAddFormSchema)(invalidData)
      }).toThrow()
    })

    test('should reject empty orgId', () => {
      // Given: user add form data with empty orgId
      const invalidData = {
        email: 'user@example.com',
        password: 'password123',
        orgId: '',
        roleIds: ['role-1'],
      }

      // When: decoding the data
      // Then: should throw validation error
      expect(() => {
        Schema.decodeUnknownSync(userAddFormSchema)(invalidData)
      }).toThrow()
    })

    test('should reject missing orgId', () => {
      // Given: user add form data without orgId
      const invalidData = {
        email: 'user@example.com',
        password: 'password123',
        roleIds: ['role-1'],
      }

      // When: decoding the data
      // Then: should throw validation error
      expect(() => {
        Schema.decodeUnknownSync(userAddFormSchema)(invalidData)
      }).toThrow()
    })

    test('should accept single roleId', () => {
      // Given: user add form data with single roleId
      const validData = {
        email: 'user@example.com',
        password: 'password123',
        orgId: 'org-123',
        roleIds: ['role-1'],
      }

      // When: decoding the data
      const result = Schema.decodeUnknownSync(userAddFormSchema)(validData)

      // Then: should decode successfully
      expect(result.roleIds).toEqual(['role-1'])
    })
  })

  describe('userAddFormSchemaStrict behavior', () => {
    test('should validate valid user add form data with at least one role', () => {
      // Given: valid user add form data with roles
      const validData = {
        email: 'newuser@example.com',
        password: 'password123',
        orgId: 'org-123',
        roleIds: ['role-1'],
      }

      // When: decoding the data
      const result = Schema.decodeUnknownSync(userAddFormSchemaStrict)(
        validData,
      )

      // Then: should decode successfully
      expect(result.roleIds).toEqual(['role-1'])
    })

    test('should reject empty roleIds array', () => {
      // Given: user add form data with empty roleIds
      const invalidData = {
        email: 'user@example.com',
        password: 'password123',
        orgId: 'org-123',
        roleIds: [],
      }

      // When: decoding the data
      // Then: should throw validation error
      expect(() => {
        Schema.decodeUnknownSync(userAddFormSchemaStrict)(invalidData)
      }).toThrow()
    })

    test('should accept multiple roleIds', () => {
      // Given: user add form data with multiple roleIds
      const validData = {
        email: 'user@example.com',
        password: 'password123',
        orgId: 'org-123',
        roleIds: ['role-1', 'role-2', 'role-3'],
      }

      // When: decoding the data
      const result = Schema.decodeUnknownSync(userAddFormSchemaStrict)(
        validData,
      )

      // Then: should decode successfully
      expect(result.roleIds).toEqual(['role-1', 'role-2', 'role-3'])
    })

    test('should validate all other fields same as userAddFormSchema', () => {
      // Given: valid user add form data
      const validData = {
        email: 'user@example.com',
        password: 'password123',
        orgId: 'org-123',
        roleIds: ['role-1'],
      }

      // When: decoding the data
      const result = Schema.decodeUnknownSync(userAddFormSchemaStrict)(
        validData,
      )

      // Then: should validate email, password, and orgId
      expect(result.email).toBe('user@example.com')
      expect(result.password).toBe('password123')
      expect(result.orgId).toBe('org-123')
    })
  })

  describe('userEditFormSchema behavior', () => {
    test('should validate valid user edit form data', () => {
      // Given: valid user edit form data
      const validData = {
        email: 'updated@example.com',
        active: true,
      }

      // When: decoding the data
      const result = Schema.decodeUnknownSync(userEditFormSchema)(validData)

      // Then: should decode successfully
      expect(result.email).toBe('updated@example.com')
      expect(result.active).toBe(true)
    })

    test('should accept active: false', () => {
      // Given: user edit form data with active: false
      const validData = {
        email: 'user@example.com',
        active: false,
      }

      // When: decoding the data
      const result = Schema.decodeUnknownSync(userEditFormSchema)(validData)

      // Then: should decode successfully
      expect(result.active).toBe(false)
    })

    test('should reject empty email', () => {
      // Given: user edit form data with empty email
      const invalidData = {
        email: '',
        active: true,
      }

      // When: decoding the data
      // Then: should throw validation error
      expect(() => {
        Schema.decodeUnknownSync(userEditFormSchema)(invalidData)
      }).toThrow()
    })

    test('should reject email with only whitespace', () => {
      // Given: user edit form data with whitespace-only email
      const invalidData = {
        email: '   ',
        active: true,
      }

      // When: decoding the data
      // Then: should throw validation error
      expect(() => {
        Schema.decodeUnknownSync(userEditFormSchema)(invalidData)
      }).toThrow()
    })

    test('should reject missing email', () => {
      // Given: user edit form data without email
      const invalidData = {
        active: true,
      }

      // When: decoding the data
      // Then: should throw validation error
      expect(() => {
        Schema.decodeUnknownSync(userEditFormSchema)(invalidData)
      }).toThrow()
    })

    test('should reject missing active field', () => {
      // Given: user edit form data without active
      const invalidData = {
        email: 'user@example.com',
      }

      // When: decoding the data
      // Then: should throw validation error
      expect(() => {
        Schema.decodeUnknownSync(userEditFormSchema)(invalidData)
      }).toThrow()
    })

    test('should reject email with whitespace (NonEmptyTrimmedString validates no whitespace)', () => {
      // Given: user edit form data with email containing whitespace
      // Note: NonEmptyTrimmedString validates that string has no leading/trailing whitespace
      const invalidData = {
        email: '  user@example.com  ',
        active: true,
      }

      // When: decoding the data
      // Then: should throw validation error (whitespace not allowed)
      expect(() => {
        Schema.decodeUnknownSync(userEditFormSchema)(invalidData)
      }).toThrow()
    })
  })

  describe('type exports behavior', () => {
    test('should export LoginFormValues type', () => {
      // Given: loginFormSchema
      // When: checking type export
      // Then: LoginFormValues should be defined
      const data: LoginFormValues = {
        email: 'test@example.com',
        password: 'password',
      }
      expect(data.email).toBe('test@example.com')
    })

    test('should export RegisterFormValues type', () => {
      // Given: registerFormSchema
      // When: checking type export
      // Then: RegisterFormValues should be defined
      const data: RegisterFormValues = {
        email: 'test@example.com',
        password: 'password123',
      }
      expect(data.email).toBe('test@example.com')
    })

    test('should export UserAddFormValues type', () => {
      // Given: userAddFormSchema
      // When: checking type export
      // Then: UserAddFormValues should be defined
      const data: UserAddFormValues = {
        email: 'test@example.com',
        password: 'password123',
        orgId: 'org-123',
        roleIds: ['role-1'],
      }
      expect(data.email).toBe('test@example.com')
    })

    test('should export UserAddFormValuesStrict type', () => {
      // Given: userAddFormSchemaStrict
      // When: checking type export
      // Then: UserAddFormValuesStrict should be defined
      const data: UserAddFormValuesStrict = {
        email: 'test@example.com',
        password: 'password123',
        orgId: 'org-123',
        roleIds: ['role-1'],
      }
      expect(data.roleIds.length).toBeGreaterThan(0)
    })

    test('should export UserEditFormValues type', () => {
      // Given: userEditFormSchema
      // When: checking type export
      // Then: UserEditFormValues should be defined
      const data: UserEditFormValues = {
        email: 'test@example.com',
        active: true,
      }
      expect(data.email).toBe('test@example.com')
    })
  })

  describe('edge cases', () => {
    test('should handle email with subdomain', () => {
      // Given: form data with email containing subdomain
      const validData = {
        email: 'user@mail.example.com',
        password: 'password123',
      }

      // When: decoding the data
      const result = Schema.decodeUnknownSync(loginFormSchema)(validData)

      // Then: should decode successfully
      expect(result.email).toBe('user@mail.example.com')
    })

    test('should handle password with special characters', () => {
      // Given: form data with password containing special characters
      const validData = {
        email: 'user@example.com',
        password: 'p@ssw0rd!123',
      }

      // When: decoding the data
      const result = Schema.decodeUnknownSync(registerFormSchema)(validData)

      // Then: should decode successfully
      expect(result.password).toBe('p@ssw0rd!123')
    })

    test('should handle UUID format orgId', () => {
      // Given: user add form data with UUID format orgId
      const validData = {
        email: 'user@example.com',
        password: 'password123',
        orgId: '550e8400-e29b-41d4-a716-446655440000',
        roleIds: ['role-1'],
      }

      // When: decoding the data
      const result = Schema.decodeUnknownSync(userAddFormSchema)(validData)

      // Then: should decode successfully
      expect(result.orgId).toBe('550e8400-e29b-41d4-a716-446655440000')
    })

    test('should handle multiple roleIds with various formats', () => {
      // Given: user add form data with multiple roleIds
      const validData = {
        email: 'user@example.com',
        password: 'password123',
        orgId: 'org-123',
        roleIds: ['role-1', 'role-2', '550e8400-e29b-41d4-a716-446655440000'],
      }

      // When: decoding the data
      const result = Schema.decodeUnknownSync(userAddFormSchema)(validData)

      // Then: should decode successfully
      expect(result.roleIds).toHaveLength(3)
    })
  })
})
