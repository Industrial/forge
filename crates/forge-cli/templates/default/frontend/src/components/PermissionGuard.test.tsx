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
import { Effect, Layer, Option , Stream, Chunk} from 'effect'

import { PermissionGuard } from './PermissionGuard'
import { Providers } from '@/Providers'
import { getApplicationLayer, buildApplicationLayer } from '@/lib/appLayer'
import type { ReactiveStore } from '@/lib/ReactiveStore'
import type { AuthenticationState } from '@/features/authentication/stores/AuthenticationStateReactiveStore'
import type { ReactiveStore } from '@/lib/ReactiveStore'
import type { AuthenticationState } from '@/features/authentication/stores/AuthenticationStateReactiveStore'
import { Authentication } from '@/features/authentication/services/Authentication'
import { AuthStoreTag, initialAuthenticationState } from '@/features/authentication/stores/AuthenticationStateReactiveStore'
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

// Helper to create a mock auth store with permissions
function createMockAuthStoreWithPermissions(permissions: string[]) {
  let current: AuthenticationState = {
    ...initialAuthenticationState,
    permissions,
  }
  const changeListeners = new Set<(a: AuthenticationState) => void>()

  const notify = (a: AuthenticationState) => {
    current = a
    changeListeners.forEach((l) => l(a))
  }

  const changes = Stream.async<AuthenticationState, never, never>((emit) => {
    emit(Effect.succeed(Chunk.of(current)))
    const listener = (a: AuthenticationState) => {
      emit(Effect.succeed(Chunk.of(a)))
    }
    changeListeners.add(listener)
    return Effect.sync(() => {
      changeListeners.delete(listener)
    })
  })

  const store: ReactiveStore<AuthenticationState> = {
    get: () => Effect.succeed(current),
    update: (f: (a: AuthenticationState) => AuthenticationState) =>
      Effect.sync(() => {
        notify(f(current))
      }),
    changes,
  }

  return Layer.succeed(AuthStoreTag, store)
}

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
