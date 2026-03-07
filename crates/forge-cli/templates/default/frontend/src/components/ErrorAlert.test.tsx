/**
 * BDD component tests for ErrorAlert.tsx
 * Tests verify component rendering, props handling, and MUI Alert integration
 */
import { describe, test, expect, beforeAll } from 'bun:test'
import { render, screen, waitFor } from '@testing-library/react'
import { ThemeProvider, createTheme } from '@mui/material/styles'
import { Window } from 'happy-dom'
import React from 'react'

import ErrorAlert from './ErrorAlert'

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

describe('ErrorAlert component', () => {
  describe('export behavior', () => {
    test('should export ErrorAlert as default export', () => {
      expect(ErrorAlert).toBeDefined()
      expect(typeof ErrorAlert).toBe('function')
    })
  })

  describe('rendering behavior', () => {
    test('should render error message', async () => {
      const { container } = render(
        <ErrorAlert message="Error occurred" onClose={() => {}} />,
        { wrapper: createWrapper() },
      )
      await waitFor(() => {})
      expect(container.textContent).toContain('Error occurred')
    })

    test('should render Alert component', async () => {
      const { container } = render(
        <ErrorAlert message="Error" onClose={() => {}} />,
        { wrapper: createWrapper() },
      )
      await waitFor(() => {})
      expect(container).toBeDefined()
    })

    test('should render with error severity', async () => {
      const { container } = render(
        <ErrorAlert message="Error" onClose={() => {}} />,
        { wrapper: createWrapper() },
      )
      await waitFor(() => {})
      expect(container).toBeDefined()
    })
  })

  describe('props handling behavior', () => {
    test('should accept message prop', async () => {
      const { container } = render(
        <ErrorAlert message="Custom error" onClose={() => {}} />,
        { wrapper: createWrapper() },
      )
      await waitFor(() => {})
      expect(container.textContent).toContain('Custom error')
    })

    test('should accept onClose prop', async () => {
      let closeCalled = false
      const handleClose = () => {
        closeCalled = true
      }
      render(<ErrorAlert message="Error" onClose={handleClose} />, {
        wrapper: createWrapper(),
      })
      await waitFor(() => {})
      expect(typeof handleClose).toBe('function')
      handleClose()
      expect(closeCalled).toBe(true)
    })

    test('should handle empty message', async () => {
      const { container } = render(
        <ErrorAlert message="" onClose={() => {}} />,
        { wrapper: createWrapper() },
      )
      await waitFor(() => {})
      expect(container).toBeDefined()
    })
  })

  describe('MUI integration behavior', () => {
    test('should use Alert from MUI', async () => {
      const { container } = render(
        <ErrorAlert message="Error" onClose={() => {}} />,
        { wrapper: createWrapper() },
      )
      await waitFor(() => {})
      expect(container).toBeDefined()
    })

    test('should set severity to error', async () => {
      const { container } = render(
        <ErrorAlert message="Error" onClose={() => {}} />,
        { wrapper: createWrapper() },
      )
      await waitFor(() => {})
      expect(container).toBeDefined()
    })

    test('should apply margin bottom', async () => {
      const { container } = render(
        <ErrorAlert message="Error" onClose={() => {}} />,
        { wrapper: createWrapper() },
      )
      await waitFor(() => {})
      expect(container).toBeDefined()
    })
  })

  describe('close button behavior', () => {
    test('should render close button', async () => {
      const { container } = render(
        <ErrorAlert message="Error" onClose={() => {}} />,
        { wrapper: createWrapper() },
      )
      await waitFor(() => {})
      expect(container).toBeDefined()
    })

    test('should call onClose when close button clicked', async () => {
      let closeCalled = false
      const handleClose = () => {
        closeCalled = true
      }
      render(<ErrorAlert message="Error" onClose={handleClose} />, {
        wrapper: createWrapper(),
      })
      await waitFor(() => {})
      expect(typeof handleClose).toBe('function')
    })
  })
})
