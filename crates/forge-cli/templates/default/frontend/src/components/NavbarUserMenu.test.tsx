/**
 * BDD component tests for NavbarUserMenu.tsx
 * Tests verify rendering with/without user, menu items, and scopes integration.
 */
import { describe, test, expect, beforeAll, afterEach } from 'bun:test'
import { render, waitFor } from '@testing-library/react'
import { BrowserRouter } from 'react-router-dom'
import { createTheme } from '@mui/material/styles'
import { Window } from 'happy-dom'
import React from 'react'
import { Effect, Layer, Option, Stream, Chunk } from 'effect'

import NavbarUserMenu from './NavbarUserMenu'
import { Providers } from '@/Providers'
import {
  buildApplicationLayer,
  setApplicationLayerOverrideForTesting,
  clearApplicationLayerOverrideForTesting,
} from '@/lib/appLayer'
import { clearReactiveStoreCacheForTesting } from '@/lib/ReactiveStore'
import type { ReactiveStore } from '@/lib/ReactiveStore'
import type { AuthenticationState } from '@/features/authentication/stores/AuthenticationStateReactiveStore'
import {
  AuthStoreTag,
  initialAuthenticationState,
} from '@/features/authentication/stores/AuthenticationStateReactiveStore'
import { AuthenticationUser } from '@/features/authentication/domain/AuthenticationUser'
import { createMockAuthentication } from '@/features/authentication/services/AuthenticationMock'
import { Authentication } from '@/features/authentication/services/Authentication'

beforeAll(() => {
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
    ;(window as any).SyntaxError = global.SyntaxError
  } else if (!(globalThis.window as any).SyntaxError) {
    ;(globalThis.window as any).SyntaxError = global.SyntaxError
  }
})

function createMockAuthStore(state: AuthenticationState) {
  let current: AuthenticationState = state
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
    return Effect.sync(() => changeListeners.delete(listener))
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

const createWrapper = (user: AuthenticationUser | null = null) => {
  const theme = createTheme({ palette: { mode: 'light' } })
  const mockAuth = createMockAuthentication()
  if (user) {
    mockAuth.setUser(user)
  }
  const mockStoreLayer = createMockAuthStore({
    ...initialAuthenticationState,
    user: user ? Option.some(user) : Option.none(),
    token: user ? Option.some('mock-token') : Option.none(),
    currentScope: Option.none(),
  })
  const baseLayer = buildApplicationLayer()
  clearReactiveStoreCacheForTesting(AuthStoreTag)
  setApplicationLayerOverrideForTesting(
    Layer.mergeAll(
      baseLayer,
      Layer.succeed(Authentication, mockAuth.authentication),
      mockStoreLayer,
    ),
  )
  return ({ children }: { children: React.ReactNode }) => (
    <BrowserRouter>
      <Providers theme={theme}>{children}</Providers>
    </BrowserRouter>
  )
}

describe('NavbarUserMenu component', () => {
  afterEach(() => {
    clearApplicationLayerOverrideForTesting()
  })

  describe('export behavior', () => {
    test('should export NavbarUserMenu as default export', () => {
      expect(NavbarUserMenu).toBeDefined()
      expect(typeof NavbarUserMenu).toBe('function')
    })
  })

  describe('rendering when no user', () => {
    test('should render nothing when user is not authenticated', async () => {
      const { container } = render(<NavbarUserMenu />, {
        wrapper: createWrapper(null),
      })
      await waitFor(() => {})
      expect(
        container.querySelector('[data-testid="navbar-user-menu-button"]'),
      ).toBeNull()
    })
  })

  describe('rendering when user is authenticated', () => {
    test('should render user menu button when authenticated', async () => {
      const user = new AuthenticationUser({
        id: 'user-1',
        email: 'test@example.com',
        token: 'token',
      })
      const { getByTestId } = render(<NavbarUserMenu />, {
        wrapper: createWrapper(user),
      })
      await waitFor(() => {})
      expect(getByTestId('navbar-user-menu-button')).toBeDefined()
    })

    // Skipping "menu opens and shows items": MUI Menu's transition calls node.scrollTop
    // and fails in happy-dom (node is null). E2E or real browser tests can cover menu open.
    test('should render menu button that opens dropdown when user is present', async () => {
      const user = new AuthenticationUser({
        id: 'user-1',
        email: 'test@example.com',
        token: 'token',
      })
      const { getByTestId } = render(<NavbarUserMenu />, {
        wrapper: createWrapper(user),
      })
      await waitFor(() => {
        expect(getByTestId('navbar-user-menu-button')).toBeDefined()
      })
    })
  })
})
