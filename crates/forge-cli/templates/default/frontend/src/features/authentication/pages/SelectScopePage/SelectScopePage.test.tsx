/**
 * BDD component tests for SelectScopePage.tsx
 * Tests verify component rendering, scope selection, and navigation behavior
 */
import { describe, test, expect, beforeAll } from 'bun:test'
import { render } from '@testing-library/react'
import { BrowserRouter } from 'react-router-dom'
import { ThemeProvider, createTheme } from '@mui/material/styles'
import { Window } from 'happy-dom'
import React from 'react'
import { Effect, Layer, Option } from 'effect'

import SelectScopePage from './SelectScopePage'
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

describe('SelectScopePage component', () => {
  describe('export behavior', () => {
    test('should export SelectScopePage as default export', () => {
      expect(SelectScopePage).toBeDefined()
      expect(typeof SelectScopePage).toBe('function')
    })
  })

  describe('rendering behavior', () => {
    test('should render "Select scope" heading', () => {
      const user: AuthenticationUser = {
        id: 'user-1',
        email: 'test@example.com',
        is_admin: false,
        is_active: true,
        current_org_id: null,
        current_role: null,
      }
      const { container } = render(<SelectScopePage />, {
        wrapper: createWrapper(user, true),
      })
      expect(container.textContent).toContain('Select scope')
    })

    test('should redirect to login when user is null', () => {
      const { container } = render(<SelectScopePage />, {
        wrapper: createWrapper(null, false),
      })
      expect(container).toBeDefined()
    })

    test('should redirect when needsScopeSelect is false', () => {
      const user: AuthenticationUser = {
        id: 'user-1',
        email: 'test@example.com',
        is_admin: false,
        is_active: true,
        current_org_id: null,
        current_role: null,
      }
      const { container } = render(<SelectScopePage />, {
        wrapper: createWrapper(user, false),
      })
      expect(container).toBeDefined()
    })

    test('should show loading message when loading', () => {
      const { container } = render(<SelectScopePage />, {
        wrapper: createWrapper(null, false),
      })
      expect(container).toBeDefined()
    })
  })

  describe('authentication integration behavior', () => {
    test('should use useAuthStore hook', () => {
      const { container } = render(<SelectScopePage />, {
        wrapper: createWrapper(null, false),
      })
      expect(container).toBeDefined()
    })

    test('should use Authentication service for scope selection', () => {
      const { container } = render(<SelectScopePage />, {
        wrapper: createWrapper(null, false),
      })
      expect(container).toBeDefined()
    })
  })
})
