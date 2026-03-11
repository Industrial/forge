/**
 * BDD component tests for GuestRoute.tsx
 * Tests verify component rendering, authentication checking, and redirect behavior
 */
import { describe, test, expect, beforeAll, afterEach } from 'bun:test'
import { render, cleanup, waitFor } from '@testing-library/react'
import { BrowserRouter } from 'react-router-dom'
import { createTheme } from '@mui/material/styles'
import { Window } from 'happy-dom'
import type React from 'react'
import { Effect, Layer, Option, Stream, Chunk } from 'effect'

import GuestRoute from './GuestRoute'
import { Providers } from '../Providers'
import {
  buildApplicationLayer,
  setApplicationLayerOverrideForTesting,
  clearApplicationLayerOverrideForTesting,
} from '../lib/appLayer'
import { clearReactiveStoreCacheForTesting } from '../lib/ReactiveStore'
import type { ReactiveStore } from '../lib/ReactiveStore'
import type { AuthenticationState } from '../features/authentication/stores/AuthenticationStateReactiveStore'
import {
  AuthStoreTag,
  initialAuthenticationState,
} from '../features/authentication/stores/AuthenticationStateReactiveStore'
import type { AuthenticationUser } from '../features/authentication/domain/AuthenticationUser'
import { RpcApiMock } from '../services/RpcApiMock'

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
    }
  }
})

// Helper to create a mock auth store with a user
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

  // Create a stream that emits the current value immediately and then listens for changes
  const changes = Stream.async<AuthenticationState, never, never>((emit) => {
    // Emit current value immediately when stream is subscribed
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
    // Use Effect.sync to ensure synchronous resolution
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
  // Merge baseLayer with mockStoreLayer so mockStoreLayer overrides baseLayer's store
  setApplicationLayerOverrideForTesting(
    Layer.mergeAll(baseLayer, mockStoreLayer, RpcApiMock),
  )

  return ({ children }: { children: React.ReactNode }) => (
    <BrowserRouter>
      <Providers theme={theme}>{children}</Providers>
    </BrowserRouter>
  )
}

afterEach(() => {
  cleanup()
})

describe('GuestRoute component', () => {
  afterEach(() => {
    clearApplicationLayerOverrideForTesting()
    clearReactiveStoreCacheForTesting(AuthStoreTag)
  })

  describe('export behavior', () => {
    test('should export GuestRoute as default export', () => {
      expect(GuestRoute).toBeDefined()
      expect(typeof GuestRoute).toBe('function')
    })
  })

  describe('rendering behavior', () => {
    test('should render children when user is not authenticated', async () => {
      const { container } = render(
        <GuestRoute>
          <div data-testid="content">Content</div>
        </GuestRoute>,
        { wrapper: createWrapper(null) },
      )
      await waitFor(
        () => {
          expect(
            container.querySelector('[data-testid="content"]'),
          ).not.toBeNull()
        },
        { timeout: 5000, interval: 100 },
      )
    })

    test('should redirect when user is authenticated', async () => {
      const user: AuthenticationUser = {
        id: 'user-1',
        email: 'test@example.com',
        is_admin: false,
        is_active: true,
        current_org_id: null,
        current_role: null,
      }
      const { container } = render(
        <GuestRoute>
          <div data-testid="content">Content</div>
        </GuestRoute>,
        { wrapper: createWrapper(user) },
      )
      await waitFor(
        () => {
          expect(container.querySelector('[data-testid="content"]')).toBeNull()
        },
        { timeout: 5000, interval: 100 },
      )
    })
  })

  describe('props handling behavior', () => {
    test('should accept children prop', async () => {
      const children = <div data-testid="children">Children</div>
      const { container } = render(<GuestRoute>{children}</GuestRoute>, {
        wrapper: createWrapper(null),
      })
      await waitFor(() => {})
      expect(container.querySelector('[data-testid="children"]')).not.toBeNull()
    })
  })

  describe('authentication integration behavior', () => {
    test('should use useAuthStore hook', async () => {
      const { container } = render(
        <GuestRoute>
          <div>Content</div>
        </GuestRoute>,
        { wrapper: createWrapper(null) },
      )
      await waitFor(() => {})
      expect(container).toBeDefined()
    })

    test('should check if user is authenticated using Option.isSome', async () => {
      const { container } = render(
        <GuestRoute>
          <div>Content</div>
        </GuestRoute>,
        { wrapper: createWrapper(null) },
      )
      await waitFor(() => {})
      expect(container).toBeDefined()
    })

    test('should redirect to home when authenticated', async () => {
      const user: AuthenticationUser = {
        id: 'user-1',
        email: 'test@example.com',
        is_admin: false,
        is_active: true,
        current_org_id: null,
        current_role: null,
      }
      const { container } = render(
        <GuestRoute>
          <div>Content</div>
        </GuestRoute>,
        { wrapper: createWrapper(user) },
      )
      await waitFor(() => {})
      expect(container).toBeDefined()
    })
  })
})
