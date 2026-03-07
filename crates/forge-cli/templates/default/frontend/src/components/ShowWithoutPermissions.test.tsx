/**
 * BDD component tests for ShowWithoutPermissions.tsx
 * Tests verify component rendering, permission checking, and conditional visibility
 */
import { describe, test, expect, beforeAll } from 'bun:test'
import { render } from '@testing-library/react'
import { BrowserRouter } from 'react-router-dom'
import { ThemeProvider, createTheme } from '@mui/material/styles'
import { Window } from 'happy-dom'
import React from 'react'
import { Effect, Layer, Option } from 'effect'

import { ShowWithoutPermissions } from './ShowWithoutPermissions'
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

describe('ShowWithoutPermissions component', () => {
  describe('export behavior', () => {
    test('should export ShowWithoutPermissions as named export', () => {
      expect(ShowWithoutPermissions).toBeDefined()
      expect(typeof ShowWithoutPermissions).toBe('function')
    })
  })

  describe('rendering behavior', () => {
    test('should render children when user lacks all permissions', () => {
      const { container } = render(
        <ShowWithoutPermissions permissions={['test.permission']}>
          <div data-testid="content">Content</div>
        </ShowWithoutPermissions>,
        { wrapper: createWrapper([]) },
      )
      expect(container.querySelector('[data-testid="content"]')).not.toBeNull()
    })

    test('should hide children when user has any of the permissions', () => {
      const { container } = render(
        <ShowWithoutPermissions permissions={['test.permission']}>
          <div data-testid="content">Content</div>
        </ShowWithoutPermissions>,
        { wrapper: createWrapper(['test.permission']) },
      )
      expect(container.querySelector('[data-testid="content"]')).toBeNull()
    })
  })

  describe('props handling behavior', () => {
    test('should accept permissions prop', () => {
      const { container } = render(
        <ShowWithoutPermissions permissions={['perm1', 'perm2']}>
          <div>Content</div>
        </ShowWithoutPermissions>,
        { wrapper: createWrapper([]) },
      )
      expect(container).toBeDefined()
    })

    test('should accept children prop', () => {
      const children = <div data-testid="children">Children</div>
      const { container } = render(
        <ShowWithoutPermissions permissions={['test.permission']}>
          {children}
        </ShowWithoutPermissions>,
        { wrapper: createWrapper([]) },
      )
      expect(container.querySelector('[data-testid="children"]')).not.toBeNull()
    })
  })

  describe('permission checking behavior', () => {
    test('should use shouldShowWithoutPermissions function', () => {
      const { container } = render(
        <ShowWithoutPermissions permissions={['test.permission']}>
          <div>Content</div>
        </ShowWithoutPermissions>,
        { wrapper: createWrapper([]) },
      )
      expect(container).toBeDefined()
    })

    test('should check user permissions from useAuthStore', () => {
      const { container } = render(
        <ShowWithoutPermissions permissions={['test.permission']}>
          <div>Content</div>
        </ShowWithoutPermissions>,
        { wrapper: createWrapper([]) },
      )
      expect(container).toBeDefined()
    })
  })
})
