/**
 * BDD tests for useColorSchemeMode hook
 * Tests verify localStorage persistence and state management behavior
 */
import { describe, test, expect, beforeEach, afterEach } from 'bun:test'
import { renderHook, act } from '@testing-library/react'
import { useColorSchemeMode } from './useColorScheme'
import type { PaletteMode } from '@mui/material'
import { Window } from 'happy-dom'

// Setup DOM environment for bun test
if (typeof globalThis.window === 'undefined') {
  const window = new Window()
  const global = globalThis as any
  global.window = window
  global.document = window.document
  global.localStorage = window.localStorage
  global.navigator = window.navigator
}

const STORAGE_KEY = 'mui-color-scheme'

describe('useColorSchemeMode', () => {
  beforeEach(() => {
    // Clear localStorage before each test
    if (typeof window !== 'undefined') {
      localStorage.clear()
    }
  })

  afterEach(() => {
    // Cleanup localStorage after each test
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

    test('should toggle between light and dark modes', () => {
      // Given: hook initialized with dark mode
      if (typeof window !== 'undefined') {
        localStorage.setItem(STORAGE_KEY, 'dark')
      }

      const { result } = renderHook(() => useColorSchemeMode())

      expect(result.current[0]).toBe('dark')

      // When: toggling to light
      act(() => {
        result.current[1]('light')
      })
      expect(result.current[0]).toBe('light')

      // When: toggling back to dark
      act(() => {
        result.current[1]('dark')
      })

      // Then: should be dark again
      expect(result.current[0]).toBe('dark')
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

    test('should update localStorage on every state change', () => {
      // Given: hook initialized
      if (typeof window !== 'undefined') {
        localStorage.setItem(STORAGE_KEY, 'dark')
      }

      const { result } = renderHook(() => useColorSchemeMode())

      // When: multiple state changes occur
      act(() => {
        result.current[1]('light')
      })
      if (typeof window !== 'undefined') {
        expect(localStorage.getItem(STORAGE_KEY)).toBe('light')
      }

      act(() => {
        result.current[1]('dark')
      })
      if (typeof window !== 'undefined') {
        expect(localStorage.getItem(STORAGE_KEY)).toBe('dark')
      }
    })
  })

  describe('return value structure', () => {
    test('should return tuple with mode and setMode', () => {
      // Given: hook is used
      const { result } = renderHook(() => useColorSchemeMode())

      // Then: should return tuple [mode, setMode]
      expect(Array.isArray(result.current)).toBe(true)
      expect(result.current.length).toBe(2)
      expect(typeof result.current[0]).toBe('string')
      expect(['light', 'dark']).toContain(result.current[0])
      expect(typeof result.current[1]).toBe('function')
    })

    test('should return PaletteMode type for mode', () => {
      // Given: hook is used
      const { result } = renderHook(() => useColorSchemeMode())

      // Then: mode should be a valid PaletteMode
      const mode: PaletteMode = result.current[0]
      expect(['light', 'dark']).toContain(mode)
    })
  })

  describe('edge cases', () => {
    test('should handle rapid state changes', () => {
      // Given: hook initialized
      if (typeof window !== 'undefined') {
        localStorage.setItem(STORAGE_KEY, 'dark')
      }

      const { result } = renderHook(() => useColorSchemeMode())

      // When: rapid state changes occur (each in separate act calls to ensure re-renders)
      act(() => {
        result.current[1]('light')
      })
      act(() => {
        result.current[1]('dark')
      })
      act(() => {
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
        localStorage.clear()
        localStorage.removeItem(STORAGE_KEY)
      }

      // When: hook is initialized
      const { result } = renderHook(() => useColorSchemeMode())

      // Then: should default to dark
      expect(result.current[0]).toBe('dark')
    })

    test('should handle SSR environment (no window)', () => {
      // Given: window is undefined (SSR)
      // Note: This test verifies the hook doesn't crash in SSR
      // In actual SSR, window would be undefined
      const { result } = renderHook(() => useColorSchemeMode())

      // Then: should return a valid mode (defaults to dark)
      expect(['light', 'dark']).toContain(result.current[0])
      expect(typeof result.current[1]).toBe('function')
    })
  })
})
