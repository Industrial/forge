/**
 * BDD-style unit tests for schemas index using bun:test.
 * Tests verify that all exports are properly re-exported and accessible.
 * Tests follow Given-When-Then pattern.
 */

import { describe, it, expect } from 'bun:test'
import {
  // Fragments exports
  nonEmptyTrimmedString,
  emailSchema,
  passwordMin8Schema,
  orgIdSchema,
  roleIdsSchema,
  optionalTrimmedString,
  // User form schemas exports
  loginFormSchema,
  registerFormSchema,
  userAddFormSchema,
  userAddFormSchemaStrict,
  userEditFormSchema,
  // Types
  type LoginFormValues,
  type RegisterFormValues,
  type UserAddFormValues,
  type UserAddFormValuesStrict,
  type UserEditFormValues,
} from './index'
import { Schema } from 'effect'

describe('schemas index', () => {
  describe('fragments exports', () => {
    it('should export nonEmptyTrimmedString', () => {
      // Given: the index module
      // When: I check if nonEmptyTrimmedString is exported
      // Then: it should be defined
      expect(nonEmptyTrimmedString).toBeDefined()
    })

    it('should export emailSchema', () => {
      // Given: the index module
      // When: I check if emailSchema is exported
      // Then: it should be defined
      expect(emailSchema).toBeDefined()
    })

    it('should export passwordMin8Schema', () => {
      // Given: the index module
      // When: I check if passwordMin8Schema is exported
      // Then: it should be defined
      expect(passwordMin8Schema).toBeDefined()
    })

    it('should export orgIdSchema', () => {
      // Given: the index module
      // When: I check if orgIdSchema is exported
      // Then: it should be defined
      expect(orgIdSchema).toBeDefined()
    })

    it('should export roleIdsSchema', () => {
      // Given: the index module
      // When: I check if roleIdsSchema is exported
      // Then: it should be defined
      expect(roleIdsSchema).toBeDefined()
    })

    it('should export optionalTrimmedString', () => {
      // Given: the index module
      // When: I check if optionalTrimmedString is exported
      // Then: it should be defined
      expect(optionalTrimmedString).toBeDefined()
    })
  })

  describe('user form schemas exports', () => {
    it('should export loginFormSchema', () => {
      // Given: the index module
      // When: I check if loginFormSchema is exported
      // Then: it should be defined
      expect(loginFormSchema).toBeDefined()
    })

    it('should export registerFormSchema', () => {
      // Given: the index module
      // When: I check if registerFormSchema is exported
      // Then: it should be defined
      expect(registerFormSchema).toBeDefined()
    })

    it('should export userAddFormSchema', () => {
      // Given: the index module
      // When: I check if userAddFormSchema is exported
      // Then: it should be defined
      expect(userAddFormSchema).toBeDefined()
    })

    it('should export userAddFormSchemaStrict', () => {
      // Given: the index module
      // When: I check if userAddFormSchemaStrict is exported
      // Then: it should be defined
      expect(userAddFormSchemaStrict).toBeDefined()
    })

    it('should export userEditFormSchema', () => {
      // Given: the index module
      // When: I check if userEditFormSchema is exported
      // Then: it should be defined
      expect(userEditFormSchema).toBeDefined()
    })
  })

  describe('type exports', () => {
    it('should export LoginFormValues type', () => {
      // Given: the index module
      // When: I use LoginFormValues type
      // Then: it should be usable for type annotations
      const testValues: LoginFormValues = {
        email: 'user@example.com',
        password: 'password123',
      }
      expect(testValues.email).toBe('user@example.com')
      expect(testValues.password).toBe('password123')
    })

    it('should export RegisterFormValues type', () => {
      // Given: the index module
      // When: I use RegisterFormValues type
      // Then: it should be usable for type annotations
      const testValues: RegisterFormValues = {
        email: 'user@example.com',
        password: 'password123',
      }
      expect(testValues.email).toBe('user@example.com')
      expect(testValues.password).toBe('password123')
    })

    it('should export UserAddFormValues type', () => {
      // Given: the index module
      // When: I use UserAddFormValues type
      // Then: it should be usable for type annotations
      const testValues: UserAddFormValues = {
        email: 'user@example.com',
        password: 'password123',
        orgId: 'org-1',
        roleIds: ['role-1'],
      }
      expect(testValues.email).toBe('user@example.com')
      expect(testValues.orgId).toBe('org-1')
      expect(testValues.roleIds).toEqual(['role-1'])
    })

    it('should export UserAddFormValuesStrict type', () => {
      // Given: the index module
      // When: I use UserAddFormValuesStrict type
      // Then: it should be usable for type annotations
      const testValues: UserAddFormValuesStrict = {
        email: 'user@example.com',
        password: 'password123',
        orgId: 'org-1',
        roleIds: ['role-1'],
      }
      expect(testValues.email).toBe('user@example.com')
      expect(testValues.orgId).toBe('org-1')
      expect(testValues.roleIds).toEqual(['role-1'])
    })

    it('should export UserEditFormValues type', () => {
      // Given: the index module
      // When: I use UserEditFormValues type
      // Then: it should be usable for type annotations
      const testValues: UserEditFormValues = {
        email: 'user@example.com',
        active: true,
      }
      expect(testValues.email).toBe('user@example.com')
      expect(testValues.active).toBe(true)
    })
  })

  describe('schema validation', () => {
    it('should validate loginFormSchema', () => {
      // Given: valid login form data
      const validData = {
        email: 'user@example.com',
        password: 'password123',
      }

      // When: decoding with loginFormSchema
      const result = Schema.decodeUnknownSync(loginFormSchema)(validData)

      // Then: should decode successfully
      expect(result.email).toBe('user@example.com')
      expect(result.password).toBe('password123')
    })

    it('should validate registerFormSchema', () => {
      // Given: valid register form data
      const validData = {
        email: 'user@example.com',
        password: 'password123',
      }

      // When: decoding with registerFormSchema
      const result = Schema.decodeUnknownSync(registerFormSchema)(validData)

      // Then: should decode successfully
      expect(result.email).toBe('user@example.com')
      expect(result.password).toBe('password123')
    })

    it('should validate userAddFormSchema', () => {
      // Given: valid user add form data
      const validData = {
        email: 'user@example.com',
        password: 'password123',
        orgId: 'org-1',
        roleIds: ['role-1', 'role-2'],
      }

      // When: decoding with userAddFormSchema
      const result = Schema.decodeUnknownSync(userAddFormSchema)(validData)

      // Then: should decode successfully
      expect(result.email).toBe('user@example.com')
      expect(result.orgId).toBe('org-1')
      expect(result.roleIds).toEqual(['role-1', 'role-2'])
    })

    it('should validate userAddFormSchemaStrict with non-empty roleIds', () => {
      // Given: valid user add form data with at least one role
      const validData = {
        email: 'user@example.com',
        password: 'password123',
        orgId: 'org-1',
        roleIds: ['role-1'],
      }

      // When: decoding with userAddFormSchemaStrict
      const result = Schema.decodeUnknownSync(userAddFormSchemaStrict)(
        validData,
      )

      // Then: should decode successfully
      expect(result.email).toBe('user@example.com')
      expect(result.orgId).toBe('org-1')
      expect(result.roleIds).toEqual(['role-1'])
    })

    it('should reject userAddFormSchemaStrict with empty roleIds', () => {
      // Given: user add form data with empty roleIds
      const invalidData = {
        email: 'user@example.com',
        password: 'password123',
        orgId: 'org-1',
        roleIds: [],
      }

      // When: decoding with userAddFormSchemaStrict
      // Then: should throw an error
      expect(() => {
        Schema.decodeUnknownSync(userAddFormSchemaStrict)(invalidData)
      }).toThrow('Select at least one role')
    })

    it('should validate userEditFormSchema', () => {
      // Given: valid user edit form data
      const validData = {
        email: 'user@example.com',
        active: true,
      }

      // When: decoding with userEditFormSchema
      const result = Schema.decodeUnknownSync(userEditFormSchema)(validData)

      // Then: should decode successfully
      expect(result.email).toBe('user@example.com')
      expect(result.active).toBe(true)
    })
  })

  describe('barrel export pattern', () => {
    it('should serve as a single entry point for all schemas', () => {
      // Given: the index module
      // When: I check if it exports everything needed
      // Then: it should provide a complete API surface
      const exports = {
        // Fragments
        nonEmptyTrimmedString,
        emailSchema,
        passwordMin8Schema,
        orgIdSchema,
        roleIdsSchema,
        optionalTrimmedString,
        // Form schemas
        loginFormSchema,
        registerFormSchema,
        userAddFormSchema,
        userAddFormSchemaStrict,
        userEditFormSchema,
      }

      expect(exports.nonEmptyTrimmedString).toBeDefined()
      expect(exports.emailSchema).toBeDefined()
      expect(exports.passwordMin8Schema).toBeDefined()
      expect(exports.orgIdSchema).toBeDefined()
      expect(exports.roleIdsSchema).toBeDefined()
      expect(exports.optionalTrimmedString).toBeDefined()
      expect(exports.loginFormSchema).toBeDefined()
      expect(exports.registerFormSchema).toBeDefined()
      expect(exports.userAddFormSchema).toBeDefined()
      expect(exports.userAddFormSchemaStrict).toBeDefined()
      expect(exports.userEditFormSchema).toBeDefined()
    })

    it('should allow importing everything from a single location', () => {
      // Given: the index module
      // When: I import all exports
      // Then: all should be accessible
      // This test verifies that the barrel export pattern works correctly
      expect(nonEmptyTrimmedString).toBeDefined()
      expect(emailSchema).toBeDefined()
      expect(passwordMin8Schema).toBeDefined()
      expect(orgIdSchema).toBeDefined()
      expect(roleIdsSchema).toBeDefined()
      expect(optionalTrimmedString).toBeDefined()
      expect(loginFormSchema).toBeDefined()
      expect(registerFormSchema).toBeDefined()
      expect(userAddFormSchema).toBeDefined()
      expect(userAddFormSchemaStrict).toBeDefined()
      expect(userEditFormSchema).toBeDefined()
    })
  })

  describe('export consistency', () => {
    it('should export all fragment schemas', () => {
      // Given: the index module
      // When: I check all fragment exports
      // Then: all should be defined
      expect(nonEmptyTrimmedString).toBeDefined()
      expect(emailSchema).toBeDefined()
      expect(passwordMin8Schema).toBeDefined()
      expect(orgIdSchema).toBeDefined()
      expect(roleIdsSchema).toBeDefined()
      expect(optionalTrimmedString).toBeDefined()
    })

    it('should export all form schemas', () => {
      // Given: the index module
      // When: I check all form schema exports
      // Then: all should be defined
      expect(loginFormSchema).toBeDefined()
      expect(registerFormSchema).toBeDefined()
      expect(userAddFormSchema).toBeDefined()
      expect(userAddFormSchemaStrict).toBeDefined()
      expect(userEditFormSchema).toBeDefined()
    })

    it('should export all type definitions', () => {
      // Given: the index module
      // When: I check type exports
      // Then: all types should be usable
      // Note: Types don't exist at runtime, but TypeScript enforces this
      const loginValues: LoginFormValues = {
        email: 'test@example.com',
        password: 'test',
      }
      const registerValues: RegisterFormValues = {
        email: 'test@example.com',
        password: 'test',
      }
      const userAddValues: UserAddFormValues = {
        email: 'test@example.com',
        password: 'test',
        orgId: 'org-1',
        roleIds: [],
      }
      const userAddStrictValues: UserAddFormValuesStrict = {
        email: 'test@example.com',
        password: 'test',
        orgId: 'org-1',
        roleIds: ['role-1'],
      }
      const userEditValues: UserEditFormValues = {
        email: 'test@example.com',
        active: true,
      }

      expect(loginValues).toBeDefined()
      expect(registerValues).toBeDefined()
      expect(userAddValues).toBeDefined()
      expect(userAddStrictValues).toBeDefined()
      expect(userEditValues).toBeDefined()
    })
  })
})
