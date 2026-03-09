import { describe, test, expect } from 'bun:test'
import { parseError } from './parseError'

describe('parseError', () => {
  test('returns message when body has message string', () => {
    expect(parseError({ message: 'Custom error' })).toBe('Custom error')
  })

  test('returns error when body has error string', () => {
    expect(parseError({ error: 'API error' })).toBe('API error')
  })

  test('returns String(error) when body has error non-string', () => {
    expect(parseError({ error: 123 })).toBe('123')
  })

  test('returns default when body is null', () => {
    expect(parseError(null)).toBe('Request failed.')
  })

  test('returns default when body is not an object', () => {
    expect(parseError('string')).toBe('Request failed.')
  })

  test('returns default when body has no message or error', () => {
    expect(parseError({ foo: 'bar' })).toBe('Request failed.')
  })
})
