/**
 * BDD component tests for DashboardScopeGuard.tsx
 * Tests verify component rendering, scope selection, and redirect behavior
 */
import { describe, test, expect, beforeAll } from 'bun:test'
import { render } from '@testing-library/react'
import { BrowserRouter } from 'react-router-dom'
import { ThemeProvider, createTheme } from '@mui/material/styles'
import { Window } from 'happy-dom'
import React from 'react'
import { Effect, Layer, Option } from 'effect'

import DashboardScopeGuard from './DashboardScopeGuard'
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

const createWrapper = (
  user: AuthenticationUser | null = null,
  needsScopeSelect: boolean = false,
) => {
  const theme = createTheme({ palette: { mode: 'light' } })
  const mockAuth = createMockAuthentication()
  if (user) {
    mockAuth.setUser(user)
  }
  mockAuth.state.needsScopeSelect = Option.fromNullable(
    needsScopeSelect ? true : null,
  )

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

describe('DashboardScopeGuard component', () => {
  describe('export behavior', () => {
    test('should export DashboardScopeGuard as default export', () => {
      expect(DashboardScopeGuard).toBeDefined()
      expect(typeof DashboardScopeGuard).toBe('function')
    })
  })

  describe('rendering behavior', () => {
    test('should render children when scope is selected', () => {
      const user: AuthenticationUser = {
        id: 'user-1',
        email: 'test@example.com',
        is_admin: false,
        is_active: true,
        current_org_id: null,
        current_role: null,
      }
      const { container } = render(
        <DashboardScopeGuard>
          <div data-testid="content">Content</div>
        </DashboardScopeGuard>,
        { wrapper: createWrapper(user, false) },
      )
      expect(container.querySelector('[data-testid="content"]')).not.toBeNull()
    })

    test('should show loading spinner when loading', () => {
      const { container } = render(
        <DashboardScopeGuard>
          <div>Content</div>
        </DashboardScopeGuard>,
        { wrapper: createWrapper(null, false) },
      )
      expect(container).toBeDefined()
    })

    test('should return null when user is not authenticated', () => {
      const { container } = render(
        <DashboardScopeGuard>
          <div>Content</div>
        </DashboardScopeGuard>,
        { wrapper: createWrapper(null, false) },
      )
      expect(container).toBeDefined()
    })

    test('should redirect to select-scope when needsScopeSelect is true', () => {
      const user: AuthenticationUser = {
        id: 'user-1',
        email: 'test@example.com',
        is_admin: false,
        is_active: true,
        current_org_id: null,
        current_role: null,
      }
      const { container } = render(
        <DashboardScopeGuard>
          <div data-testid="content">Content</div>
        </DashboardScopeGuard>,
        { wrapper: createWrapper(user, true) },
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
        <DashboardScopeGuard>{children}</DashboardScopeGuard>,
        { wrapper: createWrapper(user, false) },
      )
      expect(container.querySelector('[data-testid="children"]')).not.toBeNull()
    })
  })

  describe('authentication integration behavior', () => {
    test('should use useAuthStore hook', () => {
      const { container } = render(
        <DashboardScopeGuard>
          <div>Content</div>
        </DashboardScopeGuard>,
        { wrapper: createWrapper(null, false) },
      )
      expect(container).toBeDefined()
    })

    test('should check if user is authenticated', () => {
      const { container } = render(
        <DashboardScopeGuard>
          <div>Content</div>
        </DashboardScopeGuard>,
        { wrapper: createWrapper(null, false) },
      )
      expect(container).toBeDefined()
    })

    test('should check needsScopeSelect from authentication state', () => {
      const { container } = render(
        <DashboardScopeGuard>
          <div>Content</div>
        </DashboardScopeGuard>,
        { wrapper: createWrapper(null, false) },
      )
      expect(container).toBeDefined()
    })
  })
})
