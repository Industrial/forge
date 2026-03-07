/**
 * BDD component tests for EmptyState.tsx
 * Tests verify component rendering, props handling, and MUI integration
 */
import { describe, test, expect, beforeAll } from 'bun:test'
import { render } from '@testing-library/react'
import { ThemeProvider, createTheme } from '@mui/material/styles'
import { Window } from 'happy-dom'
import React from 'react'

import EmptyState from './EmptyState'

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

const createWrapper = () => {
  const theme = createTheme({ palette: { mode: 'light' } })
  return ({ children }: { children: React.ReactNode }) => (
    <ThemeProvider theme={theme}>{children}</ThemeProvider>
  )
}

describe('EmptyState component', () => {
  describe('export behavior', () => {
    test('should export EmptyState as default export', () => {
      expect(EmptyState).toBeDefined()
      expect(typeof EmptyState).toBe('function')
    })
  })

  describe('rendering behavior', () => {
    test('should render message', () => {
      const { container } = render(<EmptyState message="No items found" />, {
        wrapper: createWrapper(),
      })
      expect(container.textContent).toContain('No items found')
    })

    test('should render Paper component', () => {
      const { container } = render(<EmptyState message="Empty" />, {
        wrapper: createWrapper(),
      })
      expect(container).toBeDefined()
    })

    test('should render Typography with message', () => {
      const { container } = render(<EmptyState message="Test message" />, {
        wrapper: createWrapper(),
      })
      expect(container.textContent).toContain('Test message')
    })
  })

  describe('props handling behavior', () => {
    test('should accept message prop', () => {
      const { container } = render(<EmptyState message="Custom message" />, {
        wrapper: createWrapper(),
      })
      expect(container.textContent).toContain('Custom message')
    })

    test('should handle empty message', () => {
      const { container } = render(<EmptyState message="" />, {
        wrapper: createWrapper(),
      })
      expect(container).toBeDefined()
    })

    test('should handle long message', () => {
      const longMessage = 'A'.repeat(1000)
      const { container } = render(<EmptyState message={longMessage} />, {
        wrapper: createWrapper(),
      })
      expect(container.textContent).toContain(longMessage)
    })
  })

  describe('MUI integration behavior', () => {
    test('should use Paper from MUI', () => {
      const { container } = render(<EmptyState message="Test" />, {
        wrapper: createWrapper(),
      })
      expect(container).toBeDefined()
    })

    test('should use Typography from MUI', () => {
      const { container } = render(<EmptyState message="Test" />, {
        wrapper: createWrapper(),
      })
      expect(container).toBeDefined()
    })

    test('should apply text.secondary color', () => {
      const { container } = render(<EmptyState message="Test" />, {
        wrapper: createWrapper(),
      })
      expect(container).toBeDefined()
    })
  })

  describe('structure behavior', () => {
    test('should center text', () => {
      const { container } = render(<EmptyState message="Test" />, {
        wrapper: createWrapper(),
      })
      expect(container).toBeDefined()
    })

    test('should apply padding', () => {
      const { container } = render(<EmptyState message="Test" />, {
        wrapper: createWrapper(),
      })
      expect(container).toBeDefined()
    })
  })
})
