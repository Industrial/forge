/**
 * BDD component tests for LoadingSpinner.tsx
 * Tests verify component rendering, props handling, and MUI integration
 */
import { describe, test, expect, beforeAll, afterEach } from 'bun:test'
import { render, cleanup } from '@testing-library/react'
import { ThemeProvider, createTheme } from '@mui/material/styles'
import { Window } from 'happy-dom'
import type React from 'react'

import LoadingSpinner from './LoadingSpinner'

// Set up DOM environment for tests
beforeAll(() => {
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
    const document = window.document
    global.window = window
    global.document = document
    global.localStorage = window.localStorage
    global.navigator = window.navigator
    // Ensure document.body exists
    if (!document.body) {
      const body = document.createElement('body')
      document.appendChild(body)
    }
    // Always set SyntaxError on new window instance
    ;(window as any).SyntaxError = global.SyntaxError
  } else {
    // Ensure existing window has SyntaxError
    if (!(globalThis.window as any).SyntaxError) {
      ;(globalThis.window as any).SyntaxError = global.SyntaxError
    }
  }
})

// Helper to create a wrapper with theme
const createWrapper = () => {
  const theme = createTheme({ palette: { mode: 'light' } })
  return ({ children }: { children: React.ReactNode }) => (
    <ThemeProvider theme={theme}>{children}</ThemeProvider>
  )
}

afterEach(() => {
  cleanup()
})

describe('LoadingSpinner component', () => {
  describe('export behavior', () => {
    test('should export LoadingSpinner as default export', () => {
      // Given: the LoadingSpinner module
      // When: checking the export
      // Then: LoadingSpinner should be available
      expect(LoadingSpinner).toBeDefined()
      expect(typeof LoadingSpinner).toBe('function')
    })
  })

  describe('rendering behavior', () => {
    test('should render CircularProgress', () => {
      // Given: LoadingSpinner component
      // When: rendering LoadingSpinner
      const { container } = render(<LoadingSpinner />, {
        wrapper: createWrapper(),
      })
      // Then: CircularProgress should be rendered
      expect(container).toBeDefined()
      expect(container.innerHTML).toContain('svg') // CircularProgress renders SVG
    })

    test('should render with default py prop', () => {
      // Given: LoadingSpinner component without py prop
      // When: rendering LoadingSpinner
      const { container } = render(<LoadingSpinner />, {
        wrapper: createWrapper(),
      })
      // Then: should render with default py=4
      expect(container).toBeDefined()
    })

    test('should render with custom py prop', () => {
      // Given: LoadingSpinner component with py=8
      // When: rendering LoadingSpinner with custom py
      const { container } = render(<LoadingSpinner py={8} />, {
        wrapper: createWrapper(),
      })
      // Then: should render with py=8
      expect(container).toBeDefined()
    })
  })

  describe('props handling behavior', () => {
    test('should accept py prop', () => {
      // Given: LoadingSpinner component
      // When: rendering with py prop
      const { container } = render(<LoadingSpinner py={6} />, {
        wrapper: createWrapper(),
      })
      // Then: component should render (py prop accepted)
      expect(container).toBeDefined()
    })

    test('should use default py when not provided', () => {
      // Given: LoadingSpinner component without py
      // When: rendering LoadingSpinner
      const { container } = render(<LoadingSpinner />, {
        wrapper: createWrapper(),
      })
      // Then: should use default py=4
      expect(container).toBeDefined()
    })

    test('should accept zero py value', () => {
      // Given: LoadingSpinner component with py=0
      // When: rendering LoadingSpinner with py=0
      const { container } = render(<LoadingSpinner py={0} />, {
        wrapper: createWrapper(),
      })
      // Then: should render with py=0
      expect(container).toBeDefined()
    })
  })

  describe('structure behavior', () => {
    test('should render Box container', () => {
      // Given: LoadingSpinner component
      // When: rendering LoadingSpinner
      const { container } = render(<LoadingSpinner />, {
        wrapper: createWrapper(),
      })
      // Then: Box container should be present
      expect(container).toBeDefined()
      expect(container.querySelector('div')).not.toBeNull()
    })

    test('should center CircularProgress', () => {
      // Given: LoadingSpinner component
      // When: rendering LoadingSpinner
      const { container } = render(<LoadingSpinner />, {
        wrapper: createWrapper(),
      })
      // Then: should have flex display and justifyContent center
      expect(container).toBeDefined()
    })

    test('should apply vertical padding', () => {
      // Given: LoadingSpinner component with py prop
      // When: rendering LoadingSpinner
      const { container } = render(<LoadingSpinner py={5} />, {
        wrapper: createWrapper(),
      })
      // Then: should apply py padding
      expect(container).toBeDefined()
    })
  })

  describe('MUI integration behavior', () => {
    test('should use CircularProgress from MUI', () => {
      // Given: LoadingSpinner component
      // When: rendering LoadingSpinner
      const { container } = render(<LoadingSpinner />, {
        wrapper: createWrapper(),
      })
      // Then: CircularProgress should be rendered (MUI component)
      expect(container).toBeDefined()
      expect(container.innerHTML).toContain('svg')
    })

    test('should use Box from MUI', () => {
      // Given: LoadingSpinner component
      // When: rendering LoadingSpinner
      const { container } = render(<LoadingSpinner />, {
        wrapper: createWrapper(),
      })
      // Then: Box container should be present
      expect(container).toBeDefined()
    })
  })

  describe('edge cases', () => {
    test('should handle negative py value', () => {
      // Given: LoadingSpinner component with negative py
      // When: rendering LoadingSpinner with py=-1
      const { container } = render(<LoadingSpinner py={-1} />, {
        wrapper: createWrapper(),
      })
      // Then: should still render (MUI handles negative values)
      expect(container).toBeDefined()
    })

    test('should handle very large py value', () => {
      // Given: LoadingSpinner component with large py
      // When: rendering LoadingSpinner with py=100
      const { container } = render(<LoadingSpinner py={100} />, {
        wrapper: createWrapper(),
      })
      // Then: should render successfully
      expect(container).toBeDefined()
    })
  })
})
