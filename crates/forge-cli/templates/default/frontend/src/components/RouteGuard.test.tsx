/**
 * BDD component tests for RouteGuard.tsx
 * Tests use Effect's Layer system to provide auth store (no vi.mock()).
 */
import {
  describe,
  test,
  expect,
  beforeAll,
  beforeEach,
  afterEach,
} from 'bun:test'
import { render, waitFor } from '@testing-library/react'
import { MemoryRouter } from 'react-router-dom'
import { createTheme } from '@mui/material/styles'
import { Window } from 'happy-dom'
import React from 'react'
import { Effect, Layer, Option, Stream, Chunk } from 'effect'

import { RouteGuard } from './RouteGuard'
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
import { RpcApiMock } from '@/services/RpcApiMock'

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
  } else {
    if (!(globalThis.window as any).SyntaxError) {
      ;(globalThis.window as any).SyntaxError = global.SyntaxError
    }
  }
})

function createMockAuthStore(user: AuthenticationUser | null) {
  let current: AuthenticationState = {
    ...initialAuthenticationState,
    user: user ? Option.some(user) : Option.none(),
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
    return Effect.sync(() => changeListeners.delete(listener))
  })
  const store: ReactiveStore<AuthenticationState> = {
    get: () => Effect.sync(() => current),
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
  const mockStoreLayer = createMockAuthStore(user)
  const baseLayer = buildApplicationLayer()
  clearReactiveStoreCacheForTesting(AuthStoreTag)
  setApplicationLayerOverrideForTesting(
    Layer.mergeAll(baseLayer, mockStoreLayer, RpcApiMock),
  )
  return ({ children }: { children: React.ReactNode }) => (
    <MemoryRouter initialEntries={['/']}>
      <Providers theme={theme}>{children}</Providers>
    </MemoryRouter>
  )
}

const mockUser: AuthenticationUser = {
  id: 'user-1',
  email: 'test@example.com',
  permissions: [],
}

describe('RouteGuard component', () => {
  beforeEach(() => {
    clearApplicationLayerOverrideForTesting()
    clearReactiveStoreCacheForTesting(AuthStoreTag)
  })
  afterEach(() => {
    clearApplicationLayerOverrideForTesting()
  })

  describe('export behavior', () => {
    test('should export RouteGuard as named export', () => {
      expect(RouteGuard).toBeDefined()
      expect(typeof RouteGuard).toBe('function')
    })
  })

  describe('redirectIfAuthenticated behavior', () => {
    test('should render children when not authenticated and redirectIfAuthenticated set', () => {
      const { getByTestId, getByText } = render(
        <RouteGuard redirectIfAuthenticated="/dashboard">
          <span data-testid="guest-content">Guest content</span>
        </RouteGuard>,
        { wrapper: createWrapper(null) },
      )
      expect(getByTestId('guest-content')).toBeTruthy()
      expect(getByText('Guest content')).toBeTruthy()
    })

    test('should redirect when authenticated and redirectIfAuthenticated set', async () => {
      const { queryByTestId } = render(
        <RouteGuard redirectIfAuthenticated="/dashboard">
          <span data-testid="guest-content">Guest content</span>
        </RouteGuard>,
        { wrapper: createWrapper(mockUser) },
      )
      await waitFor(() => {
        expect(queryByTestId('guest-content')).toBeNull()
      })
    })
  })

  describe('requireAuth behavior', () => {
    test('should redirect to login when requireAuth and not authenticated', () => {
      const { queryByTestId } = render(
        <RouteGuard requireAuth>
          <span data-testid="protected">Protected</span>
        </RouteGuard>,
        { wrapper: createWrapper(null) },
      )
      expect(queryByTestId('protected')).toBeNull()
    })

    test('should render children when requireAuth and authenticated', async () => {
      const { getByTestId } = render(
        <RouteGuard requireAuth>
          <span data-testid="protected">Protected</span>
        </RouteGuard>,
        { wrapper: createWrapper(mockUser) },
      )
      await waitFor(() => {
        expect(getByTestId('protected')).toBeTruthy()
      })
    })
  })
})
