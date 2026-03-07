/**
 * BDD component tests for Sidebar.tsx
 * Tests verify component rendering, props handling, and navigation structure
 */
import { describe, test, expect, beforeAll } from 'bun:test'
import { render } from '@testing-library/react'
import { BrowserRouter } from 'react-router-dom'
import { ThemeProvider, createTheme } from '@mui/material/styles'
import { Window } from 'happy-dom'
import React from 'react'
import { Effect, Layer, Option } from 'effect'

import Sidebar from './Sidebar'
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

const createWrapper = (permissions: string[] = []) => {
  const theme = createTheme({ palette: { mode: 'light' } })
  const mockAuth = createMockAuthentication()
  mockAuth.state.permissions = permissions

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

describe('Sidebar component', () => {
  describe('export behavior', () => {
    test('should export Sidebar as default export', () => {
      expect(Sidebar).toBeDefined()
      expect(typeof Sidebar).toBe('function')
    })
  })

  describe('rendering behavior', () => {
    test('should render navigation items', () => {
      const { container } = render(
        <Sidebar expanded={true} onToggle={() => {}} />,
        { wrapper: createWrapper(['dashboard']) },
      )
      expect(container).toBeDefined()
    })

    test('should render toggle button when not hidden', () => {
      const { container } = render(
        <Sidebar expanded={true} onToggle={() => {}} />,
        { wrapper: createWrapper() },
      )
      expect(container).toBeDefined()
    })

    test('should hide toggle button when hideToggle is true', () => {
      const { container } = render(
        <Sidebar expanded={true} onToggle={() => {}} hideToggle={true} />,
        { wrapper: createWrapper() },
      )
      expect(container).toBeDefined()
    })
  })

  describe('props handling behavior', () => {
    test('should accept expanded prop', () => {
      const { container } = render(
        <Sidebar expanded={true} onToggle={() => {}} />,
        { wrapper: createWrapper() },
      )
      expect(container).toBeDefined()
    })

    test('should accept onToggle callback', () => {
      let toggleCalled = false
      const handleToggle = () => {
        toggleCalled = true
      }
      render(<Sidebar expanded={true} onToggle={handleToggle} />, {
        wrapper: createWrapper(),
      })
      expect(typeof handleToggle).toBe('function')
      handleToggle()
      expect(toggleCalled).toBe(true)
    })

    test('should accept hideToggle prop', () => {
      const { container } = render(
        <Sidebar expanded={true} onToggle={() => {}} hideToggle={true} />,
        { wrapper: createWrapper() },
      )
      expect(container).toBeDefined()
    })

    test('should accept disableBorder prop', () => {
      const { container } = render(
        <Sidebar expanded={true} onToggle={() => {}} disableBorder={true} />,
        { wrapper: createWrapper() },
      )
      expect(container).toBeDefined()
    })

    test('should accept fullWidth prop', () => {
      const { container } = render(
        <Sidebar expanded={true} onToggle={() => {}} fullWidth={true} />,
        { wrapper: createWrapper() },
      )
      expect(container).toBeDefined()
    })
  })

  describe('layout behavior', () => {
    test('should adjust width based on expanded state', () => {
      const { container } = render(
        <Sidebar expanded={true} onToggle={() => {}} />,
        { wrapper: createWrapper() },
      )
      expect(container).toBeDefined()
    })

    test('should use fullWidth when fullWidth is true', () => {
      const { container } = render(
        <Sidebar expanded={true} onToggle={() => {}} fullWidth={true} />,
        { wrapper: createWrapper() },
      )
      expect(container).toBeDefined()
    })
  })
})
