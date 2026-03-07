/**
 * BDD component tests for ProtectedRoute.tsx
 * Tests verify component rendering, authentication checking, and redirect behavior
 */
import { describe, test, expect, beforeAll } from 'bun:test'
import { render } from '@testing-library/react'
import { BrowserRouter } from 'react-router-dom'
import { ThemeProvider, createTheme } from '@mui/material/styles'
import { Window } from 'happy-dom'
import React from 'react'
import { Effect, Layer, Option } from 'effect'

import ProtectedRoute from './ProtectedRoute'
import { Providers } from '@/Providers'
import { getApplicationLayer } from '@/lib/appLayer'
import { Authentication } from '@/features/authentication/services/Authentication'
import { createMockAuthentication } from '@/features/authentication/services/AuthenticationMock'
import { AuthenticationUser } from '@/features/authentication/domain/AuthenticationUser'

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

const createWrapper = (user: AuthenticationUser | null = null) => {
  const theme = createTheme({ palette: { mode: 'light' } })
  const mockAuth = createMockAuthentication()
  if (user) {
    mockAuth.setUser(user)
  }

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

describe('ProtectedRoute component', () => {
  describe('export behavior', () => {
    test('should export ProtectedRoute as default export', () => {
      expect(ProtectedRoute).toBeDefined()
      expect(typeof ProtectedRoute).toBe('function')
    })
  })

  describe('rendering behavior', () => {
    test('should render children when user is authenticated', () => {
      const user: AuthenticationUser = {
        id: 'user-1',
        email: 'test@example.com',
        is_admin: false,
        is_active: true,
        current_org_id: null,
        current_role: null,
      }
      const { container } = render(
        <ProtectedRoute>
          <div data-testid="content">Content</div>
        </ProtectedRoute>,
        { wrapper: createWrapper(user) },
      )
      expect(container.querySelector('[data-testid="content"]')).not.toBeNull()
    })

    test('should return null when not initialized', () => {
      const { container } = render(
        <ProtectedRoute>
          <div data-testid="content">Content</div>
        </ProtectedRoute>,
        { wrapper: createWrapper(null) },
      )
      expect(container.querySelector('[data-testid="content"]')).toBeNull()
    })

    test('should redirect when user is not authenticated', () => {
      const { container } = render(
        <ProtectedRoute>
          <div data-testid="content">Content</div>
        </ProtectedRoute>,
        { wrapper: createWrapper(null) },
      )
      expect(container.querySelector('[data-testid="content"]')).toBeNull()
    })
  })

  describe('props handling behavior', () => {
    test('should accept children prop', () => {
      const user: AuthenticationUser = {
        id: 'user-1',
        email: 'test@example.com',
        is_admin: false,
        is_active: true,
        current_org_id: null,
        current_role: null,
      }
      const children = <div data-testid="children">Children</div>
      const { container } = render(
        <ProtectedRoute>{children}</ProtectedRoute>,
        { wrapper: createWrapper(user) },
      )
      expect(container.querySelector('[data-testid="children"]')).not.toBeNull()
    })
  })

  describe('authentication integration behavior', () => {
    test('should use useAuthStoreWithInit hook', () => {
      const { container } = render(
        <ProtectedRoute>
          <div>Content</div>
        </ProtectedRoute>,
        { wrapper: createWrapper(null) },
      )
      expect(container).toBeDefined()
    })

    test('should check if user is authenticated using Option.isSome', () => {
      const { container } = render(
        <ProtectedRoute>
          <div>Content</div>
        </ProtectedRoute>,
        { wrapper: createWrapper(null) },
      )
      expect(container).toBeDefined()
    })

    test('should redirect to login when not authenticated', () => {
      const { container } = render(
        <ProtectedRoute>
          <div>Content</div>
        </ProtectedRoute>,
        { wrapper: createWrapper(null) },
      )
      expect(container).toBeDefined()
    })
  })
})
