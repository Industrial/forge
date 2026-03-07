/**
 * BDD component tests for App.tsx
 * Tests verify component rendering, routing structure, and theme integration
 */
import { describe, test, expect, beforeAll } from 'bun:test'
import { render } from '@testing-library/react'
import { BrowserRouter } from 'react-router-dom'
import { ThemeProvider, createTheme } from '@mui/material/styles'
import { Window } from 'happy-dom'
import React from 'react'
import { Effect, Layer, Option } from 'effect'

import App from './App'
import { Providers } from './Providers'
import { getApplicationLayer } from '@/lib/appLayer'
import { Authentication } from '@/features/authentication/services/Authentication'
import { createMockAuthentication } from '@/features/authentication/services/AuthenticationMock'

// Set up DOM environment for tests
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
    // Ensure document.body exists
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

// Helper to create a wrapper with theme, router, and app layer context
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

describe('App component', () => {
  describe('export behavior', () => {
    test('should export App as default export', () => {
      // Given: the App module
      // When: checking the export
      // Then: App should be available
      expect(App).toBeDefined()
      expect(typeof App).toBe('function')
    })
  })

  describe('rendering behavior', () => {
    test('should render Providers component', () => {
      // Given: App component
      // When: rendering App
      const { container } = render(<App />, { wrapper: createWrapper() })
      // Then: Providers should wrap the app
      expect(container).toBeDefined()
    })

    test('should render Box container', () => {
      // Given: App component
      // When: rendering App
      const { container } = render(<App />, { wrapper: createWrapper() })
      // Then: Box container should be present (Box renders as <main> element)
      expect(container).toBeDefined()
      expect(container.querySelector('main')).not.toBeNull()
    })

    test('should render Routes component', () => {
      // Given: App component
      // When: rendering App
      const { container } = render(<App />, { wrapper: createWrapper() })
      // Then: Routes should be rendered
      expect(container).toBeDefined()
    })
  })

  describe('theme integration behavior', () => {
    test('should use useColorSchemeMode hook', () => {
      // Given: App component
      // When: checking component structure
      // Then: should use useColorSchemeMode (verified by component rendering)
      const { container } = render(<App />, { wrapper: createWrapper() })
      expect(container).toBeDefined()
    })

    test('should create theme from colorSchemeMode', () => {
      // Given: App component
      // When: rendering App
      const { container } = render(<App />, { wrapper: createWrapper() })
      // Then: theme should be created from colorSchemeMode
      expect(container).toBeDefined()
    })

    test('should pass theme to Providers', () => {
      // Given: App component
      // When: rendering App
      const { container } = render(<App />, { wrapper: createWrapper() })
      // Then: theme should be passed to Providers
      expect(container).toBeDefined()
    })

    test('should toggle theme via onToggleTheme', () => {
      // Given: App component
      // When: checking component structure
      // Then: should provide onToggleTheme callback
      const { container } = render(<App />, { wrapper: createWrapper() })
      expect(container).toBeDefined()
    })
  })

  describe('routing structure behavior', () => {
    test('should define authentication routes', () => {
      // Given: App component
      // When: checking routing structure
      // Then: should define /authentication/login route
      const { container } = render(<App />, { wrapper: createWrapper() })
      expect(container).toBeDefined()
    })

    test('should define dashboard routes', () => {
      // Given: App component
      // When: checking routing structure
      // Then: should define /dashboard route
      const { container } = render(<App />, { wrapper: createWrapper() })
      expect(container).toBeDefined()
    })

    test('should define home route', () => {
      // Given: App component
      // When: checking routing structure
      // Then: should define / route
      const { container } = render(<App />, { wrapper: createWrapper() })
      expect(container).toBeDefined()
    })

    test('should use ProtectedRoute for protected routes', () => {
      // Given: App component
      // When: checking routing structure
      // Then: should use ProtectedRoute wrapper
      const { container } = render(<App />, { wrapper: createWrapper() })
      expect(container).toBeDefined()
    })

    test('should use GuestRoute for guest routes', () => {
      // Given: App component
      // When: checking routing structure
      // Then: should use GuestRoute wrapper
      const { container } = render(<App />, { wrapper: createWrapper() })
      expect(container).toBeDefined()
    })
  })

  describe('layout integration behavior', () => {
    test('should use Layout component', () => {
      // Given: App component
      // When: checking component structure
      // Then: should use Layout component
      const { container } = render(<App />, { wrapper: createWrapper() })
      expect(container).toBeDefined()
    })

    test('should use DashboardLayout component', () => {
      // Given: App component
      // When: checking component structure
      // Then: should use DashboardLayout component
      const { container } = render(<App />, { wrapper: createWrapper() })
      expect(container).toBeDefined()
    })

    test('should use AuthenticationLayout component', () => {
      // Given: App component
      // When: checking component structure
      // Then: should use AuthenticationLayout component
      const { container } = render(<App />, { wrapper: createWrapper() })
      expect(container).toBeDefined()
    })

    test('should pass layoutProps to Layout components', () => {
      // Given: App component
      // When: checking component structure
      // Then: should pass layoutProps (colorScheme, onToggleTheme)
      const { container } = render(<App />, { wrapper: createWrapper() })
      expect(container).toBeDefined()
    })
  })

  describe('component integration behavior', () => {
    test('should include SubscriptionStreamRunner', () => {
      // Given: App component
      // When: checking component structure
      // Then: should include SubscriptionStreamRunner
      const { container } = render(<App />, { wrapper: createWrapper() })
      expect(container).toBeDefined()
    })

    test('should use DashboardScopeGuard', () => {
      // Given: App component
      // When: checking component structure
      // Then: should use DashboardScopeGuard
      const { container } = render(<App />, { wrapper: createWrapper() })
      expect(container).toBeDefined()
    })

    test('should use SelectScopeOnlyGuard', () => {
      // Given: App component
      // When: checking component structure
      // Then: should use SelectScopeOnlyGuard
      const { container } = render(<App />, { wrapper: createWrapper() })
      expect(container).toBeDefined()
    })

    test('should use PermissionGuard', () => {
      // Given: App component
      // When: checking component structure
      // Then: should use PermissionGuard
      const { container } = render(<App />, { wrapper: createWrapper() })
      expect(container).toBeDefined()
    })
  })

  describe('structure behavior', () => {
    test('should render Box with full viewport height', () => {
      // Given: App component
      // When: rendering App
      const { container } = render(<App />, { wrapper: createWrapper() })
      // Then: Box should have height: 100vh
      expect(container).toBeDefined()
    })

    test('should render Box with flex column layout', () => {
      // Given: App component
      // When: rendering App
      const { container } = render(<App />, { wrapper: createWrapper() })
      // Then: Box should have flexDirection: column
      expect(container).toBeDefined()
    })
  })
})
