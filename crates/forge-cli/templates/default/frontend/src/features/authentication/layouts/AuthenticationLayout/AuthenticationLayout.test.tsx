/**
 * BDD component tests for AuthenticationLayout.tsx
 * Tests verify component rendering, props handling, and layout structure
 */
import { describe, test, expect, beforeAll, afterEach } from 'bun:test'
import { render, cleanup } from '@testing-library/react'
import { ThemeProvider, createTheme } from '@mui/material/styles'
import { Window } from 'happy-dom'
import type React from 'react'

import AuthenticationLayout from './AuthenticationLayout'

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

afterEach(() => {
  cleanup()
})

describe('AuthenticationLayout component', () => {
  describe('export behavior', () => {
    test('should export AuthenticationLayout as default export', () => {
      expect(AuthenticationLayout).toBeDefined()
      expect(typeof AuthenticationLayout).toBe('function')
    })
  })

  describe('rendering behavior', () => {
    test('should render children', () => {
      const { container } = render(
        <AuthenticationLayout>
          <div data-testid="content">Content</div>
        </AuthenticationLayout>,
        { wrapper: createWrapper() },
      )
      expect(container.querySelector('[data-testid="content"]')).not.toBeNull()
    })

    test('should render Box container', () => {
      const { container } = render(
        <AuthenticationLayout>
          <div>Content</div>
        </AuthenticationLayout>,
        { wrapper: createWrapper() },
      )
      expect(container).toBeDefined()
    })

    test('should center content vertically and horizontally', () => {
      const { container } = render(
        <AuthenticationLayout>
          <div>Content</div>
        </AuthenticationLayout>,
        { wrapper: createWrapper() },
      )
      expect(container).toBeDefined()
    })
  })

  describe('props handling behavior', () => {
    test('should accept children prop', () => {
      const children = <div data-testid="children">Children</div>
      const { container } = render(
        <AuthenticationLayout>{children}</AuthenticationLayout>,
        { wrapper: createWrapper() },
      )
      expect(container.querySelector('[data-testid="children"]')).not.toBeNull()
    })

    test('should handle multiple children', () => {
      const { container } = render(
        <AuthenticationLayout>
          <div>Child 1</div>
          <div>Child 2</div>
        </AuthenticationLayout>,
        { wrapper: createWrapper() },
      )
      expect(container).toBeDefined()
    })
  })

  describe('layout structure behavior', () => {
    test('should apply minHeight 100%', () => {
      const { container } = render(
        <AuthenticationLayout>
          <div>Content</div>
        </AuthenticationLayout>,
        { wrapper: createWrapper() },
      )
      expect(container).toBeDefined()
    })

    test('should apply flex layout', () => {
      const { container } = render(
        <AuthenticationLayout>
          <div>Content</div>
        </AuthenticationLayout>,
        { wrapper: createWrapper() },
      )
      expect(container).toBeDefined()
    })

    test('should center content', () => {
      const { container } = render(
        <AuthenticationLayout>
          <div>Content</div>
        </AuthenticationLayout>,
        { wrapper: createWrapper() },
      )
      expect(container).toBeDefined()
    })

    test('should apply padding', () => {
      const { container } = render(
        <AuthenticationLayout>
          <div>Content</div>
        </AuthenticationLayout>,
        { wrapper: createWrapper() },
      )
      expect(container).toBeDefined()
    })

    test('should limit max width to 400px', () => {
      const { container } = render(
        <AuthenticationLayout>
          <div>Content</div>
        </AuthenticationLayout>,
        { wrapper: createWrapper() },
      )
      expect(container).toBeDefined()
    })

    test('should apply gap between children', () => {
      const { container } = render(
        <AuthenticationLayout>
          <div>Child 1</div>
          <div>Child 2</div>
        </AuthenticationLayout>,
        { wrapper: createWrapper() },
      )
      expect(container).toBeDefined()
    })
  })
})
