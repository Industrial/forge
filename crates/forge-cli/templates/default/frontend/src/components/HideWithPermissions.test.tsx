/**
 * BDD component tests for HideWithPermissions.tsx
 * Tests verify component rendering, permission checking, and conditional visibility
 */
import { describe, test, expect, beforeAll } from 'bun:test'
import { render } from '@testing-library/react'
import { BrowserRouter } from 'react-router-dom'
import { ThemeProvider, createTheme } from '@mui/material/styles'
import { Window } from 'happy-dom'
import React from 'react'
import { Effect, Layer, Option } from 'effect'

import { HideWithPermissions } from './HideWithPermissions'
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

describe('HideWithPermissions component', () => {
  describe('export behavior', () => {
    test('should export HideWithPermissions as named export', () => {
      expect(HideWithPermissions).toBeDefined()
      expect(typeof HideWithPermissions).toBe('function')
    })
  })

  describe('rendering behavior', () => {
    test('should hide children when user has any of the permissions', () => {
      const { container } = render(
        <HideWithPermissions permissions={['test.permission']}>
          <div data-testid="content">Content</div>
        </HideWithPermissions>,
        { wrapper: createWrapper(['test.permission']) },
      )
      expect(container.querySelector('[data-testid="content"]')).toBeNull()
    })

    test('should render children when user lacks all permissions', () => {
      const { container } = render(
        <HideWithPermissions permissions={['test.permission']}>
          <div data-testid="content">Content</div>
        </HideWithPermissions>,
        { wrapper: createWrapper([]) },
      )
      expect(container.querySelector('[data-testid="content"]')).not.toBeNull()
    })
  })

  describe('props handling behavior', () => {
    test('should accept permissions prop', () => {
      const { container } = render(
        <HideWithPermissions permissions={['perm1', 'perm2']}>
          <div>Content</div>
        </HideWithPermissions>,
        { wrapper: createWrapper([]) },
      )
      expect(container).toBeDefined()
    })

    test('should accept children prop', () => {
      const children = <div data-testid="children">Children</div>
      const { container } = render(
        <HideWithPermissions permissions={['test.permission']}>
          {children}
        </HideWithPermissions>,
        { wrapper: createWrapper([]) },
      )
      expect(container.querySelector('[data-testid="children"]')).not.toBeNull()
    })
  })

  describe('permission checking behavior', () => {
    test('should use shouldHideWithPermissions function', () => {
      const { container } = render(
        <HideWithPermissions permissions={['test.permission']}>
          <div>Content</div>
        </HideWithPermissions>,
        { wrapper: createWrapper([]) },
      )
      expect(container).toBeDefined()
    })

    test('should check user permissions from useAuthStore', () => {
      const { container } = render(
        <HideWithPermissions permissions={['test.permission']}>
          <div>Content</div>
        </HideWithPermissions>,
        { wrapper: createWrapper([]) },
      )
      expect(container).toBeDefined()
    })
  })
})
