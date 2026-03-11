/**
 * BDD tests for useColorSchemeMode hook
 * Tests verify localStorage persistence and state management behavior
 */
import {
  describe,
  test,
  expect,
  beforeEach,
  beforeAll,
  afterEach,
} from 'bun:test'
import { renderHook, cleanup, act } from '@testing-library/react'
import { useColorSchemeMode } from './useColorScheme'
import { Window } from 'happy-dom'

const STORAGE_KEY = 'mui-color-scheme'

// Set up DOM environment for tests
beforeAll(() => {
  const window = new Window()
  const document = window.document
  // @ts-expect-error - Setting global DOM APIs for bun test environment
  globalThis.window = window
  // @ts-expect-error - Setting global DOM APIs for bun test environment
  globalThis.document = document
  // @ts-expect-error - Setting global DOM APIs for bun test environment
  globalThis.localStorage = window.localStorage
})

afterEach(() => {
  cleanup()
})

describe('useColorSchemeMode', () => {
  beforeEach(() => {
    // Clear localStorage before each test
    if (typeof window !== 'undefined') {
      localStorage.clear()
    }
  })

  describe('initialization behavior', () => {
    test('should default to dark mode when no localStorage value exists', () => {
      // Given: no value in localStorage
      if (typeof window !== 'undefined') {
        localStorage.removeItem(STORAGE_KEY)
      }

      // When: hook is initialized
      const { result } = renderHook(() => useColorSchemeMode())

      // Then: should default to dark mode
      expect(result.current[0]).toBe('dark')
      expect(result.current[1]).toBeDefined()
      expect(typeof result.current[1]).toBe('function')
    })

    test('should initialize from localStorage when light value exists', () => {
      // Given: light mode stored in localStorage
      if (typeof window !== 'undefined') {
        localStorage.setItem(STORAGE_KEY, 'light')
      }

      // When: hook is initialized
      const { result } = renderHook(() => useColorSchemeMode())

      // Then: should use light mode from localStorage
      expect(result.current[0]).toBe('light')
    })

    test('should initialize from localStorage when dark value exists', () => {
      // Given: dark mode stored in localStorage
      if (typeof window !== 'undefined') {
        localStorage.setItem(STORAGE_KEY, 'dark')
      }

      // When: hook is initialized
      const { result } = renderHook(() => useColorSchemeMode())

      // Then: should use dark mode from localStorage
      expect(result.current[0]).toBe('dark')
    })

    test('should default to dark when localStorage has invalid value', () => {
      // Given: invalid value in localStorage
      if (typeof window !== 'undefined') {
        localStorage.setItem(STORAGE_KEY, 'invalid')
      }

      // When: hook is initialized
      const { result } = renderHook(() => useColorSchemeMode())

      // Then: should default to dark mode
      expect(result.current[0]).toBe('dark')
    })
  })

  describe('state update behavior', () => {
    test('should update state when setMode is called', () => {
      // Given: hook initialized with dark mode
      if (typeof window !== 'undefined') {
        localStorage.setItem(STORAGE_KEY, 'dark')
      }

      const { result } = renderHook(() => useColorSchemeMode())

      expect(result.current[0]).toBe('dark')

      // When: setMode is called with light
      act(() => {
        result.current[1]('light')
      })

      // Then: mode should be updated to light
      expect(result.current[0]).toBe('light')
    })

    test('should update state when setMode is called with function updater', () => {
      // Given: hook initialized with dark mode
      if (typeof window !== 'undefined') {
        localStorage.setItem(STORAGE_KEY, 'dark')
      }

      const { result } = renderHook(() => useColorSchemeMode())

      expect(result.current[0]).toBe('dark')

      // When: setMode is called with function updater
      act(() => {
        result.current[1]((prev) => (prev === 'dark' ? 'light' : 'dark'))
      })

      // Then: mode should be updated to light
      expect(result.current[0]).toBe('light')
    })
  })

  describe('localStorage persistence behavior', () => {
    test('should persist to localStorage when mode changes', () => {
      // Given: hook initialized with dark mode
      if (typeof window !== 'undefined') {
        localStorage.setItem(STORAGE_KEY, 'dark')
      }

      const { result } = renderHook(() => useColorSchemeMode())

      // When: setMode is called with light
      act(() => {
        result.current[1]('light')
      })

      // Then: localStorage should be updated
      if (typeof window !== 'undefined') {
        expect(localStorage.getItem(STORAGE_KEY)).toBe('light')
      }
    })

    test('should persist to localStorage when mode changes via function updater', () => {
      // Given: hook initialized with dark mode
      if (typeof window !== 'undefined') {
        localStorage.setItem(STORAGE_KEY, 'dark')
      }

      const { result } = renderHook(() => useColorSchemeMode())

      // When: setMode is called with function updater
      act(() => {
        result.current[1]((prev) => (prev === 'dark' ? 'light' : 'dark'))
      })

      // Then: localStorage should be updated
      if (typeof window !== 'undefined') {
        expect(localStorage.getItem(STORAGE_KEY)).toBe('light')
      }
    })
  })

  describe('return value structure', () => {
    test('should return tuple with mode and setMode', () => {
      // Given: hook is used
      const { result } = renderHook(() => useColorSchemeMode())

      // Then: should return tuple [mode, setMode]
      expect(result.current).toBeDefined()
      expect(Array.isArray(result.current)).toBe(true)
      expect(result.current.length).toBe(2)
      expect(result.current[0]).toBeDefined()
      expect(typeof result.current[0]).toBe('string')
      expect(['light', 'dark']).toContain(result.current[0])
      expect(result.current[1]).toBeDefined()
      expect(typeof result.current[1]).toBe('function')
    })
  })

  describe('edge cases', () => {
    test('should handle rapid state changes', () => {
      // Given: hook initialized
      const { result } = renderHook(() => useColorSchemeMode())

      // When: rapid state changes occur
      act(() => {
        result.current[1]('light')
        result.current[1]('dark')
        result.current[1]('light')
      })

      // Then: should have final state
      expect(result.current[0]).toBe('light')
      if (typeof window !== 'undefined') {
        expect(localStorage.getItem(STORAGE_KEY)).toBe('light')
      }
    })

    test('should handle empty localStorage gracefully', () => {
      // Given: localStorage is empty
      if (typeof window !== 'undefined') {
        localStorage.removeItem(STORAGE_KEY)
      }

      // When: hook is initialized
      const { result } = renderHook(() => useColorSchemeMode())

      // Then: should default to dark
      expect(result.current[0]).toBe('dark')
    })
  })
})
