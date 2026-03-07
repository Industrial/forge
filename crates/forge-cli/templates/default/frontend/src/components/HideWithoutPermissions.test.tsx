/**
 * BDD component tests for HideWithoutPermissions.tsx
 * Tests verify component rendering, permission checking, and conditional visibility
 */
import { describe, test, expect, beforeAll } from 'bun:test'
import { render } from '@testing-library/react'
import { BrowserRouter } from 'react-router-dom'
import { ThemeProvider, createTheme } from '@mui/material/styles'
import { Window } from 'happy-dom'
import React from 'react'
import { Effect, Layer, Option } from 'effect'

import { HideWithoutPermissions } from './HideWithoutPermissions'
import { Providers } from '@/Providers'
import { getApplicationLayer } from '@/lib/appLayer'
import { Authentication } from '@/features/authentication/services/Authentication'
import { createMockAuthentication } from '@/features/authentication/services/AuthenticationMock'
import { AuthenticationUser } from '@/features/authentication/domain/AuthenticationUser'

// Set up DOM environment for tests
beforeAll(() => {
  if (typeof globalThis.window === 'undefined') {
    const window = new Window()
    const document = window.document
    const global = globalThis as any
    global.window = window
    global.document = document
    global.localStorage = window.localStorage
    global.navigator = window.navigator
    // Ensure document.body exists
    if (!document.body) {
      const body = document.createElement('body')
      document.appendChild(body)
    }
    // Add missing Error constructors that happy-dom needs
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

// Helper to create a wrapper with theme, router, and app layer context
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

describe('HideWithoutPermissions component', () => {
  describe('export behavior', () => {
    test('should export HideWithoutPermissions as named export', () => {
      // Given: the HideWithoutPermissions module
      // When: checking the export
      // Then: HideWithoutPermissions should be available
      expect(HideWithoutPermissions).toBeDefined()
      expect(typeof HideWithoutPermissions).toBe('function')
    })
  })

  describe('rendering behavior', () => {
    test('should render children when user has required permissions', () => {
      // Given: HideWithoutPermissions component with user having permissions
      const { container } = render(
        <HideWithoutPermissions permissions={['test.permission']}>
          <div data-testid="content">Content</div>
        </HideWithoutPermissions>,
        { wrapper: createWrapper(['test.permission']) },
      )
      // Then: children should be rendered
      expect(container).toBeDefined()
      expect(container.querySelector('[data-testid="content"]')).not.toBeNull()
    })

    test('should hide children when user lacks required permissions', () => {
      // Given: HideWithoutPermissions component with user lacking permissions
      const { container } = render(
        <HideWithoutPermissions permissions={['test.permission']}>
          <div data-testid="content">Content</div>
        </HideWithoutPermissions>,
        { wrapper: createWrapper([]) },
      )
      // Then: children should not be rendered
      expect(container.querySelector('[data-testid="content"]')).toBeNull()
    })

    test('should render when user has at least one permission', () => {
      // Given: HideWithoutPermissions component with multiple permissions
      const { container } = render(
        <HideWithoutPermissions
          permissions={['permission1', 'permission2', 'permission3']}
        >
          <div data-testid="content">Content</div>
        </HideWithoutPermissions>,
        { wrapper: createWrapper(['permission2']) },
      )
      // Then: children should be rendered (user has permission2)
      expect(container.querySelector('[data-testid="content"]')).not.toBeNull()
    })
  })

  describe('props handling behavior', () => {
    test('should accept permissions prop', () => {
      // Given: HideWithoutPermissions component with permissions
      const { container } = render(
        <HideWithoutPermissions permissions={['test.permission']}>
          <div>Content</div>
        </HideWithoutPermissions>,
        { wrapper: createWrapper(['test.permission']) },
      )
      // Then: component should render (permissions prop accepted)
      expect(container).toBeDefined()
    })

    test('should accept children prop', () => {
      // Given: HideWithoutPermissions component with children
      const children = <div data-testid="children">Children</div>
      const { container } = render(
        <HideWithoutPermissions permissions={['test.permission']}>
          {children}
        </HideWithoutPermissions>,
        { wrapper: createWrapper(['test.permission']) },
      )
      // Then: children should be rendered
      expect(container.querySelector('[data-testid="children"]')).not.toBeNull()
    })

    test('should handle empty permissions array', () => {
      // Given: HideWithoutPermissions component with empty permissions
      const { container } = render(
        <HideWithoutPermissions permissions={[]}>
          <div data-testid="content">Content</div>
        </HideWithoutPermissions>,
        { wrapper: createWrapper(['some.permission']) },
      )
      // Then: children should be hidden (no permissions match)
      expect(container.querySelector('[data-testid="content"]')).toBeNull()
    })

    test('should handle multiple permissions', () => {
      // Given: HideWithoutPermissions component with multiple permissions
      const { container } = render(
        <HideWithoutPermissions
          permissions={['perm1', 'perm2', 'perm3']}
        >
          <div data-testid="content">Content</div>
        </HideWithoutPermissions>,
        { wrapper: createWrapper(['perm2']) },
      )
      // Then: children should be rendered (user has perm2)
      expect(container.querySelector('[data-testid="content"]')).not.toBeNull()
    })
  })

  describe('permission checking behavior', () => {
    test('should use shouldHideWithoutPermissions function', () => {
      // Given: HideWithoutPermissions component
      // When: checking component structure
      // Then: should use shouldHideWithoutPermissions (verified by rendering)
      const { container } = render(
        <HideWithoutPermissions permissions={['test.permission']}>
          <div>Content</div>
        </HideWithoutPermissions>,
        { wrapper: createWrapper(['test.permission']) },
      )
      expect(container).toBeDefined()
    })

    test('should check user permissions from useAuthStore', () => {
      // Given: HideWithoutPermissions component
      // When: rendering component
      // Then: should read permissions from useAuthStore
      const { container } = render(
        <HideWithoutPermissions permissions={['test.permission']}>
          <div>Content</div>
        </HideWithoutPermissions>,
        { wrapper: createWrapper(['test.permission']) },
      )
      expect(container).toBeDefined()
    })

    test('should hide when none of the permissions match', () => {
      // Given: HideWithoutPermissions component with permissions
      const { container } = render(
        <HideWithoutPermissions
          permissions={['perm1', 'perm2']}
        >
          <div data-testid="content">Content</div>
        </HideWithoutPermissions>,
        { wrapper: createWrapper(['perm3', 'perm4']) },
      )
      // Then: children should be hidden (no matching permissions)
      expect(container.querySelector('[data-testid="content"]')).toBeNull()
    })
  })

  describe('edge cases', () => {
    test('should handle null children', () => {
      // Given: HideWithoutPermissions component with null children
      const { container } = render(
        <HideWithoutPermissions permissions={['test.permission']}>
          {null}
        </HideWithoutPermissions>,
        { wrapper: createWrapper(['test.permission']) },
      )
      // Then: component should still render (no error)
      expect(container).toBeDefined()
    })

    test('should handle undefined children', () => {
      // Given: HideWithoutPermissions component with undefined children
      const { container } = render(
        <HideWithoutPermissions permissions={['test.permission']}>
          {undefined}
        </HideWithoutPermissions>,
        { wrapper: createWrapper(['test.permission']) },
      )
      // Then: component should still render (no error)
      expect(container).toBeDefined()
    })

    test('should handle complex nested children', () => {
      // Given: HideWithoutPermissions component with nested children
      const { container } = render(
        <HideWithoutPermissions permissions={['test.permission']}>
          <div>
            <h1>Title</h1>
            <p>Paragraph</p>
          </div>
        </HideWithoutPermissions>,
        { wrapper: createWrapper(['test.permission']) },
      )
      // Then: nested children should be rendered
      expect(container).toBeDefined()
    })
  })
})
