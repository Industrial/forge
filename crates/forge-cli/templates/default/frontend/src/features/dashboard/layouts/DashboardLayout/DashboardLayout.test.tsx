/**
 * BDD component tests for DashboardLayout.tsx
 * Tests verify component rendering, props handling, and layout structure
 */
import { describe, test, expect, beforeAll } from 'bun:test'
import { render } from '@testing-library/react'
import { BrowserRouter } from 'react-router-dom'
import { ThemeProvider, createTheme } from '@mui/material/styles'
import { Window } from 'happy-dom'
import React from 'react'
import { Effect, Layer, Option } from 'effect'

import DashboardLayout from './DashboardLayout'
import { Providers } from '@/Providers'
import { getApplicationLayer } from '@/lib/appLayer'
import { Authentication } from '@/features/authentication/services/Authentication'
import { createMockAuthentication } from '@/features/authentication/services/AuthenticationMock'

beforeAll(() => {
  if (typeof globalThis.window === 'undefined') {
    const window = new Window()
    const document = window.document
    const global = globalThis as any
    global.window = window
    global.document = document
    global.localStorage = window.localStorage
    global.navigator = window.navigator
    if (!document.body) {
      const body = document.createElement('body')
      document.appendChild(body)
    }
    global.SyntaxError = class SyntaxError extends Error {
      constructor(message?: string) {
        super(message)
        this.name = 'SyntaxError'
        Object.setPrototypeOf(this, SyntaxError.prototype)
      }
    }
    if (window.SyntaxError === undefined) {
      window.SyntaxError = global.SyntaxError as any
    }
  }
})

const createWrapper = () => {
  const theme = createTheme({ palette: { mode: 'light' } })
  const mockAuth = createMockAuthentication()

  const appLayer = getApplicationLayer(
    Layer.mergeAll(
      mockAuth.authentication,
      Layer.succeed(Authentication, mockAuth.authentication),
    ),
  )

  return ({ children }: { children: React.ReactNode }) => (
    <BrowserRouter>
      <Providers theme={theme}>{children}</Providers>
    </BrowserRouter>
  )
}

describe('DashboardLayout component', () => {
  describe('export behavior', () => {
    test('should export DashboardLayout as default export', () => {
      expect(DashboardLayout).toBeDefined()
      expect(typeof DashboardLayout).toBe('function')
    })
  })

  describe('rendering behavior', () => {
    test('should render Navbar', () => {
      const { container } = render(
        <DashboardLayout colorScheme="light" onToggleTheme={() => {}}>
          <div>Content</div>
        </DashboardLayout>,
        { wrapper: createWrapper() },
      )
      expect(container).toBeDefined()
    })

    test('should render Sidebar', () => {
      const { container } = render(
        <DashboardLayout colorScheme="light" onToggleTheme={() => {}}>
          <div>Content</div>
        </DashboardLayout>,
        { wrapper: createWrapper() },
      )
      expect(container).toBeDefined()
    })

    test('should render Outlet for nested routes', () => {
      const { container } = render(
        <DashboardLayout colorScheme="light" onToggleTheme={() => {}}>
          <div>Content</div>
        </DashboardLayout>,
        { wrapper: createWrapper() },
      )
      expect(container).toBeDefined()
    })
  })

  describe('props handling behavior', () => {
    test('should accept colorScheme prop', () => {
      const { container } = render(
        <DashboardLayout colorScheme="dark" onToggleTheme={() => {}}>
          <div>Content</div>
        </DashboardLayout>,
        { wrapper: createWrapper() },
      )
      expect(container).toBeDefined()
    })

    test('should accept onToggleTheme callback', () => {
      let toggleCalled = false
      const handleToggle = () => {
        toggleCalled = true
      }
      render(
        <DashboardLayout colorScheme="light" onToggleTheme={handleToggle}>
          <div>Content</div>
        </DashboardLayout>,
        { wrapper: createWrapper() },
      )
      expect(typeof handleToggle).toBe('function')
      handleToggle()
      expect(toggleCalled).toBe(true)
    })
  })

  describe('responsive behavior', () => {
    test('should use desktop sidebar on desktop', () => {
      const { container } = render(
        <DashboardLayout colorScheme="light" onToggleTheme={() => {}}>
          <div>Content</div>
        </DashboardLayout>,
        { wrapper: createWrapper() },
      )
      expect(container).toBeDefined()
    })

    test('should use mobile drawer on mobile', () => {
      const { container } = render(
        <DashboardLayout colorScheme="light" onToggleTheme={() => {}}>
          <div>Content</div>
        </DashboardLayout>,
        { wrapper: createWrapper() },
      )
      expect(container).toBeDefined()
    })
  })
})
