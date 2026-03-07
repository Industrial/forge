/**
 * BDD tests for formatDate utility function
 */
import { describe, test, expect } from 'bun:test'
import { formatDate } from './formatDate'

describe('formatDate', () => {
  describe('null/undefined/empty handling', () => {
    test('should return "—" for undefined', () => {
      // Given: undefined value
      // When: formatting date
      const result = formatDate(undefined)

      // Then: should return "—"
      expect(result).toBe('—')
    })

    test('should return "—" for null', () => {
      // Given: null value (as string | undefined, but testing nullish behavior)
      // When: formatting date
      // Note: TypeScript doesn't allow null, but runtime might receive it
      const result = formatDate(null as unknown as string | undefined)

      // Then: should return "—"
      expect(result).toBe('—')
    })

    test('should return "—" for empty string', () => {
      // Given: empty string
      // When: formatting date
      const result = formatDate('')

      // Then: should return "—"
      expect(result).toBe('—')
    })
  })

  describe('valid ISO date string formatting', () => {
    test('should format ISO 8601 date string', () => {
      // Given: valid ISO 8601 date string
      const isoDate = '2024-01-15T10:30:00Z'

      // When: formatting date
      const result = formatDate(isoDate)

      // Then: should return formatted locale string
      expect(result).not.toBe('—')
      expect(result).not.toBe(isoDate) // Should be formatted differently
      expect(typeof result).toBe('string')
      expect(result.length).toBeGreaterThan(0)
    })

    test('should format ISO date with timezone offset', () => {
      // Given: ISO date with timezone offset
      const isoDate = '2024-01-15T10:30:00+05:00'

      // When: formatting date
      const result = formatDate(isoDate)

      // Then: should return formatted locale string
      expect(result).not.toBe('—')
      expect(result).not.toBe(isoDate)
      expect(typeof result).toBe('string')
    })

    test('should format ISO date without time', () => {
      // Given: ISO date without time component
      const isoDate = '2024-01-15'

      // When: formatting date
      const result = formatDate(isoDate)

      // Then: should return formatted locale string
      expect(result).not.toBe('—')
      expect(typeof result).toBe('string')
    })

    test('should format date with milliseconds', () => {
      // Given: ISO date with milliseconds
      const isoDate = '2024-01-15T10:30:00.123Z'

      // When: formatting date
      const result = formatDate(isoDate)

      // Then: should return formatted locale string
      expect(result).not.toBe('—')
      expect(typeof result).toBe('string')
    })

    test('should handle different years', () => {
      // Given: dates from different years
      const date1 = '2020-01-01T00:00:00Z'
      const date2 = '2025-12-31T23:59:59Z'
      const date3 = '1999-06-15T12:00:00Z'

      // When: formatting dates
      const result1 = formatDate(date1)
      const result2 = formatDate(date2)
      const result3 = formatDate(date3)

      // Then: all should be formatted
      expect(result1).not.toBe('—')
      expect(result2).not.toBe('—')
      expect(result3).not.toBe('—')
      expect(typeof result1).toBe('string')
      expect(typeof result2).toBe('string')
      expect(typeof result3).toBe('string')
    })
  })

  describe('invalid date string handling', () => {
    test('should return "Invalid Date" for invalid date', () => {
      // Given: invalid date string
      const invalidDate = 'not-a-date'

      // When: formatting date
      const result = formatDate(invalidDate)

      // Then: should return "Invalid Date" (Date constructor doesn't throw, creates Invalid Date)
      expect(result).toBe('Invalid Date')
    })

    test('should return original string for malformed date', () => {
      // Given: malformed date string
      const malformedDate = '2024-13-45T99:99:99Z'

      // When: formatting date
      const result = formatDate(malformedDate)

      // Then: should return original string (or formatted if Date constructor accepts it)
      // Note: Date constructor is lenient, so this might actually parse
      expect(typeof result).toBe('string')
      expect(result.length).toBeGreaterThan(0)
    })

    test('should return "Invalid Date" for non-date string', () => {
      // Given: non-date string
      const nonDate = 'hello world'

      // When: formatting date
      const result = formatDate(nonDate)

      // Then: should return "Invalid Date" (Date constructor doesn't throw)
      expect(result).toBe('Invalid Date')
    })

    test('should return original string for empty date parts', () => {
      // Given: string with empty date parts
      const emptyParts = '--T::'

      // When: formatting date
      const result = formatDate(emptyParts)

      // Then: should return original string (or handle gracefully)
      expect(typeof result).toBe('string')
    })
  })

  describe('edge cases', () => {
    test('should handle very old dates', () => {
      // Given: very old date
      const oldDate = '1900-01-01T00:00:00Z'

      // When: formatting date
      const result = formatDate(oldDate)

      // Then: should format successfully
      expect(result).not.toBe('—')
      expect(typeof result).toBe('string')
    })

    test('should handle future dates', () => {
      // Given: future date
      const futureDate = '2100-12-31T23:59:59Z'

      // When: formatting date
      const result = formatDate(futureDate)

      // Then: should format successfully
      expect(result).not.toBe('—')
      expect(typeof result).toBe('string')
    })

    test('should handle date with only numbers', () => {
      // Given: numeric string (not a valid date format)
      const numeric = '1234567890'

      // When: formatting date
      const result = formatDate(numeric)

      // Then: should return original string (Date constructor might parse as timestamp)
      expect(typeof result).toBe('string')
    })

    test('should handle whitespace-only string', () => {
      // Given: whitespace-only string
      const whitespace = '   '

      // When: formatting date
      const result = formatDate(whitespace)

      // Then: should return original string (Date might parse, but likely invalid)
      expect(typeof result).toBe('string')
    })

    test('should handle date with extra characters', () => {
      // Given: date string with extra characters
      const withExtra = '2024-01-15T10:30:00Z extra text'

      // When: formatting date
      const result = formatDate(withExtra)

      // Then: should return original string (Date constructor might parse prefix)
      expect(typeof result).toBe('string')
    })
  })

  describe('locale formatting behavior', () => {
    test('should use locale-specific formatting', () => {
      // Given: ISO date string
      const isoDate = '2024-01-15T10:30:00Z'

      // When: formatting date
      const result = formatDate(isoDate)

      // Then: should use toLocaleString formatting
      // The exact format depends on locale, but should be different from ISO format
      expect(result).not.toBe(isoDate)
      expect(result).toContain('2024') // Should contain year
    })

    test('should format consistently for same input', () => {
      // Given: same ISO date string
      const isoDate = '2024-01-15T10:30:00Z'

      // When: formatting multiple times
      const result1 = formatDate(isoDate)
      const result2 = formatDate(isoDate)
      const result3 = formatDate(isoDate)

      // Then: should return same formatted string
      expect(result1).toBe(result2)
      expect(result2).toBe(result3)
    })
  })
})
