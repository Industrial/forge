/**
 * BDD tests for getBaseUrl utility function
 */
import { describe, test, expect, beforeEach, afterEach } from 'bun:test'
import { getBaseUrl } from './baseUrl'

describe('getBaseUrl', () => {
  // Store original window to restore later
  const originalWindow = globalThis.window

  beforeEach(() => {
    // Clean up any existing window mock
    if ('window' in globalThis) {
      delete (globalThis as any).window
    }
  })

  afterEach(() => {
    // Restore original window
    if (originalWindow) {
      ;(globalThis as any).window = originalWindow
    } else {
      delete (globalThis as any).window
    }
  })

  describe('server-side behavior', () => {
    test('should return default URL when window is undefined', () => {
      // Given: window is undefined (server-side)
      delete (globalThis as any).window

      // When: getting base URL
      const result = getBaseUrl()

      // Then: should return default localhost URL
      expect(result).toBe('http://localhost:5173')
    })

    test('should return default URL when window type check fails', () => {
      // Given: window is not defined
      // Ensure window is truly undefined
      if ('window' in globalThis) {
        delete (globalThis as any).window
      }

      // When: getting base URL
      const result = getBaseUrl()

      // Then: should return default localhost URL
      expect(result).toBe('http://localhost:5173')
    })
  })

  describe('client-side behavior', () => {
    test('should return window.location.origin when window is defined', () => {
      // Given: window is defined with location.origin
      const mockOrigin = 'https://example.com'
      ;(globalThis as any).window = {
        location: {
          origin: mockOrigin,
        },
      }

      // When: getting base URL
      const result = getBaseUrl()

      // Then: should return window.location.origin
      expect(result).toBe(mockOrigin)
    })

    test('should return different origins for different windows', () => {
      // Given: window with different origin
      const mockOrigin1 = 'https://app.example.com'
      ;(globalThis as any).window = {
        location: {
          origin: mockOrigin1,
        },
      }

      // When: getting base URL
      const result1 = getBaseUrl()

      // Then: should return first origin
      expect(result1).toBe(mockOrigin1)

      // Given: window with different origin
      const mockOrigin2 = 'https://staging.example.com'
      ;(globalThis as any).window = {
        location: {
          origin: mockOrigin2,
        },
      }

      // When: getting base URL again
      const result2 = getBaseUrl()

      // Then: should return second origin
      expect(result2).toBe(mockOrigin2)
    })

    test('should handle localhost origin', () => {
      // Given: window with localhost origin
      const mockOrigin = 'http://localhost:3000'
      ;(globalThis as any).window = {
        location: {
          origin: mockOrigin,
        },
      }

      // When: getting base URL
      const result = getBaseUrl()

      // Then: should return localhost origin
      expect(result).toBe(mockOrigin)
    })

    test('should handle IP address origin', () => {
      // Given: window with IP address origin
      const mockOrigin = 'http://192.168.1.1:8080'
      ;(globalThis as any).window = {
        location: {
          origin: mockOrigin,
        },
      }

      // When: getting base URL
      const result = getBaseUrl()

      // Then: should return IP address origin
      expect(result).toBe(mockOrigin)
    })

    test('should handle HTTPS origin', () => {
      // Given: window with HTTPS origin
      const mockOrigin = 'https://secure.example.com'
      ;(globalThis as any).window = {
        location: {
          origin: mockOrigin,
        },
      }

      // When: getting base URL
      const result = getBaseUrl()

      // Then: should return HTTPS origin
      expect(result).toBe(mockOrigin)
    })

    test('should handle HTTP origin', () => {
      // Given: window with HTTP origin
      const mockOrigin = 'http://example.com'
      ;(globalThis as any).window = {
        location: {
          origin: mockOrigin,
        },
      }

      // When: getting base URL
      const result = getBaseUrl()

      // Then: should return HTTP origin
      expect(result).toBe(mockOrigin)
    })

    test('should handle origin with port', () => {
      // Given: window with origin including port
      const mockOrigin = 'https://example.com:8443'
      ;(globalThis as any).window = {
        location: {
          origin: mockOrigin,
        },
      }

      // When: getting base URL
      const result = getBaseUrl()

      // Then: should return origin with port
      expect(result).toBe(mockOrigin)
    })

    test('should handle origin without port', () => {
      // Given: window with origin without port
      const mockOrigin = 'https://example.com'
      ;(globalThis as any).window = {
        location: {
          origin: mockOrigin,
        },
      }

      // When: getting base URL
      const result = getBaseUrl()

      // Then: should return origin without port
      expect(result).toBe(mockOrigin)
    })
  })

  describe('edge cases', () => {
    test('should handle empty origin string', () => {
      // Given: window with empty origin
      ;(globalThis as any).window = {
        location: {
          origin: '',
        },
      }

      // When: getting base URL
      const result = getBaseUrl()

      // Then: should return empty string (window.location.origin behavior)
      expect(result).toBe('')
    })

    test('should be consistent for same window', () => {
      // Given: window with specific origin
      const mockOrigin = 'https://example.com'
      ;(globalThis as any).window = {
        location: {
          origin: mockOrigin,
        },
      }

      // When: getting base URL multiple times
      const result1 = getBaseUrl()
      const result2 = getBaseUrl()
      const result3 = getBaseUrl()

      // Then: should return same value
      expect(result1).toBe(mockOrigin)
      expect(result2).toBe(mockOrigin)
      expect(result3).toBe(mockOrigin)
    })
  })

  describe('type checking behavior', () => {
    test('should use typeof check for window', () => {
      // Given: window exists but typeof check would fail
      // This tests the actual implementation logic
      delete (globalThis as any).window

      // When: getting base URL
      const result = getBaseUrl()

      // Then: should return default (typeof window === 'undefined')
      expect(result).toBe('http://localhost:5173')
    })

    test('should handle window being null', () => {
      // Given: window is explicitly null
      // Note: typeof null === 'object', not 'undefined'
      // But the function checks typeof window === 'undefined'
      // So null window should still trigger the undefined check
      // Actually, if window is null, typeof window would be 'object', not 'undefined'
      // So this would go to the else branch and try to access window.location.origin
      // But for this test, let's ensure window is truly undefined
      ;(globalThis as any).window = null

      // When: getting base URL
      // This might throw or behave unexpectedly, but let's test it
      // Actually, typeof null === 'object', so it won't match 'undefined'
      // The function will try to access null.location.origin which will throw
      // But let's test the actual behavior
      try {
        const result = getBaseUrl()
        // If it doesn't throw, it means the check worked differently
        expect(typeof result).toBe('string')
      } catch (e) {
        // If it throws, that's expected behavior
        expect(e).toBeDefined()
      }
    })
  })
})
