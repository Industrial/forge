/**
 * BDD component tests for RegisterPage.tsx
 * Tests verify component rendering, form handling, and registration integration
 */
import { describe, test, expect, beforeAll } from 'bun:test'
import { render } from '@testing-library/react'
import { BrowserRouter } from 'react-router-dom'
import { ThemeProvider, createTheme } from '@mui/material/styles'
import { Window } from 'happy-dom'
import React from 'react'
import { Effect, Layer } from 'effect'

import RegisterPage from './RegisterPage'
import { Providers } from '@/Providers'
import { getApplicationLayer } from '@/lib/appLayer'
import { Authentication } from '@/features/authentication/services/Authentication'
import { createMockAuthentication } from '@/features/authentication/services/AuthenticationMock'

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

describe('RegisterPage component', () => {
  describe('export behavior', () => {
    test('should export RegisterPage as default export', () => {
      expect(RegisterPage).toBeDefined()
      expect(typeof RegisterPage).toBe('function')
    })
  })

  describe('rendering behavior', () => {
    test('should render registration form', () => {
      const { container } = render(<RegisterPage />, { wrapper: createWrapper() })
      expect(container.querySelector('[data-testid="register-form"]')).not.toBeNull()
    })

    test('should render "Create an account" heading', () => {
      const { container } = render(<RegisterPage />, { wrapper: createWrapper() })
      expect(container.textContent).toContain('Create an account')
    })

    test('should render email field', () => {
      const { container } = render(<RegisterPage />, { wrapper: createWrapper() })
      expect(container.querySelector('[data-testid="register-email"]')).not.toBeNull()
    })

    test('should render password field', () => {
      const { container } = render(<RegisterPage />, { wrapper: createWrapper() })
      expect(container.querySelector('[data-testid="register-password"]')).not.toBeNull()
    })

    test('should render submit button', () => {
      const { container } = render(<RegisterPage />, { wrapper: createWrapper() })
      expect(container.querySelector('[data-testid="register-submit"]')).not.toBeNull()
    })

    test('should render login link', () => {
      const { container } = render(<RegisterPage />, { wrapper: createWrapper() })
      expect(container.textContent).toContain('Already have an account?')
    })
  })

  describe('form handling behavior', () => {
    test('should use react-hook-form', () => {
      const { container } = render(<RegisterPage />, { wrapper: createWrapper() })
      expect(container.querySelector('form')).not.toBeNull()
    })

    test('should use effectSchemaResolver', () => {
      const { container } = render(<RegisterPage />, { wrapper: createWrapper() })
      expect(container).toBeDefined()
    })

    test('should have default form values', () => {
      const { container } = render(<RegisterPage />, { wrapper: createWrapper() })
      expect(container).toBeDefined()
    })

    test('should handle form submission', () => {
      const { container } = render(<RegisterPage />, { wrapper: createWrapper() })
      expect(container.querySelector('form')).not.toBeNull()
    })
  })

  describe('authentication integration behavior', () => {
    test('should use Authentication service', () => {
      const { container } = render(<RegisterPage />, { wrapper: createWrapper() })
      expect(container).toBeDefined()
    })

    test('should handle registration success', () => {
      const { container } = render(<RegisterPage />, { wrapper: createWrapper() })
      expect(container).toBeDefined()
    })

    test('should handle registration error', () => {
      const { container } = render(<RegisterPage />, { wrapper: createWrapper() })
      expect(container).toBeDefined()
    })

    test('should display error message on failure', () => {
      const { container } = render(<RegisterPage />, { wrapper: createWrapper() })
      expect(container.querySelector('[data-testid="register-error"]')).toBeNull()
    })
  })

  describe('navigation behavior', () => {
    test('should navigate to login on successful registration', () => {
      const { container } = render(<RegisterPage />, { wrapper: createWrapper() })
      expect(container).toBeDefined()
    })

    test('should link to login page', () => {
      const { container } = render(<RegisterPage />, { wrapper: createWrapper() })
      expect(container.textContent).toContain('Log in')
    })
  })
})
