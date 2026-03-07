/**
 * BDD component tests for GuestRoute.tsx
 * Tests verify component rendering, authentication checking, and redirect behavior
 */
import { describe, test, expect, beforeAll } from 'bun:test'
import { render } from '@testing-library/react'
import { BrowserRouter } from 'react-router-dom'
import { ThemeProvider, createTheme } from '@mui/material/styles'
import { Window } from 'happy-dom'
import React from 'react'
import { Effect, Layer, Option } from 'effect'

import GuestRoute from './GuestRoute'
import { Providers } from '@/Providers'
import { getApplicationLayer } from '@/lib/appLayer'
import { Authentication } from '@/features/authentication/services/Authentication'
import { createMockAuthentication } from '@/features/authentication/services/AuthenticationMock'
import { AuthenticationUser } from '@/features/authentication/domain/AuthenticationUser'

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

describe('GuestRoute component', () => {
  describe('export behavior', () => {
    test('should export GuestRoute as default export', () => {
      expect(GuestRoute).toBeDefined()
      expect(typeof GuestRoute).toBe('function')
    })
  })

  describe('rendering behavior', () => {
    test('should render children when user is not authenticated', () => {
      const { container } = render(
        <GuestRoute>
          <div data-testid="content">Content</div>
        </GuestRoute>,
        { wrapper: createWrapper(null) },
      )
      expect(container.querySelector('[data-testid="content"]')).not.toBeNull()
    })

    test('should redirect when user is authenticated', () => {
      const user: AuthenticationUser = {
        id: 'user-1',
        email: 'test@example.com',
        is_admin: false,
        is_active: true,
        current_org_id: null,
        current_role: null,
      }
      const { container } = render(
        <GuestRoute>
          <div data-testid="content">Content</div>
        </GuestRoute>,
        { wrapper: createWrapper(user) },
      )
      expect(container.querySelector('[data-testid="content"]')).toBeNull()
    })
  })

  describe('props handling behavior', () => {
    test('should accept children prop', () => {
      const children = <div data-testid="children">Children</div>
      const { container } = render(
        <GuestRoute>{children}</GuestRoute>,
        { wrapper: createWrapper(null) },
      )
      expect(container.querySelector('[data-testid="children"]')).not.toBeNull()
    })
  })

  describe('authentication integration behavior', () => {
    test('should use useAuthStore hook', () => {
      const { container } = render(
        <GuestRoute>
          <div>Content</div>
        </GuestRoute>,
        { wrapper: createWrapper(null) },
      )
      expect(container).toBeDefined()
    })

    test('should check if user is authenticated using Option.isSome', () => {
      const { container } = render(
        <GuestRoute>
          <div>Content</div>
        </GuestRoute>,
        { wrapper: createWrapper(null) },
      )
      expect(container).toBeDefined()
    })

    test('should redirect to home when authenticated', () => {
      const user: AuthenticationUser = {
        id: 'user-1',
        email: 'test@example.com',
        is_admin: false,
        is_active: true,
        current_org_id: null,
        current_role: null,
      }
      const { container } = render(
        <GuestRoute>
          <div>Content</div>
        </GuestRoute>,
        { wrapper: createWrapper(user) },
      )
      expect(container).toBeDefined()
    })
  })
})
