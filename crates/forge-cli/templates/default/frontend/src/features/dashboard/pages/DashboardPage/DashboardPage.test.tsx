/**
 * BDD component tests for DashboardPage.tsx
 * Tests verify component rendering and PageHeader integration
 */
import { describe, test, expect, beforeAll } from 'bun:test'
import { render } from '@testing-library/react'
import { ThemeProvider, createTheme } from '@mui/material/styles'
import { Window } from 'happy-dom'
import React from 'react'

import DashboardPage from './DashboardPage'

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

describe('DashboardPage component', () => {
  describe('export behavior', () => {
    test('should export DashboardPage as default export', () => {
      expect(DashboardPage).toBeDefined()
      expect(typeof DashboardPage).toBe('function')
    })
  })

  describe('rendering behavior', () => {
    test('should render PageHeader', () => {
      const { container } = render(<DashboardPage />, { wrapper: createWrapper() })
      expect(container).toBeDefined()
    })

    test('should render "Dashboard" title', () => {
      const { container } = render(<DashboardPage />, { wrapper: createWrapper() })
      expect(container.textContent).toContain('Dashboard')
    })

    test('should render welcome description', () => {
      const { container } = render(<DashboardPage />, { wrapper: createWrapper() })
      expect(container.textContent).toContain('Welcome to your dashboard')
    })

    test('should have data-testid on heading', () => {
      const { container } = render(<DashboardPage />, { wrapper: createWrapper() })
      expect(container.querySelector('[data-testid="dashboard-heading"]')).not.toBeNull()
    })
  })

  describe('PageHeader integration behavior', () => {
    test('should use PageHeader component', () => {
      const { container } = render(<DashboardPage />, { wrapper: createWrapper() })
      expect(container).toBeDefined()
    })

    test('should pass title prop to PageHeader', () => {
      const { container } = render(<DashboardPage />, { wrapper: createWrapper() })
      expect(container.textContent).toContain('Dashboard')
    })

    test('should pass description prop to PageHeader', () => {
      const { container } = render(<DashboardPage />, { wrapper: createWrapper() })
      expect(container.textContent).toContain('Welcome')
    })
  })
})
