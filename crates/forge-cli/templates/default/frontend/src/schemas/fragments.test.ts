/**
 * BDD-style unit tests for schema fragments using bun:test and Effect Schema.
 * Tests verify validation behavior for reusable schema fragments.
 * Tests follow Given-When-Then pattern.
 */

import { describe, it, expect } from 'bun:test'
import { Schema } from 'effect'
import {
  nonEmptyTrimmedString,
  emailSchema,
  passwordMin8Schema,
  orgIdSchema,
  roleIdsSchema,
  optionalTrimmedString,
} from './fragments'

describe('schema fragments', () => {
  describe('nonEmptyTrimmedString', () => {
    it('should accept non-empty trimmed strings', () => {
      // Given: a non-empty trimmed string
      const valid = 'test'

      // When: decoding the value
      const result = Schema.decodeUnknownSync(nonEmptyTrimmedString)(valid)

      // Then: should decode successfully
      expect(result).toBe('test')
    })

    it('should reject strings with leading or trailing whitespace', () => {
      // Given: a string with leading and trailing whitespace
      // Note: NonEmptyTrimmedString validates that strings are already trimmed
      const withWhitespace = '  test  '

      // When: decoding the value
      // Then: should throw an error (expects already-trimmed input)
      expect(() => {
        Schema.decodeUnknownSync(nonEmptyTrimmedString)(withWhitespace)
      }).toThrow()
    })

    it('should reject empty strings', () => {
      // Given: an empty string
      const empty = ''

      // When: decoding the value
      // Then: should throw an error
      expect(() => {
        Schema.decodeUnknownSync(nonEmptyTrimmedString)(empty)
      }).toThrow()
    })

    it('should reject strings with only whitespace', () => {
      // Given: a string with only whitespace
      const whitespaceOnly = '   '

      // When: decoding the value
      // Then: should throw an error (trimmed becomes empty)
      expect(() => {
        Schema.decodeUnknownSync(nonEmptyTrimmedString)(whitespaceOnly)
      }).toThrow()
    })

    it('should accept strings with internal spaces', () => {
      // Given: a string with internal spaces
      const withSpaces = 'test string'

      // When: decoding the value
      const result = Schema.decodeUnknownSync(nonEmptyTrimmedString)(withSpaces)

      // Then: should decode successfully
      expect(result).toBe('test string')
    })
  })

  describe('emailSchema', () => {
    it('should accept valid email addresses', () => {
      // Given: valid email addresses
      const validEmails = [
        'user@example.com',
        'test.email@domain.co.uk',
        'user+tag@example.org',
        'user_name@example-domain.com',
      ]

      // When: decoding each email
      // Then: all should decode successfully
      validEmails.forEach((email) => {
        const result = Schema.decodeUnknownSync(emailSchema)(email)
        expect(result).toBe(email)
      })
    })

    it('should reject invalid email addresses', () => {
      // Given: invalid email addresses
      const invalidEmails = [
        'notanemail',
        '@example.com',
        'user@',
        'user@example',
        'user @example.com',
        'user@example .com',
        '',
        'user@@example.com',
      ]

      // When: decoding each email
      // Then: all should throw errors
      invalidEmails.forEach((email) => {
        expect(() => {
          Schema.decodeUnknownSync(emailSchema)(email)
        }).toThrow()
      })
    })

    it('should reject email addresses with leading or trailing whitespace', () => {
      // Given: an email with whitespace
      // Note: NonEmptyTrimmedString validates that strings are already trimmed
      const emailWithWhitespace = '  user@example.com  '

      // When: decoding the value
      // Then: should throw an error (expects already-trimmed input)
      expect(() => {
        Schema.decodeUnknownSync(emailSchema)(emailWithWhitespace)
      }).toThrow()
    })

    it('should reject emails with spaces', () => {
      // Given: an email with spaces
      const emailWithSpaces = 'user name@example.com'

      // When: decoding the value
      // Then: should throw an error
      expect(() => {
        Schema.decodeUnknownSync(emailSchema)(emailWithSpaces)
      }).toThrow('Enter a valid email address')
    })

    it('should require @ symbol', () => {
      // Given: a string without @ symbol
      const noAt = 'userexample.com'

      // When: decoding the value
      // Then: should throw an error
      expect(() => {
        Schema.decodeUnknownSync(emailSchema)(noAt)
      }).toThrow()
    })

    it('should require domain with dot', () => {
      // Given: an email without domain dot
      const noDot = 'user@example'

      // When: decoding the value
      // Then: should throw an error
      expect(() => {
        Schema.decodeUnknownSync(emailSchema)(noDot)
      }).toThrow()
    })
  })

  describe('passwordMin8Schema', () => {
    it('should accept passwords with 8 or more characters', () => {
      // Given: passwords with 8+ characters
      const validPasswords = [
        '12345678',
        'password',
        'verylongpassword123',
        'P@ssw0rd',
      ]

      // When: decoding each password
      // Then: all should decode successfully
      validPasswords.forEach((password) => {
        const result = Schema.decodeUnknownSync(passwordMin8Schema)(password)
        expect(result).toBe(password)
      })
    })

    it('should reject passwords with fewer than 8 characters', () => {
      // Given: passwords with less than 8 characters
      const invalidPasswords = ['', '1234567', 'short', 'pass']

      // When: decoding each password
      // Then: all should throw errors
      invalidPasswords.forEach((password) => {
        expect(() => {
          Schema.decodeUnknownSync(passwordMin8Schema)(password)
        }).toThrow()
      })
    })

    it('should reject passwords with leading or trailing whitespace', () => {
      // Given: a password with whitespace
      // Note: NonEmptyTrimmedString validates that strings are already trimmed
      const passwordWithWhitespace = '  password  '

      // When: decoding the value
      // Then: should throw an error (expects already-trimmed input)
      expect(() => {
        Schema.decodeUnknownSync(passwordMin8Schema)(passwordWithWhitespace)
      }).toThrow()
    })

    it('should reject short passwords with whitespace', () => {
      // Given: a password with whitespace that is <8 chars when trimmed
      // Note: NonEmptyTrimmedString validates that strings are already trimmed
      const shortWithWhitespace = '  12345  '

      // When: decoding the value
      // Then: should throw an error (expects already-trimmed input)
      expect(() => {
        Schema.decodeUnknownSync(passwordMin8Schema)(shortWithWhitespace)
      }).toThrow()
    })

    it('should accept exactly 8 characters', () => {
      // Given: a password with exactly 8 characters
      const exactly8 = '12345678'

      // When: decoding the value
      const result = Schema.decodeUnknownSync(passwordMin8Schema)(exactly8)

      // Then: should decode successfully
      expect(result).toBe('12345678')
    })
  })

  describe('orgIdSchema', () => {
    it('should accept non-empty trimmed strings', () => {
      // Given: a non-empty organization ID
      const orgId = 'org-123'

      // When: decoding the value
      const result = Schema.decodeUnknownSync(orgIdSchema)(orgId)

      // Then: should decode successfully
      expect(result).toBe('org-123')
    })

    it('should reject organization IDs with leading or trailing whitespace', () => {
      // Given: an organization ID with whitespace
      // Note: NonEmptyTrimmedString validates that strings are already trimmed
      const orgIdWithWhitespace = '  org-123  '

      // When: decoding the value
      // Then: should throw an error (expects already-trimmed input)
      expect(() => {
        Schema.decodeUnknownSync(orgIdSchema)(orgIdWithWhitespace)
      }).toThrow()
    })

    it('should reject empty strings', () => {
      // Given: an empty string
      const empty = ''

      // When: decoding the value
      // Then: should throw an error
      expect(() => {
        Schema.decodeUnknownSync(orgIdSchema)(empty)
      }).toThrow()
    })

    it('should reject strings with only whitespace', () => {
      // Given: a string with only whitespace
      const whitespaceOnly = '   '

      // When: decoding the value
      // Then: should throw an error
      expect(() => {
        Schema.decodeUnknownSync(orgIdSchema)(whitespaceOnly)
      }).toThrow()
    })

    it('should accept UUID strings', () => {
      // Given: a UUID string
      const uuid = '550e8400-e29b-41d4-a716-446655440000'

      // When: decoding the value
      const result = Schema.decodeUnknownSync(orgIdSchema)(uuid)

      // Then: should decode successfully
      expect(result).toBe(uuid)
    })
  })

  describe('roleIdsSchema', () => {
    it('should accept arrays of strings', () => {
      // Given: an array of role ID strings
      const roleIds = ['role-1', 'role-2', 'role-3']

      // When: decoding the value
      const result = Schema.decodeUnknownSync(roleIdsSchema)(roleIds)

      // Then: should decode successfully
      expect(result).toEqual(['role-1', 'role-2', 'role-3'])
    })

    it('should accept empty arrays', () => {
      // Given: an empty array
      const emptyArray: string[] = []

      // When: decoding the value
      const result = Schema.decodeUnknownSync(roleIdsSchema)(emptyArray)

      // Then: should decode successfully
      expect(result).toEqual([])
    })

    it('should accept arrays with single role ID', () => {
      // Given: an array with one role ID
      const singleRole = ['role-1']

      // When: decoding the value
      const result = Schema.decodeUnknownSync(roleIdsSchema)(singleRole)

      // Then: should decode successfully
      expect(result).toEqual(['role-1'])
    })

    it('should reject non-array values', () => {
      // Given: non-array values
      const invalidValues = ['role-1', 123, null, undefined, {}]

      // When: decoding each value
      // Then: all should throw errors
      invalidValues.forEach((value) => {
        expect(() => {
          Schema.decodeUnknownSync(roleIdsSchema)(value)
        }).toThrow()
      })
    })

    it('should reject arrays with non-string elements', () => {
      // Given: an array with non-string elements
      const invalidArray = ['role-1', 123, true]

      // When: decoding the value
      // Then: should throw an error
      expect(() => {
        Schema.decodeUnknownSync(roleIdsSchema)(invalidArray)
      }).toThrow()
    })

    it('should accept arrays with UUID strings', () => {
      // Given: an array of UUID strings
      const uuidArray = [
        '550e8400-e29b-41d4-a716-446655440000',
        '6ba7b810-9dad-11d1-80b4-00c04fd430c8',
      ]

      // When: decoding the value
      const result = Schema.decodeUnknownSync(roleIdsSchema)(uuidArray)

      // Then: should decode successfully
      expect(result).toEqual(uuidArray)
    })
  })

  describe('optionalTrimmedString', () => {
    it('should accept string values when used in struct', () => {
      // Given: a struct schema with optionalTrimmedString field
      const testSchema = Schema.Struct({
        required: Schema.String,
        optional: optionalTrimmedString,
      })
      const data = {
        required: 'value',
        optional: 'test',
      }

      // When: decoding the value
      const result = Schema.decodeUnknownSync(testSchema)(data)

      // Then: should decode successfully
      expect(result.optional).toBe('test')
    })

    it('should accept undefined when used in struct', () => {
      // Given: a struct schema with optionalTrimmedString field
      const testSchema = Schema.Struct({
        required: Schema.String,
        optional: optionalTrimmedString,
      })
      const data = {
        required: 'value',
      }

      // When: decoding the value
      const result = Schema.decodeUnknownSync(testSchema)(data)

      // Then: should decode to undefined
      expect(result.optional).toBeUndefined()
    })

    it('should accept empty strings when used in struct', () => {
      // Given: a struct schema with optionalTrimmedString field
      const testSchema = Schema.Struct({
        required: Schema.String,
        optional: optionalTrimmedString,
      })
      const data = {
        required: 'value',
        optional: '',
      }

      // When: decoding the value
      const result = Schema.decodeUnknownSync(testSchema)(data)

      // Then: should decode successfully (optional allows empty)
      expect(result.optional).toBe('')
    })

    it('should accept strings with spaces when used in struct', () => {
      // Given: a struct schema with optionalTrimmedString field
      const testSchema = Schema.Struct({
        required: Schema.String,
        optional: optionalTrimmedString,
      })
      const data = {
        required: 'value',
        optional: 'test string',
      }

      // When: decoding the value
      const result = Schema.decodeUnknownSync(testSchema)(data)

      // Then: should decode successfully
      expect(result.optional).toBe('test string')
    })

    it('should preserve whitespace when used in struct', () => {
      // Given: a struct schema with optionalTrimmedString field
      // Note: optionalTrimmedString uses Schema.String (not NonEmptyTrimmedString)
      // So it doesn't trim, preserves whitespace
      const testSchema = Schema.Struct({
        required: Schema.String,
        optional: optionalTrimmedString,
      })
      const data = {
        required: 'value',
        optional: '  test  ',
      }

      // When: decoding the value
      const result = Schema.decodeUnknownSync(testSchema)(data)

      // Then: should decode successfully without trimming
      expect(result.optional).toBe('  test  ')
    })
  })

  describe('schema composition', () => {
    it('should work together in form schemas', () => {
      // Given: a form schema using multiple fragments
      const formSchema = Schema.Struct({
        email: emailSchema,
        password: passwordMin8Schema,
        orgId: orgIdSchema,
        roleIds: roleIdsSchema,
        optionalField: optionalTrimmedString,
      })

      // When: decoding valid form data
      const validData = {
        email: 'user@example.com',
        password: 'password123',
        orgId: 'org-1',
        roleIds: ['role-1', 'role-2'],
        optionalField: 'optional value',
      }

      const result = Schema.decodeUnknownSync(formSchema)(validData)

      // Then: should decode successfully
      expect(result.email).toBe('user@example.com')
      expect(result.password).toBe('password123')
      expect(result.orgId).toBe('org-1')
      expect(result.roleIds).toEqual(['role-1', 'role-2'])
      expect(result.optionalField).toBe('optional value')
    })

    it('should validate all fragments in form schema', () => {
      // Given: a form schema using multiple fragments
      const formSchema = Schema.Struct({
        email: emailSchema,
        password: passwordMin8Schema,
        orgId: orgIdSchema,
        roleIds: roleIdsSchema,
      })

      // When: decoding invalid form data
      const invalidData = {
        email: 'invalid-email',
        password: 'short',
        orgId: '',
        roleIds: ['role-1'],
      }

      // Then: should throw validation errors
      expect(() => {
        Schema.decodeUnknownSync(formSchema)(invalidData)
      }).toThrow()
    })

    it('should allow optional fields to be undefined', () => {
      // Given: a form schema with optional field
      const formSchema = Schema.Struct({
        required: nonEmptyTrimmedString,
        optional: optionalTrimmedString,
      })

      // When: decoding data without optional field
      const dataWithoutOptional = {
        required: 'value',
      }

      const result = Schema.decodeUnknownSync(formSchema)(dataWithoutOptional)

      // Then: should decode successfully with undefined optional field
      expect(result.required).toBe('value')
      expect(result.optional).toBeUndefined()
    })
  })
})
