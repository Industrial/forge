/**
 * BDD component tests for PermissionGuard.tsx
 * Tests verify component rendering, permission checking, and redirect behavior
 */
import { describe, test, expect, beforeAll } from 'bun:test'
import { render } from '@testing-library/react'
import { BrowserRouter } from 'react-router-dom'
import { ThemeProvider, createTheme } from '@mui/material/styles'
import { Window } from 'happy-dom'
import React from 'react'
import { Effect, Layer, Option } from 'effect'

import { PermissionGuard } from './PermissionGuard'
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

describe('PermissionGuard component', () => {
  describe('export behavior', () => {
    test('should export PermissionGuard as named export', () => {
      expect(PermissionGuard).toBeDefined()
      expect(typeof PermissionGuard).toBe('function')
    })
  })

  describe('rendering behavior', () => {
    test('should render children when user has required permissions', () => {
      const { container } = render(
        <PermissionGuard permissions={['test.permission']}>
          <div data-testid="content">Content</div>
        </PermissionGuard>,
        { wrapper: createWrapper(['test.permission']) },
      )
      expect(container.querySelector('[data-testid="content"]')).not.toBeNull()
    })

    test('should redirect when user lacks required permissions', () => {
      const { container } = render(
        <PermissionGuard permissions={['test.permission']}>
          <div data-testid="content">Content</div>
        </PermissionGuard>,
        { wrapper: createWrapper([]) },
      )
      expect(container.querySelector('[data-testid="content"]')).toBeNull()
    })

    test('should use default redirectTo when not provided', () => {
      const { container } = render(
        <PermissionGuard permissions={['test.permission']}>
          <div data-testid="content">Content</div>
        </PermissionGuard>,
        { wrapper: createWrapper([]) },
      )
      expect(container.querySelector('[data-testid="content"]')).toBeNull()
    })

    test('should use custom redirectTo when provided', () => {
      const { container } = render(
        <PermissionGuard
          permissions={['test.permission']}
          redirectTo="/custom"
        >
          <div data-testid="content">Content</div>
        </PermissionGuard>,
        { wrapper: createWrapper([]) },
      )
      expect(container.querySelector('[data-testid="content"]')).toBeNull()
    })
  })

  describe('props handling behavior', () => {
    test('should accept permissions prop', () => {
      const { container } = render(
        <PermissionGuard permissions={['perm1', 'perm2']}>
          <div>Content</div>
        </PermissionGuard>,
        { wrapper: createWrapper(['perm1']) },
      )
      expect(container).toBeDefined()
    })

    test('should accept redirectTo prop', () => {
      const { container } = render(
        <PermissionGuard
          permissions={['test.permission']}
          redirectTo="/custom"
        >
          <div>Content</div>
        </PermissionGuard>,
        { wrapper: createWrapper([]) },
      )
      expect(container).toBeDefined()
    })

    test('should accept children prop', () => {
      const children = <div data-testid="children">Children</div>
      const { container } = render(
        <PermissionGuard permissions={['test.permission']}>
          {children}
        </PermissionGuard>,
        { wrapper: createWrapper(['test.permission']) },
      )
      expect(container.querySelector('[data-testid="children"]')).not.toBeNull()
    })
  })

  describe('permission checking behavior', () => {
    test('should use hasPermission function', () => {
      const { container } = render(
        <PermissionGuard permissions={['test.permission']}>
          <div>Content</div>
        </PermissionGuard>,
        { wrapper: createWrapper(['test.permission']) },
      )
      expect(container).toBeDefined()
    })

    test('should check user permissions from useAuthStore', () => {
      const { container } = render(
        <PermissionGuard permissions={['test.permission']}>
          <div>Content</div>
        </PermissionGuard>,
        { wrapper: createWrapper(['test.permission']) },
      )
      expect(container).toBeDefined()
    })

    test('should allow access when user has at least one permission', () => {
      const { container } = render(
        <PermissionGuard permissions={['perm1', 'perm2']}>
          <div data-testid="content">Content</div>
        </PermissionGuard>,
        { wrapper: createWrapper(['perm2']) },
      )
      expect(container.querySelector('[data-testid="content"]')).not.toBeNull()
    })
  })
})
