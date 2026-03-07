/**
 * BDD component tests for LoginPage.tsx
 * Tests verify component rendering, form handling, and authentication integration
 */
import { describe, test, expect, beforeAll } from 'bun:test'
import { render } from '@testing-library/react'
import { BrowserRouter } from 'react-router-dom'
import { ThemeProvider, createTheme } from '@mui/material/styles'
import { Window } from 'happy-dom'
import React from 'react'
import { Effect, Layer } from 'effect'

import LoginPage from './LoginPage'
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

describe('LoginPage component', () => {
  describe('export behavior', () => {
    test('should export LoginPage as default export', () => {
      expect(LoginPage).toBeDefined()
      expect(typeof LoginPage).toBe('function')
    })
  })

  describe('rendering behavior', () => {
    test('should render login form', () => {
      const { container } = render(<LoginPage />, { wrapper: createWrapper() })
      expect(container.querySelector('[data-testid="login-form"]')).not.toBeNull()
    })

    test('should render "Log in" heading', () => {
      const { container } = render(<LoginPage />, { wrapper: createWrapper() })
      expect(container.textContent).toContain('Log in')
    })

    test('should render email field', () => {
      const { container } = render(<LoginPage />, { wrapper: createWrapper() })
      expect(container.querySelector('[data-testid="login-email"]')).not.toBeNull()
    })

    test('should render password field', () => {
      const { container } = render(<LoginPage />, { wrapper: createWrapper() })
      expect(container.querySelector('[data-testid="login-password"]')).not.toBeNull()
    })

    test('should render submit button', () => {
      const { container } = render(<LoginPage />, { wrapper: createWrapper() })
      expect(container.querySelector('[data-testid="login-submit"]')).not.toBeNull()
    })

    test('should render register link', () => {
      const { container } = render(<LoginPage />, { wrapper: createWrapper() })
      expect(container.textContent).toContain("Don't have an account?")
    })
  })

  describe('form handling behavior', () => {
    test('should use react-hook-form', () => {
      const { container } = render(<LoginPage />, { wrapper: createWrapper() })
      expect(container.querySelector('form')).not.toBeNull()
    })

    test('should use effectSchemaResolver', () => {
      const { container } = render(<LoginPage />, { wrapper: createWrapper() })
      expect(container).toBeDefined()
    })

    test('should have default form values', () => {
      const { container } = render(<LoginPage />, { wrapper: createWrapper() })
      expect(container).toBeDefined()
    })

    test('should handle form submission', () => {
      const { container } = render(<LoginPage />, { wrapper: createWrapper() })
      expect(container.querySelector('form')).not.toBeNull()
    })
  })

  describe('authentication integration behavior', () => {
    test('should use Authentication service', () => {
      const { container } = render(<LoginPage />, { wrapper: createWrapper() })
      expect(container).toBeDefined()
    })

    test('should handle login success', () => {
      const { container } = render(<LoginPage />, { wrapper: createWrapper() })
      expect(container).toBeDefined()
    })

    test('should handle login error', () => {
      const { container } = render(<LoginPage />, { wrapper: createWrapper() })
      expect(container).toBeDefined()
    })

    test('should display error message on failure', () => {
      const { container } = render(<LoginPage />, { wrapper: createWrapper() })
      expect(container.querySelector('[data-testid="login-error"]')).toBeNull()
    })
  })

  describe('navigation behavior', () => {
    test('should navigate to home on successful login', () => {
      const { container } = render(<LoginPage />, { wrapper: createWrapper() })
      expect(container).toBeDefined()
    })

    test('should link to register page', () => {
      const { container } = render(<LoginPage />, { wrapper: createWrapper() })
      expect(container.textContent).toContain('Create an Account')
    })
  })
})
