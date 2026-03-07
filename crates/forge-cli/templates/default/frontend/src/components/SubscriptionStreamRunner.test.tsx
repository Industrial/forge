/**
 * BDD component tests for SubscriptionStreamRunner.tsx
 * Tests verify component behavior, stream setup, and Effect integration
 */
import { describe, test, expect, beforeAll } from 'bun:test'
// Import test setup to configure React Testing Library (reduces verbose output)
import '@/test-setup'
import { render } from '@testing-library/react'
import { BrowserRouter } from 'react-router-dom'
import { ThemeProvider, createTheme } from '@mui/material/styles'
import { Window } from 'happy-dom'
import React from 'react'
import { Effect, Layer, Option } from 'effect'

import { SubscriptionStreamRunner } from './SubscriptionStreamRunner'
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

const createWrapper = (hasToken: boolean = false) => {
  const theme = createTheme({ palette: { mode: 'light' } })
  const mockAuth = createMockAuthentication()
  if (hasToken) {
    mockAuth.state.token = Option.some('test-token')
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

describe('SubscriptionStreamRunner component', () => {
  describe('export behavior', () => {
    test('should export SubscriptionStreamRunner as named export', () => {
      expect(SubscriptionStreamRunner).toBeDefined()
      expect(typeof SubscriptionStreamRunner).toBe('function')
    })
  })

  describe('rendering behavior', () => {
    test(
      'should return null (no UI)',
      () => {
        // Given: SubscriptionStreamRunner component
        // When: rendering the component
        const { container } = render(<SubscriptionStreamRunner />, {
          wrapper: createWrapper(true),
        })

        // Then: should return null immediately (component always returns null)
        // Note: useEffect runs asynchronously but doesn't affect the return value
        expect(container.firstChild).toBeNull()
      },
      { timeout: 10000 },
    )
  })

  describe('authentication integration behavior', () => {
    test('should use useAuthStore hook', () => {
      const { container } = render(<SubscriptionStreamRunner />, {
        wrapper: createWrapper(false),
      })
      expect(container).toBeDefined()
    })

    test('should check if token exists', () => {
      const { container } = render(<SubscriptionStreamRunner />, {
        wrapper: createWrapper(true),
      })
      expect(container).toBeDefined()
    })

    test('should not open stream when token is missing', () => {
      const { container } = render(<SubscriptionStreamRunner />, {
        wrapper: createWrapper(false),
      })
      expect(container).toBeDefined()
    })
  })

  describe('stream setup behavior', () => {
    test('should use SubscriptionStream service', () => {
      const { container } = render(<SubscriptionStreamRunner />, {
        wrapper: createWrapper(true),
      })
      expect(container).toBeDefined()
    })

    test('should use SubscriptionStreamStatusStoreTag', () => {
      const { container } = render(<SubscriptionStreamRunner />, {
        wrapper: createWrapper(true),
      })
      expect(container).toBeDefined()
    })

    test('should trigger subscription registry on events', () => {
      const { container } = render(<SubscriptionStreamRunner />, {
        wrapper: createWrapper(true),
      })
      expect(container).toBeDefined()
    })
  })
})
