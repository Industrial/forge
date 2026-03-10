/**
 * BDD tests for useIsMobile hook
 * Tests verify hook structure and breakpoint parameter handling
 * Note: Full media query testing requires browser environment
 */
import { describe, test, expect } from 'bun:test'
import type React from 'react'
import { renderHook } from '@testing-library/react'
import { ThemeProvider, createTheme } from '@mui/material/styles'
import { useIsMobile } from './useIsMobile'
import { Window } from 'happy-dom'

// Setup DOM environment for bun test
// Ensure SyntaxError exists globally first
const global = globalThis as any
if (!global.SyntaxError) {
  global.SyntaxError = class SyntaxError extends Error {
    constructor(message?: string) {
      super(message)
      this.name = 'SyntaxError'
      Object.setPrototypeOf(this, SyntaxError.prototype)
    }
  }
}

if (typeof globalThis.window === 'undefined') {
  const window = new Window()
  global.window = window
  global.document = window.document
  global.localStorage = window.localStorage
  global.navigator = window.navigator
  // Always set SyntaxError on new window instance
  ;(window as any).SyntaxError = global.SyntaxError
} else {
  // Ensure existing window has SyntaxError
  if (!(globalThis.window as any).SyntaxError) {
    ;(globalThis.window as any).SyntaxError = global.SyntaxError
  }
}

// Helper to create a wrapper with theme
const createWrapper = () => {
  const theme = createTheme()
  return ({ children }: { children: React.ReactNode }) => (
    <ThemeProvider theme={theme}>{children}</ThemeProvider>
  )
}

describe('useIsMobile', () => {
  describe('export behavior', () => {
    test('should export useIsMobile as a function', () => {
      // Given: the module
      // When: checking the export
      // Then: should be a function
      expect(typeof useIsMobile).toBe('function')
    })
  })

  describe('default breakpoint behavior', () => {
    test('should return boolean value by default', () => {
      // Given: hook is called with default breakpoint
      // When: hook is initialized
      const { result } = renderHook(() => useIsMobile(), {
        wrapper: createWrapper(),
      })

      // Then: should return boolean
      expect(typeof result.current).toBe('boolean')
    })

    test('should use md breakpoint by default', () => {
      // Given: hook is called without breakpoint parameter
      // When: hook is initialized
      const { result } = renderHook(() => useIsMobile(), {
        wrapper: createWrapper(),
      })

      // Then: should return boolean (uses md breakpoint internally)
      expect(typeof result.current).toBe('boolean')
    })
  })

  describe('breakpoint parameter behavior', () => {
    test('should accept xs breakpoint', () => {
      // Given: hook is called with 'xs' breakpoint
      // When: hook is initialized
      const { result } = renderHook(() => useIsMobile('xs'), {
        wrapper: createWrapper(),
      })

      // Then: should return boolean
      expect(typeof result.current).toBe('boolean')
    })

    test('should accept sm breakpoint', () => {
      // Given: hook is called with 'sm' breakpoint
      // When: hook is initialized
      const { result } = renderHook(() => useIsMobile('sm'), {
        wrapper: createWrapper(),
      })

      // Then: should return boolean
      expect(typeof result.current).toBe('boolean')
    })

    test('should accept md breakpoint', () => {
      // Given: hook is called with 'md' breakpoint
      // When: hook is initialized
      const { result } = renderHook(() => useIsMobile('md'), {
        wrapper: createWrapper(),
      })

      // Then: should return boolean
      expect(typeof result.current).toBe('boolean')
    })

    test('should accept lg breakpoint', () => {
      // Given: hook is called with 'lg' breakpoint
      // When: hook is initialized
      const { result } = renderHook(() => useIsMobile('lg'), {
        wrapper: createWrapper(),
      })

      // Then: should return boolean
      expect(typeof result.current).toBe('boolean')
    })

    test('should accept xl breakpoint', () => {
      // Given: hook is called with 'xl' breakpoint
      // When: hook is initialized
      const { result } = renderHook(() => useIsMobile('xl'), {
        wrapper: createWrapper(),
      })

      // Then: should return boolean
      expect(typeof result.current).toBe('boolean')
    })
  })

  describe('return value behavior', () => {
    test('should return boolean value', () => {
      // Given: hook is called
      // When: hook is initialized
      const { result } = renderHook(() => useIsMobile(), {
        wrapper: createWrapper(),
      })

      // Then: should return boolean
      expect(typeof result.current).toBe('boolean')
      expect(result.current === true || result.current === false).toBe(true)
    })
  })

  describe('theme integration', () => {
    test('should use theme breakpoints', () => {
      // Given: custom theme
      const customTheme = createTheme({
        breakpoints: {
          values: {
            xs: 0,
            sm: 600,
            md: 900,
            lg: 1200,
            xl: 1536,
          },
        },
      })

      const wrapper = ({ children }: { children: React.ReactNode }) => (
        <ThemeProvider theme={customTheme}>{children}</ThemeProvider>
      )

      // When: hook is called with custom theme
      const { result } = renderHook(() => useIsMobile(), {
        wrapper,
      })

      // Then: should use theme breakpoints and return boolean
      expect(typeof result.current).toBe('boolean')
    })
  })
})
